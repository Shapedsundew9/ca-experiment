//! ca-experiment library

pub mod substrate;

pub mod experiments {
    pub mod exp_2026_001a_mva_homeostasis;
    pub mod exp_2026_002a_mva_attractor_mapping;
    pub mod exp_2026_003a_mva_temporal_xor;
    pub mod exp_2026_004a_mva_hebbian_plasticity;
    pub mod exp_2026_005a_mva_continual_learning;
    pub mod exp_2026_006a_mva_signal_transport;
    pub mod exp_2026_007a_mva_multi_gate_composition;
    pub mod exp_2026_008a_mva_bistable_latching;
    pub mod exp_2026_009a_mva_finite_state_automata;
}

pub use experiments::exp_2026_008a_mva_bistable_latching;
pub use experiments::exp_2026_009a_mva_finite_state_automata;
pub use substrate::prelude::*;
