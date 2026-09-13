//! Statistical metrics and hypothesis evaluation routines for EXP-2026-009a.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub accuracy_seq: f64,
    pub fidelity_trans: f64,
    pub crosstalk_violation_rate: f64,
    pub ber_read: f64,
    pub mean_firing_density: f64,
    pub sequences_total: usize,
    pub sequences_correct: usize,
    pub transitions_total: usize,
    pub transitions_correct: usize,
    pub crosstalk_ticks: usize,
    pub settled_ticks: usize,
}

pub fn sample_mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

pub fn sample_std(data: &[f64], mean: f64) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
    variance.sqrt()
}

pub fn cohen_d(mean1: f64, std1: f64, mean2: f64, std2: f64) -> f64 {
    let pooled_var = (std1.powi(2) + std2.powi(2)) / 2.0;
    if pooled_var <= 1e-12 {
        if (mean1 - mean2).abs() < 1e-12 {
            0.0
        } else {
            (mean1 - mean2).abs() / 1e-6
        }
    } else {
        (mean1 - mean2).abs() / pooled_var.sqrt()
    }
}

/// Compute Welch's two-sample t-statistic and approximate two-tailed p-value.
pub fn welch_t_test(group1: &[f64], group2: &[f64]) -> (f64, f64, f64) {
    let n1 = group1.len() as f64;
    let n2 = group2.len() as f64;
    if n1 < 2.0 || n2 < 2.0 {
        return (0.0, 1.0, 0.0);
    }

    let m1 = sample_mean(group1);
    let m2 = sample_mean(group2);
    let s1 = sample_std(group1, m1);
    let s2 = sample_std(group2, m2);

    let d = cohen_d(m1, s1, m2, s2);

    let v1 = s1.powi(2) / n1;
    let v2 = s2.powi(2) / n2;
    let se = (v1 + v2).sqrt();

    if se < 1e-12 {
        if (m1 - m2).abs() < 1e-12 {
            return (0.0, 1.0, 0.0);
        } else {
            return (100.0, 0.0, d);
        }
    }

    let t = (m1 - m2) / se;

    // Welch-Satterthwaite degrees of freedom
    let df_num = (v1 + v2).powi(2);
    let df_den = (v1.powi(2) / (n1 - 1.0)) + (v2.powi(2) / (n2 - 1.0));
    let df = if df_den > 0.0 {
        df_num / df_den
    } else {
        n1 + n2 - 2.0
    };

    let p = student_t_two_tailed_p(t.abs(), df);
    (t, p, d)
}

/// Approximate p-value for Student's t distribution.
fn student_t_two_tailed_p(t: f64, df: f64) -> f64 {
    if df <= 0.0 || t.is_nan() {
        return 1.0;
    }
    // Hill's approximation / normal approximation for large df
    let x = df / (df + t * t);
    let a = df / 2.0;
    let b = 0.5;
    let ib = incomplete_beta(x, a, b);
    ib.clamp(0.0, 1.0)
}

/// Continued fraction approximation for regularized incomplete beta function I_x(a, b)
fn incomplete_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // ln(Beta(a, b)) = ln(Gamma(a)) + ln(Gamma(b)) - ln(Gamma(a + b))
    let lbeta = ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b);
    let front = (a * x.ln() + b * (1.0 - x).ln() - lbeta).exp() / a;

    // Continued fraction
    let max_iter = 100;
    let eps = 1e-10;

    let mut c = 1.0;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if d.abs() < 1e-30 {
        d = 1e-30;
    }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..=max_iter {
        let mf = m as f64;

        // One step of continued fraction
        let num_even = mf * (b - mf) * x / ((a + 2.0 * mf - 1.0) * (a + 2.0 * mf));
        d = 1.0 + num_even * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + num_even / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        h *= d * c;

        let num_odd = -(a + mf) * (a + b + mf) * x / ((a + 2.0 * mf) * (a + 2.0 * mf + 1.0));
        d = 1.0 + num_odd * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + num_odd / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;

        if (delta - 1.0).abs() < eps {
            break;
        }
    }

    front * h
}

/// Stirling approximation for log-gamma function
fn ln_gamma(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let coeffs = [
        76.18009172947146,
        -86.50532032941677,
        24.01409824083091,
        -1.231739572450155,
        0.001208650973866179,
        -0.000005395239384953,
    ];
    let mut y = x;
    let mut tmp = x + 5.5;
    tmp -= (x + 0.5) * tmp.ln();
    let mut ser = 1.000000000190015;
    for &c in &coeffs {
        y += 1.0;
        ser += c / y;
    }
    -tmp + (2.5066282746310005 * ser / x).ln()
}
