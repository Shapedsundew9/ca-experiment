//! Excitable node dynamics and network substrate for EXP-2026-009a.

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
    pub leak: f64,
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
        leak: f64,
    ) -> Self {
        Self {
            id,
            name,
            v: 0.0,
            leak,
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
    pub in_sigma0_idx: usize,
    pub in_sigma1_idx: usize,
    pub v_accept_idx: usize,
    /// Ring node indices for each ring: ring_nodes[ring_idx] = [r0, r1, r2, r3]
    pub ring_nodes: Vec<[usize; 4]>,
    pub is_ring_node: Vec<bool>,
    pub total_spikes: usize,
    pub total_steps: usize,
    pub eval_spikes: usize,
    pub eval_steps: usize,
}

impl CircuitSubstrate {
    pub fn new(
        config: ExperimentConfig,
        nodes: Vec<CircuitNode>,
        in_neighbors: Vec<Vec<(usize, f64)>>,
        in_sigma0_idx: usize,
        in_sigma1_idx: usize,
        v_accept_idx: usize,
        ring_nodes: Vec<[usize; 4]>,
    ) -> Self {
        let n = nodes.len();
        let prev_spikes = vec![0.0; n];
        let rng = FastRng::seed_from_u64(config.seed.wrapping_add(0xCA_2026_009A));

        let mut is_ring_node = vec![false; n];
        for ring in &ring_nodes {
            for &idx in ring {
                is_ring_node[idx] = true;
            }
        }

        Self {
            config,
            nodes,
            in_neighbors,
            prev_spikes,
            rng,
            in_sigma0_idx,
            in_sigma1_idx,
            v_accept_idx,
            ring_nodes,
            is_ring_node,
            total_spikes: 0,
            total_steps: 0,
            eval_spikes: 0,
            eval_steps: 0,
        }
    }

    /// Advance simulation by one discrete clock tick.
    /// Inputs: `(token_0, token_1)`
    /// Returns: `v_accept_spike`.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, inputs: (f64, f64), in_eval_window: bool) -> f64 {
        let n = self.nodes.len();
        let mut curr_spikes = vec![0.0; n];
        let alpha_rho = self.config.alpha_rho;

        for i in 0..n {
            let mut in_synaptic = 0.0;

            // Sensory ingress inputs
            if i == self.in_sigma0_idx && inputs.0 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.0;
            }
            if i == self.in_sigma1_idx && inputs.1 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.1;
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
                && self.is_ring_node[i]
                && self.rng.bernoulli(self.config.noise_rate)
            {
                noise_amp = self.rng.normal(0.923, 0.0274).max(0.0);
            }

            // Somatic integration and Heaviside thresholding
            let node = &mut self.nodes[i];
            let leak = node.leak;
            let spike: f64;

            if node.refractory_counter > 0 {
                spike = 0.0;
                node.v = 0.0;
                node.refractory_counter -= 1;
            } else {
                let eff_leak = if node.v < 0.0 { 0.50 } else { leak };
                let v_cand = (1.0 - eff_leak) * node.v + in_synaptic + noise_amp;
                if v_cand >= node.theta {
                    spike = 1.0;
                    node.v = 0.0;
                    node.refractory_counter = node.n_ref;
                } else {
                    spike = 0.0;
                    let v_next = (1.0 - eff_leak) * node.v + in_synaptic;
                    node.v = v_next.clamp(-2.0, 4.0);
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

        self.nodes[self.v_accept_idx].spike
    }

    /// Directly inject a spike into a specific node (e.g. for initial state seeding).
    pub fn force_spike(&mut self, node_idx: usize) {
        if node_idx < self.nodes.len() {
            self.prev_spikes[node_idx] = 1.0;
            self.nodes[node_idx].spike = 1.0;
            self.nodes[node_idx].refractory_counter = self.nodes[node_idx].n_ref;
        }
    }

    /// Returns the number of currently active rings in the substrate based on recent firing.
    pub fn active_ring_count(&self) -> usize {
        let mut count = 0;
        for ring in &self.ring_nodes {
            let active = ring.iter().any(|&idx| self.nodes[idx].spike > 0.0);
            if active {
                count += 1;
            }
        }
        count
    }

    /// Returns which ring is currently active (if exactly one), or None.
    pub fn active_ring_id(&self) -> Option<usize> {
        let mut active_id = None;
        for (r_idx, ring) in self.ring_nodes.iter().enumerate() {
            let active = ring.iter().any(|&idx| self.nodes[idx].spike > 0.0);
            if active {
                if active_id.is_some() {
                    return None; // More than one ring active
                }
                active_id = Some(r_idx);
            }
        }
        active_id
    }

    /// Calculate mean firing density over the evaluation window
    pub fn evaluation_firing_density(&self) -> f64 {
        if self.eval_steps == 0 || self.nodes.is_empty() {
            0.0
        } else {
            self.eval_spikes as f64 / (self.nodes.len() as f64 * self.eval_steps as f64)
        }
    }
}
