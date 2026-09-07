use rust_3::experiments::exp_2026_001a_mva_homeostasis::homeostasis::AdaptationMode;
use rust_3::experiments::exp_2026_001a_mva_homeostasis::node::MicroNode;
use rust_3::experiments::exp_2026_001a_mva_homeostasis::torus::{
    NUM_NODES, TorusConfig, TorusNetwork,
};

#[test]
fn test_torus_regularity_and_degree() {
    let config = TorusConfig::default();
    let net = TorusNetwork::new(config);

    assert_eq!(net.neighbors.len(), NUM_NODES);
    let mut in_degrees = [0usize; NUM_NODES];

    for (i, nbrs) in net.neighbors.iter().enumerate() {
        // In-degree is exactly 4
        assert_eq!(nbrs.len(), 4);
        // All neighbors must be distinct
        let mut sorted = nbrs.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 4, "Node {i} has duplicate neighbors");

        // Count out-degree to each neighbor
        for &nbr in nbrs {
            in_degrees[nbr] += 1;
        }
    }

    // Conservation of track degree: each node has exactly 4 in-degree
    for (i, &deg) in in_degrees.iter().enumerate() {
        assert_eq!(deg, 4, "Node {i} in-degree violated: {deg}");
    }
}

#[test]
fn test_refractory_lockout_invariant() {
    let mut node = MicroNode::new(0, 2.0, 100);
    // Inject suprathreshold charge
    node.step(
        0,
        3.0,
        0.0,
        2,
        AdaptationMode::Fixed,
        0.12,
        0.02,
        0.5,
        8.0,
        0.0,
    );
    assert_eq!(node.spike, 1, "Node must fire on suprathreshold charge");
    assert_eq!(node.refractory_counter, 2);

    // During lockout (tick 1), node cannot fire even with massive input
    node.step(
        1,
        10.0,
        0.0,
        2,
        AdaptationMode::Fixed,
        0.12,
        0.02,
        0.5,
        8.0,
        0.0,
    );
    assert_eq!(
        node.spike, 0,
        "Node must not fire during refractory lockout (1)"
    );
    assert_eq!(node.v, 0.0, "Potential must remain zero during lockout");
    assert_eq!(node.refractory_counter, 1);

    // During lockout (tick 2), node cannot fire
    node.step(
        2,
        10.0,
        0.0,
        2,
        AdaptationMode::Fixed,
        0.12,
        0.02,
        0.5,
        8.0,
        0.0,
    );
    assert_eq!(
        node.spike, 0,
        "Node must not fire during refractory lockout (2)"
    );
    assert_eq!(node.refractory_counter, 0);

    // Tick 3: receptive again
    node.step(
        3,
        3.0,
        0.0,
        2,
        AdaptationMode::Fixed,
        0.12,
        0.02,
        0.5,
        8.0,
        0.0,
    );
    assert_eq!(
        node.spike, 1,
        "Node must fire once refractory lockout expires"
    );
}

#[test]
fn test_threshold_compactness_invariant() {
    let mut node = MicroNode::new(0, 2.0, 100);
    // Force repeated downward adaptation
    for tick in 0..500 {
        node.step(
            tick,
            0.0,
            0.0,
            1,
            AdaptationMode::Active,
            0.12,
            0.05,
            0.5,
            8.0,
            0.0,
        );
    }
    assert!(
        node.v_thresh >= 0.5,
        "Threshold must not drop below v_min: {}",
        node.v_thresh
    );

    // Force repeated upward adaptation (inverted mode under hypoactivity)
    for tick in 0..500 {
        node.step(
            tick,
            0.0,
            0.0,
            1,
            AdaptationMode::Inverted,
            0.12,
            0.05,
            0.5,
            8.0,
            0.0,
        );
    }
    assert!(
        node.v_thresh <= 8.0,
        "Threshold must not exceed v_max: {}",
        node.v_thresh
    );
}

#[test]
fn test_active_homeostasis_survival_short() {
    let mut config = TorusConfig::default();
    config.ticks = 2000;
    config.mode = AdaptationMode::Active;
    config.leak = 0.05;
    config.seed = 42;
    config.v_init = 1.0;

    let mut net = TorusNetwork::new(config);
    let mut total_spikes = 0usize;
    for tick in 0..2000 {
        let (density, _) = net.step(tick);
        let spikes = (density * (NUM_NODES as f64)).round() as usize;
        total_spikes += spikes;
    }

    assert!(
        total_spikes > 100,
        "Network must sustain persistent activity: total_spikes={total_spikes}"
    );
}
