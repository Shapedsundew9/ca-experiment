//! Excitable node dynamics and network substrate for EXP-2026-008a.

use super::config::ExperimentConfig;

/// Deterministic pseudo-random number generator based on Xoshiro256++
#[derive(Debug, Clone)]
pub struct FastRng {
    s: [u64; 4],
}

impl FastRng {
    pub fn seed_from_u64(seed: u64) -> Self {
        let mut sm = seed;
        let mut next_sm = || {
            sm = sm.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = sm;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        };
        let s0 = next_sm();
        let s1 = next_sm();
        let s2 = next_sm();
        let s3 = next_sm();
        Self {
            s: [if (s0 | s1 | s2 | s3) == 0 { 1 } else { s0 }, s1, s2, s3],
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let res = (self.s[0].wrapping_add(self.s[3]))
            .rotate_left(23)
            .wrapping_add(self.s[0]);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        res
    }

    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    #[inline]
    pub fn bernoulli(&mut self, p: f64) -> bool {
        if p <= 0.0 {
            false
        } else if p >= 1.0 {
            true
        } else {
            self.next_f64() < p
        }
    }

    #[inline]
    pub fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        let u1 = self.next_f64().max(1e-15);
        let u2 = self.next_f64();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        mean + z0 * std_dev
    }
}

#[derive(Debug, Clone)]
pub struct CircuitNode {
    pub id: usize,
    pub name: String,
    pub v: f64,
    pub refractory_counter: usize,
    pub n_ref: usize,
    pub rolling_rate: f64,
    pub theta: f64,
    pub theta_floor: f64,
    pub theta_max: f64,
    pub beta_theta: f64,
    pub spike: f64,
}

