//! Regularized Ridge regression linear classifier solver for EXP-2026-003a.

/// Gaussian elimination with partial pivoting to solve Ax = b
#[allow(clippy::needless_range_loop)]
pub fn solve_linear_system(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let d = b.len();
    for col in 0..d {
        let mut max_row = col;
        let mut max_val = a[col][col].abs();
        for row in (col + 1)..d {
            let val = a[row][col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return None;
        }
        if max_row != col {
            a.swap(col, max_row);
            b.swap(col, max_row);
        }

        let pivot = a[col][col];
        for row in (col + 1)..d {
            let factor = a[row][col] / pivot;
            for c in col..d {
                let val = a[col][c];
                a[row][c] -= factor * val;
            }
            b[row] -= factor * b[col];
        }
    }

    let mut x = vec![0.0; d];
    for col in (0..d).rev() {
        let mut sum = b[col];
        for c in (col + 1)..d {
            sum -= a[col][c] * x[c];
        }
        x[col] = sum / a[col][col];
    }
    Some(x)
}

#[derive(Debug, Clone)]
pub struct OnlineRidgeAccumulator {
    pub dim: usize,
    pub xtx: Vec<Vec<f64>>,
    pub xty: Vec<f64>,
    pub count: usize,
}

impl OnlineRidgeAccumulator {
    pub fn new(feature_dim: usize) -> Self {
        let dim = feature_dim + 1; // plus bias
        Self {
            dim,
            xtx: vec![vec![0.0; dim]; dim],
            xty: vec![0.0; dim],
            count: 0,
        }
    }

    #[inline]
    pub fn update(&mut self, features: &[f64], target: f64) {
        debug_assert_eq!(features.len() + 1, self.dim);
        self.count += 1;
        let d = self.dim;

        // features with 1.0 appended as bias
        for i in 0..d {
            let xi = if i < features.len() { features[i] } else { 1.0 };
            for j in 0..d {
                let xj = if j < features.len() { features[j] } else { 1.0 };
                self.xtx[i][j] += xi * xj;
            }
            self.xty[i] += xi * target;
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn solve(&self, ridge_alpha: f64) -> Option<RidgeModel> {
        let mut a = self.xtx.clone();
        let b = self.xty.clone();

        // Regularize weights, leave bias unregularized
        for i in 0..(self.dim - 1) {
            a[i][i] += ridge_alpha;
        }

        solve_linear_system(a, b).map(|weights| {
            let bias = weights[self.dim - 1];
            let w = weights[..(self.dim - 1)].to_vec();
            RidgeModel { weights: w, bias }
        })
    }
}

#[derive(Debug, Clone)]
pub struct RidgeModel {
    pub weights: Vec<f64>,
    pub bias: f64,
}

impl RidgeModel {
    #[inline]
    pub fn predict_activation(&self, features: &[f64]) -> f64 {
        let mut act = self.bias;
        for (w, &f) in self.weights.iter().zip(features.iter()) {
            act += w * f;
        }
        act
    }

    #[inline]
    pub fn predict_binary(&self, features: &[f64]) -> f64 {
        if self.predict_activation(features) >= 0.5 {
            1.0
        } else {
            0.0
        }
    }

    pub fn l2_norm(&self) -> f64 {
        self.weights.iter().map(|w| w * w).sum::<f64>().sqrt()
    }
}
