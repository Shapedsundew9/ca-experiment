//! Circuit graph builder for assembling excitable neural networks and automata.

use super::config::SubstrateConfig;
use super::network::NetworkSubstrate;
use super::node::CircuitNode;

#[derive(Debug, Clone)]
pub struct CircuitBuilder {
    pub config: SubstrateConfig,
    pub nodes: Vec<CircuitNode>,
    pub in_neighbors: Vec<Vec<(usize, f64)>>,
}

impl CircuitBuilder {
    pub fn new(config: SubstrateConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
            in_neighbors: Vec::new(),
        }
    }

    /// Add an excitable node to the graph and return its index.
    #[allow(clippy::too_many_arguments)]
    pub fn add_node(
        &mut self,
        name: &str,
        theta_init: f64,
        theta_floor: f64,
        theta_max: f64,
        beta_theta: f64,
        n_ref: usize,
        rho_target: f64,
        leak: f64,
    ) -> usize {
        let id = self.nodes.len();
        let node = CircuitNode::new(
            id,
            name.to_string(),
            theta_init,
            theta_floor,
            theta_max,
            beta_theta,
            n_ref,
            rho_target,
            leak,
        );
        self.nodes.push(node);
        self.in_neighbors.push(Vec::new());
        id
    }

    /// Add a directed synaptic edge with weight from node `from` to node `to`.
    pub fn add_edge(&mut self, from: usize, to: usize, weight: f64) {
        if to < self.in_neighbors.len() {
            self.in_neighbors[to].push((from, weight));
        }
    }

    /// Construct a linear transmission line of excitable nodes of given length.
    /// Returns a tuple of (ingress_node_id, egress_node_id).
    pub fn add_transmission_track(
        &mut self,
        prefix: &str,
        length: usize,
        theta: f64,
        n_ref: usize,
        weight: f64,
    ) -> (usize, usize) {
        assert!(length >= 1, "Transmission line length must be >= 1");
        let first = self.add_node(
            &format!("{prefix}_0"),
            theta,
            self.config.theta_floor,
            self.config.theta_max,
            self.config.beta_theta,
            n_ref,
            self.config.rho_target,
            self.config.leak,
        );

        let mut prev = first;
        for i in 1..length {
            let next = self.add_node(
                &format!("{prefix}_{i}"),
                theta,
                self.config.theta_floor,
                self.config.theta_max,
                self.config.beta_theta,
                n_ref,
                self.config.rho_target,
                self.config.leak,
            );
            self.add_edge(prev, next, weight);
            prev = next;
        }

        (first, prev)
    }

    /// Finalize and build into a `NetworkSubstrate`.
    pub fn build(self) -> NetworkSubstrate {
        NetworkSubstrate::new(self.config, self.nodes, self.in_neighbors)
    }
}