impl CircuitNode {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: usize,
        name: String,
        theta_init: f64,
        theta_floor: f64,
        theta_max: f64,
        beta_theta: f64,
        n_ref: usize,
        rho_target: f64,
    ) -> Self {
        Self {
            id,
            name,
            v: 0.0,
            refractory_counter: 0,
            n_ref,
            rolling_rate: rho_target,
            theta: theta_init,
            theta_floor,
            theta_max,
            beta_theta,
            spike: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitSubstrate {
    pub config: ExperimentConfig,
    pub nodes: Vec<CircuitNode>,
    /// in_neighbors[i] = list of (source_node_id, synaptic_weight)
    pub in_neighbors: Vec<Vec<(usize, f64)>>,
    pub prev_spikes: Vec<f64>,
    pub rng: FastRng,
    pub in_d_idx: usize,
    pub in_we_idx: usize,
    pub in_s_idx: usize,
    pub in_r_idx: usize,
    pub in_re_idx: usize,
    pub gate_set_idx: usize,
    pub gate_rst_idx: usize,
    pub v_inh_idx: usize,
    pub r0_idx: usize,
    pub r1_idx: usize,
    pub r2_idx: usize,
    pub r3_idx: usize,
    pub v_sense_idx: usize,
    pub v_read_idx: usize,
    pub total_spikes: usize,
    pub total_steps: usize,
    pub eval_spikes: usize,
    pub eval_steps: usize,
}

impl CircuitSubstrate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: ExperimentConfig,
        nodes: Vec<CircuitNode>,
        in_neighbors: Vec<Vec<(usize, f64)>>,
        in_d_idx: usize,
        in_we_idx: usize,
        in_s_idx: usize,
        in_r_idx: usize,
        in_re_idx: usize,
        gate_set_idx: usize,
        gate_rst_idx: usize,
        v_inh_idx: usize,
        r0_idx: usize,
        r1_idx: usize,
        r2_idx: usize,
        r3_idx: usize,
        v_sense_idx: usize,
        v_read_idx: usize,
    ) -> Self {
        let n = nodes.len();
        let prev_spikes = vec![0.0; n];
        let rng = FastRng::seed_from_u64(config.seed.wrapping_add(0xCA_2026_008A));

        Self {
            config,
            nodes,
            in_neighbors,
            prev_spikes,
            rng,
            in_d_idx,
            in_we_idx,
            in_s_idx,
            in_r_idx,
            in_re_idx,
            gate_set_idx,
            gate_rst_idx,
            v_inh_idx,
            r0_idx,
            r1_idx,
            r2_idx,
            r3_idx,
            v_sense_idx,
            v_read_idx,
            total_spikes: 0,
            total_steps: 0,
            eval_spikes: 0,
            eval_steps: 0,
        }
    }

    /// Advance simulation by one discrete clock tick.
    /// Inputs: `(in_d, in_we, in_s, in_r, in_re)`
    /// Returns `(v_sense_spike, v_read_spike)`.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, inputs: (f64, f64, f64, f64, f64), in_eval_window: bool) -> (f64, f64) {
        let n = self.nodes.len();
        let mut curr_spikes = vec![0.0; n];
        let leak = self.config.leak;
        let alpha_rho = self.config.alpha_rho;

        for i in 0..n {
            let mut in_synaptic = 0.0;

            // Sensory ingress inputs
            if i == self.in_d_idx && inputs.0 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.0;
            }
            if i == self.in_we_idx && inputs.1 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.1;
            }
            if i == self.in_s_idx && inputs.2 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.2;
            }
            if i == self.in_r_idx && inputs.3 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.3;
            }
            if i == self.in_re_idx && inputs.4 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.4;
            }

            // Presynaptic spikes from graph edges
            for &(src, w) in &self.in_neighbors[i] {
                if self.prev_spikes[src] > 0.0 {
                    in_synaptic += w * self.prev_spikes[src];
                }
            }

            // Channel noise pulse: solitary subthreshold thermal fluctuation on ring nodes
            let mut noise_amp = 0.0;
            if self.config.noise_rate > 0.0
                && (i == self.r0_idx || i == self.r1_idx || i == self.r2_idx || i == self.r3_idx)
                && self.rng.bernoulli(self.config.noise_rate)
            {
                noise_amp = self.rng.normal(0.923, 0.0274).max(0.0);
            }

            // Somatic integration and Heaviside thresholding
            let node = &mut self.nodes[i];
            let spike: f64;

            if node.refractory_counter > 0 {
                spike = 0.0;
                node.v = 0.0;
                node.refractory_counter -= 1;
            } else {
                let v_cand = (1.0 - leak) * node.v + in_synaptic + noise_amp;
                if v_cand >= node.theta {
                    spike = 1.0;
                    node.v = 0.0;
                    node.refractory_counter = node.n_ref;
                } else {
                    spike = 0.0;
                    node.v = ((1.0 - leak) * node.v + in_synaptic).clamp(-2.0, 4.0);
                    node.refractory_counter = 0;
                }
            }

            node.spike = spike;
            curr_spikes[i] = spike;
            if spike > 0.0 {
                self.total_spikes += 1;
                if in_eval_window {
                    self.eval_spikes += 1;
                }
            }

            // Rolling firing rate EMA
            node.rolling_rate = (1.0 - alpha_rho) * node.rolling_rate + alpha_rho * spike;

            // Somatic threshold adaptation:
            // Increments by homeostatic quantum on firing; relaxes toward resting baseline when silent
            if node.beta_theta > 0.0 {
                if spike > 0.0 {
                    node.theta = (node.theta + node.beta_theta * alpha_rho).min(node.theta_max);
                } else {
                    node.theta = (node.theta - node.beta_theta * 0.01).max(node.theta_floor);
                }
            }
        }

        self.prev_spikes = curr_spikes;
        self.total_steps += 1;
        if in_eval_window {
            self.eval_steps += 1;
        }

        (
            self.prev_spikes[self.v_sense_idx],
            self.prev_spikes[self.v_read_idx],
        )
    }

    /// Substrate rolling activity density across evaluation steps
    pub fn mean_firing_density(&self) -> f64 {
        if self.eval_steps == 0 || self.nodes.is_empty() {
            0.0
        } else {
            self.eval_spikes as f64 / (self.eval_steps * self.nodes.len()) as f64
        }
    }

    /// Reset node potentials, refractory states, and spikes
    pub fn reset_state(&mut self) {
        for node in &mut self.nodes {
            node.v = 0.0;
            node.refractory_counter = 0;
            node.spike = 0.0;
        }
        self.prev_spikes.fill(0.0);
    }
}
