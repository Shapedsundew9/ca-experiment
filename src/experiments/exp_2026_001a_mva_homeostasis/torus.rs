//! 4x4 Torus Network with von Neumann 4-neighbor connectivity, periodic boundaries,
//! sensory ingress, and synchronous evolution.

use super::homeostasis::AdaptationMode;
use super::node::MicroNode;

pub const LATTICE_WIDTH: usize = 4;
pub const LATTICE_HEIGHT: usize = 4;
pub const NUM_NODES: usize = LATTICE_WIDTH * LATTICE_HEIGHT; // 16
pub const DRIVE_TICKS: usize = 32;
pub const BURN_IN_TICKS: usize = 10_000;
pub const EXTINCTION_WINDOW: usize = 100;

#[derive(Debug, Clone)]
pub struct TorusConfig {
    pub mode: AdaptationMode,
    pub ticks: usize,
    pub seed: u64,
    pub leak: f64,
    pub n_ref: usize,
    pub target_rate: f64,
    pub eta: f64,
    pub window_size: usize,
    pub v_min: f64,
    pub v_max: f64,
    pub v_init: f64,
    pub carrier: bool,
}

impl Default for TorusConfig {
    fn default() -> Self {
        Self {
            mode: AdaptationMode::Active,
            ticks: 100_000,
            seed: 1,
            leak: 0.05,
            n_ref: 2,
            target_rate: 0.12,
            eta: 0.02,
            window_size: 100,
            v_min: 0.5,
            v_max: 8.0,
            v_init: 2.0,
            carrier: true,
        }
    }
}

/// Fast, deterministic PRNG based on Xoshiro256++
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
        Self {
            s: [next_sm(), next_sm(), next_sm(), next_sm()],
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let result = (self.s[0].wrapping_add(self.s[3]))
            .rotate_left(23)
            .wrapping_add(self.s[0]);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    #[inline]
    pub fn bernoulli(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }

    #[inline]
    pub fn coin_flip(&mut self) -> f64 {
        if (self.next_u64() & 1) == 0 {
            1.0
        } else {
            -1.0
        }
    }
}

pub struct TorusNetwork {
    pub config: TorusConfig,
    pub nodes: Vec<MicroNode>,
    /// Precomputed 4 neighbors for each node in order [North, South, East, West]
    pub neighbors: Vec<[usize; 4]>,
    pub prev_spikes: [u8; NUM_NODES],
    pub rng: FastRng,
}

impl TorusNetwork {
    pub fn new(config: TorusConfig) -> Self {
        let mut neighbors = Vec::with_capacity(NUM_NODES);
        for y in 0..LATTICE_HEIGHT {
            for x in 0..LATTICE_WIDTH {
                let north = ((y + 3) % LATTICE_HEIGHT) * LATTICE_WIDTH + x;
                let south = ((y + 1) % LATTICE_HEIGHT) * LATTICE_WIDTH + x;
                let east = y * LATTICE_WIDTH + ((x + 1) % LATTICE_WIDTH);
                let west = y * LATTICE_WIDTH + ((x + 3) % LATTICE_WIDTH);
                neighbors.push([north, south, east, west]);
            }
        }

        let nodes = (0..NUM_NODES)
            .map(|i| MicroNode::new(i, config.v_init, config.window_size))
            .collect();

        let rng = FastRng::seed_from_u64(config.seed);

        Self {
            config,
            nodes,
            neighbors,
            prev_spikes: [0; NUM_NODES],
            rng,
        }
    }

    /// Execute one discrete clock tick across the network
    #[inline]
    pub fn step(&mut self, tick: usize) -> (f64, f64) {
        // Compute incoming charges from previous tick spikes
        let mut incoming_charges = [0.0; NUM_NODES];
        for (i, charge) in incoming_charges.iter_mut().enumerate() {
            let mut sum_tracks = 0.0;
            for &nbr in &self.neighbors[i] {
                sum_tracks += self.prev_spikes[nbr] as f64;
            }

            // Sensory ingress on nodes (0, 0) -> idx 0 and (2, 2) -> idx 10
            let mut ext = 0.0;
            if tick < DRIVE_TICKS && (i == 0 || i == 10) {
                if self.rng.bernoulli(0.25) {
                    ext = 1.0;
                }
            } else if self.config.carrier && (i == 0 || i == 10) && tick.is_multiple_of(4) {
                // Neutral carrier pattern (periodic clock on sensory tracks)
                ext = 1.0;
            }

            *charge = sum_tracks + ext;
        }

        let mut current_spikes = [0u8; NUM_NODES];
        let mut total_spikes = 0usize;

        for i in 0..NUM_NODES {
            let coin = if self.config.mode == AdaptationMode::RandomDrift {
                self.rng.coin_flip()
            } else {
                0.0
            };

            self.nodes[i].step(
                tick,
                incoming_charges[i],
                self.config.leak,
                self.config.n_ref,
                self.config.mode,
                self.config.target_rate,
                self.config.eta,
                self.config.v_min,
                self.config.v_max,
                coin,
            );

            current_spikes[i] = self.nodes[i].spike;
            total_spikes += current_spikes[i] as usize;
        }

        // Branching ratio: \sigma(t) = \sum s_i(t) / \sum s_i(t-1)
        let prev_total_spikes: usize = self.prev_spikes.iter().map(|&s| s as usize).sum();
        let branching_ratio = if prev_total_spikes > 0 {
            (total_spikes as f64) / (prev_total_spikes as f64)
        } else {
            1.0
        };

        self.prev_spikes = current_spikes;
        let instant_density = (total_spikes as f64) / (NUM_NODES as f64);

        (instant_density, branching_ratio)
    }
}
