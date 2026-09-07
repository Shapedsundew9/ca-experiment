//! Attractor detection, limit cycle analysis, and phase-space distance metrics.

pub const WINDOW_SIZE: usize = 128;
pub const NUM_NODES: usize = 16;

/// Detect periodic limit cycle in a sequence of binary spike masks
pub fn detect_period(snapshots: &[u16]) -> (usize, bool) {
    let len = snapshots.len();
    if len < 4 {
        return (0, false);
    }

    // Check candidate periods from 1 up to len / 2
    let max_p = (len / 2).min(WINDOW_SIZE);
    for p in 1..=max_p {
        let mut is_periodic = true;
        // Verify periodicity across last 2 * p entries
        let check_len = (2 * p).min(len);
        for k in 0..check_len {
            if snapshots[len - 1 - k] != snapshots[len - 1 - k - p] {
                is_periodic = false;
                break;
            }
        }
        if is_periodic {
            return (p, true);
        }
    }
    (max_p, false)
}

/// Compute node occupancy probability vector from trailing snapshots
pub fn compute_occupancy(snapshots: &[u16]) -> [f64; NUM_NODES] {
    let mut counts = [0usize; NUM_NODES];
    let n = snapshots.len();
    if n == 0 {
        return [0.0; NUM_NODES];
    }
    for &mask in snapshots {
        for (i, count) in counts.iter_mut().enumerate() {
            if (mask & (1 << i)) != 0 {
                *count += 1;
            }
        }
    }
    let mut occ = [0.0; NUM_NODES];
    for i in 0..NUM_NODES {
        occ[i] = (counts[i] as f64) / (n as f64);
    }
    occ
}

/// Compute L1 occupancy distance: D_occ = (1 / N) * ||m_A - m_B||_1
pub fn occupancy_distance(occ_a: &[f64; NUM_NODES], occ_b: &[f64; NUM_NODES]) -> f64 {
    let mut diff_sum = 0.0;
    for i in 0..NUM_NODES {
        diff_sum += (occ_a[i] - occ_b[i]).abs();
    }
    diff_sum / (NUM_NODES as f64)
}

/// Normalized Hamming distance between two binary state masks
#[inline]
pub fn hamming_distance(mask_a: u16, mask_b: u16) -> f64 {
    (mask_a ^ mask_b).count_ones() as f64 / (NUM_NODES as f64)
}

/// Phase-aligned orbit distance: min_phi (1 / L) sum h(s_A(t), s_B(t + phi))
pub fn orbit_distance(snaps_a: &[u16], snaps_b: &[u16], period_b: usize) -> f64 {
    let l = snaps_a.len().min(snaps_b.len());
    if l == 0 {
        return 0.0;
    }

    let p_b = period_b.max(1).min(l);
    let mut min_dist = f64::INFINITY;

    for phi in 0..p_b {
        let mut sum_dist = 0.0;
        let mut count = 0usize;
        for k in 0..(l - phi) {
            sum_dist += hamming_distance(snaps_a[k], snaps_b[k + phi]);
            count += 1;
        }
        if count > 0 {
            let avg = sum_dist / (count as f64);
            if avg < min_dist {
                min_dist = avg;
            }
        }
    }

    if min_dist.is_infinite() {
        0.0
    } else {
        min_dist
    }
}

/// Composite attractor distance: D(S_A, S_B) = 0.5 * (D_orbit + D_occ)
pub fn composite_distance(
    snaps_a: &[u16],
    snaps_b: &[u16],
    occ_a: &[f64; NUM_NODES],
    occ_b: &[f64; NUM_NODES],
    period_b: usize,
) -> f64 {
    let d_orbit = orbit_distance(snaps_a, snaps_b, period_b);
    let d_occ = occupancy_distance(occ_a, occ_b);
    0.5 * (d_orbit + d_occ)
}
