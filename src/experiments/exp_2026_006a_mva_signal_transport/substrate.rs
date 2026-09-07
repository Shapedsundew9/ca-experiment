//! Transport graph topology and Leaky Integrate-and-Fire node substrate for EXP-2026-006a.

use super::config::{Condition, ExperimentConfig};
use super::signals::FastRng;

#[derive(Debug, Clone)]
pub struct TransportTopology {
    pub distance: usize,
    pub jitter: usize,
    pub l_stem: usize,
    pub l_branch1: usize,
    pub l_branch2: usize,
    pub total_nodes: usize,
    pub fork_idx: usize,
    pub branch1_root_idx: usize,
    pub branch1_readout_idx: usize,
    pub branch2_root_idx: usize,
    pub branch2_readout_idx: usize,
    /// in_neighbors[i] = list of (presynaptic_node, forward_weight, backward_weight)
    pub in_neighbors: Vec<Vec<(usize, f64, f64)>>,
}

impl TransportTopology {
    pub fn new(distance: usize, jitter: usize) -> Self {
        let l_stem = distance / 2;
        let l_branch1 = distance - l_stem;
        let l_branch2 = l_branch1 + jitter;
        let total_nodes = l_stem + l_branch1 + l_branch2;

        let fork_idx = l_stem - 1;
        let branch1_root_idx = l_stem;
        let branch1_readout_idx = l_stem + l_branch1 - 1;
        let branch2_root_idx = l_stem + l_branch1;
        let branch2_readout_idx = total_nodes - 1;

        let mut in_neighbors = vec![Vec::new(); total_nodes];

        // Stem internal edges
        for l in 1..l_stem {
            in_neighbors[l].push((l - 1, 1.0, 0.0));
            in_neighbors[l - 1].push((l, 0.0, 1.0));
        }

        // Fork to Branch 1
        in_neighbors[branch1_root_idx].push((fork_idx, 1.0, 0.0));
        in_neighbors[fork_idx].push((branch1_root_idx, 0.0, 1.0));

        // Fork to Branch 2
        in_neighbors[branch2_root_idx].push((fork_idx, 1.0, 0.0));
        in_neighbors[fork_idx].push((branch2_root_idx, 0.0, 1.0));

        // Branch 1 internal edges
        for m in 1..l_branch1 {
            let curr = branch1_root_idx + m;
            let prev = curr - 1;
            in_neighbors[curr].push((prev, 1.0, 0.0));
            in_neighbors[prev].push((curr, 0.0, 1.0));
        }

        // Branch 2 internal edges
        for n in 1..l_branch2 {
            let curr = branch2_root_idx + n;
            let prev = curr - 1;
            in_neighbors[curr].push((prev, 1.0, 0.0));
            in_neighbors[prev].push((curr, 0.0, 1.0));
        }

        Self {
            distance,
            jitter,
            l_stem,
            l_branch1,
            l_branch2,
            total_nodes,
            fork_idx,
            branch1_root_idx,
            branch1_readout_idx,
            branch2_root_idx,
            branch2_readout_idx,
            in_neighbors,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransportNode {
    pub id: usize,
    pub v: f64,
    pub refractory_counter: usize,
    pub rolling_rate: f64,
    pub theta: f64,
    pub spike: f64,
}

impl TransportNode {
    pub fn new(id: usize, theta_init: f64, rho_target: f64) -> Self {
        Self {
            id,
            v: 0.0,
            refractory_counter: 0,
            rolling_rate: rho_target,
            theta: theta_init,
            spike: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransportSubstrate {
    pub config: ExperimentConfig,
    pub topology: TransportTopology,
    pub nodes: Vec<TransportNode>,
    pub prev_spikes: Vec<f64>,
    pub prev_u: f64,
    pub rng: FastRng,
    pub total_spikes: usize,
    pub total_steps: usize,
    pub eval_spikes: usize,
    pub eval_steps: usize,
}

impl TransportSubstrate {
    pub fn new(config: ExperimentConfig) -> Self {
        let topology = TransportTopology::new(config.distance, config.jitter);
        let theta_init = if config.condition == Condition::ActiveRegenerative {
            1.05
        } else {
            config.theta_init
        };

        let nodes = (0..topology.total_nodes)
            .map(|i| TransportNode::new(i, theta_init, config.rho_target))
            .collect();
        let prev_spikes = vec![0.0; topology.total_nodes];
        let rng = FastRng::seed_from_u64(config.seed.wrapping_add(0x5A5A_2026));

        Self {
            config,
            topology,
            nodes,
            prev_spikes,
            prev_u: 0.0,
            rng,
            total_spikes: 0,
            total_steps: 0,
            eval_spikes: 0,
            eval_steps: 0,
        }
    }

    /// Advance one synchronous clock tick.
    /// Returns the firing output `(readout_1_spike, readout_2_spike)`.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, u_input: f64, perturb_b1: f64, in_eval_window: bool) -> (f64, f64) {
        let n = self.topology.total_nodes;
        let mut curr_spikes = vec![0.0; n];
        let cond = self.config.condition;
        let leak = self.config.leak;

        // Channel noise: with probability noise_rate per clock tick, a solitary noise pulse hits a channel node
        let noise_node =
            if self.config.noise_rate > 0.0 && self.rng.bernoulli(self.config.noise_rate) {
                Some((self.rng.next_u64() as usize) % n)
            } else {
                None
            };

        if cond == Condition::BaselinePassive {
            // Passive continuous leaky cable without action potentials
            let mut next_v = vec![0.0; n];
            for i in 0..n {
                let mut in_analog = 0.0;
                if i == 0 {
                    in_analog = self.prev_u;
                } else {
                    for &(j, w_fwd, _) in &self.topology.in_neighbors[i] {
                        if w_fwd > 0.0 {
                            in_analog += (1.0 - leak) * self.nodes[j].v * w_fwd;
                        }
                    }
                }

                if noise_node == Some(i) {
                    in_analog += 1.0;
                }

                next_v[i] = in_analog;

                // Readout threshold detector
                if i == self.topology.branch1_readout_idx || i == self.topology.branch2_readout_idx
                {
                    curr_spikes[i] = if next_v[i] >= self.nodes[i].theta {
                        1.0
                    } else {
                        0.0
                    };
                } else {
                    curr_spikes[i] = 0.0;
                }

                self.nodes[i].v = next_v[i];
                self.nodes[i].spike = curr_spikes[i];
            }
        } else {
            // Discrete excitable node dynamics
            let n_ref = self.config.n_ref;
            let alpha_rho = self.config.alpha_rho;
            let rho_target = self.config.rho_target;
            let beta_theta = self.config.beta_theta;
            let theta_max = self.config.theta_max;
            let theta_floor = if cond == Condition::ActiveRegenerative {
                1.02
            } else {
                self.config.theta_min
            };

            for i in 0..n {
                let mut in_charge = 0.0;

                // Primary sensory ingress at node 0
                if i == 0 {
                    let w_ingress = if cond == Condition::ActiveRegenerative {
                        self.nodes[0].theta.max(1.10)
                    } else {
                        1.0
                    };
                    in_charge += w_ingress * self.prev_u;
                }

                // Presynaptic lateral & branch connections
                for &(j, w_fwd, w_back) in &self.topology.in_neighbors[i] {
                    if w_fwd > 0.0 {
                        let w = if cond == Condition::ActiveRegenerative {
                            self.nodes[i].theta.max(1.10)
                        } else {
                            1.0
                        };
                        in_charge += w * self.prev_spikes[j];
                    } else if w_back > 0.0 && cond == Condition::AblationUnshielded {
                        // Retrograde coupling in unshielded ablation
                        in_charge += 1.0 * self.prev_spikes[j];
                    }
                }

                // Unilateral perturbation test on Branch 1 root
                if i == self.topology.branch1_root_idx && perturb_b1 > 0.0 {
                    let w_pert = if cond == Condition::ActiveRegenerative {
                        self.nodes[i].theta.max(1.10)
                    } else {
                        1.0
                    };
                    in_charge += w_pert * perturb_b1;
                }

                // Injected channel noise
                if noise_node == Some(i) {
                    in_charge += 1.0;
                }

                // Somatic integration & Heaviside thresholding
                let node = &mut self.nodes[i];
                let spike: f64;

                if node.refractory_counter > 0 {
                    spike = 0.0;
                    node.v = 0.0;
                    node.refractory_counter -= 1;
                } else {
                    let v_cand = (1.0 - leak) * node.v + in_charge;
                    if v_cand >= node.theta {
                        spike = 1.0;
                        node.v = 0.0;
                        node.refractory_counter = n_ref;
                    } else {
                        spike = 0.0;
                        node.v = v_cand;
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

                // Homeostatic rolling rate EMA
                node.rolling_rate = (1.0 - alpha_rho) * node.rolling_rate + alpha_rho * spike;

                // Homeostatic somatic threshold adaptation
                if beta_theta > 0.0 {
                    let delta = beta_theta * (node.rolling_rate - rho_target);
                    node.theta = (node.theta + delta).clamp(theta_floor, theta_max);
                }
            }
        }

        self.prev_spikes = curr_spikes;
        self.prev_u = u_input;
        self.total_steps += 1;
        if in_eval_window {
            self.eval_steps += 1;
        }

        (
            self.prev_spikes[self.topology.branch1_readout_idx],
            self.prev_spikes[self.topology.branch2_readout_idx],
        )
    }

    /// Evaluated substrate activity density across nodes during active transmission
    pub fn mean_firing_density(&self) -> f64 {
        if self.eval_steps == 0 || self.topology.total_nodes == 0 {
            0.0
        } else {
            self.eval_spikes as f64 / (self.eval_steps * self.topology.total_nodes) as f64
        }
    }
}
