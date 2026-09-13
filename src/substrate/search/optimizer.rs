//! Ask/tell optimizers over the normalised `[0,1]^n` cube.
//!
//! - [`RandomSearch`]: Latin-hypercube baseline. Any claim that a smarter optimizer helped
//!   must beat it.
//! - [`DifferentialEvolution`]: classic DE/rand/1/bin, the default. Handles bounds naturally
//!   and descends plateau-heavy objectives that stall simplex methods.

use super::super::rng::FastRng;

/// Population-style search interface. The driver evaluates each [`Optimizer::ask`] batch in
/// parallel and returns the scored candidates, in the same order, to [`Optimizer::tell`].
pub trait Optimizer {
    /// Propose a batch of normalised candidates.
    fn ask(&mut self) -> Vec<Vec<f64>>;

    /// Report losses for the most recent batch, in the order it was proposed.
    fn tell(&mut self, batch: &[(Vec<f64>, f64)]);

    /// Whether the optimizer has exhausted its own internal schedule.
    fn is_done(&self) -> bool;

    /// Short identifier recorded in the calibration provenance.
    fn name(&self) -> &str {
        "optimizer"
    }
}

/// Fold a coordinate back into `[0,1]` by reflection at the bounds.
fn reflect(mut v: f64) -> f64 {
    if !v.is_finite() {
        return 0.5;
    }
    for _ in 0..8 {
        if v < 0.0 {
            v = -v;
        } else if v > 1.0 {
            v = 2.0 - v;
        } else {
            break;
        }
    }
    v.clamp(0.0, 1.0)
}

/// Latin-hypercube random search: a stratified, non-adaptive baseline.
#[derive(Debug, Clone)]
pub struct RandomSearch {
    design: Vec<Vec<f64>>,
    batch_size: usize,
    cursor: usize,
}

impl RandomSearch {
    /// Draw a Latin-hypercube design of `samples` points, issued `batch_size` at a time.
    pub fn new(dim: usize, samples: usize, batch_size: usize, seed: u64) -> Self {
        let mut rng = FastRng::seed_from_u64(seed);
        let mut design = vec![vec![0.0; dim]; samples];
        for j in 0..dim {
            let mut strata: Vec<usize> = (0..samples).collect();
            for i in (1..samples).rev() {
                let k = (rng.next_u64() % (i as u64 + 1)) as usize;
                strata.swap(i, k);
            }
            for (i, row) in design.iter_mut().enumerate() {
                row[j] = (strata[i] as f64 + rng.next_f64()) / samples as f64;
            }
        }
        Self {
            design,
            batch_size: batch_size.max(1),
            cursor: 0,
        }
    }
}

impl Optimizer for RandomSearch {
    fn ask(&mut self) -> Vec<Vec<f64>> {
        let end = (self.cursor + self.batch_size).min(self.design.len());
        let batch = self.design[self.cursor..end].to_vec();
        self.cursor = end;
        batch
    }

    fn tell(&mut self, _batch: &[(Vec<f64>, f64)]) {}

    fn is_done(&self) -> bool {
        self.cursor >= self.design.len()
    }

    fn name(&self) -> &str {
        "random_search"
    }
}

/// Classic DE/rand/1/bin over the unit cube.
#[derive(Debug, Clone)]
pub struct DifferentialEvolution {
    dim: usize,
    pop_size: usize,
    f: f64,
    cr: f64,
    max_generations: usize,
    rng: FastRng,
    population: Vec<Vec<f64>>,
    fitness: Vec<f64>,
    generation: usize,
    initialised: bool,
}

impl DifferentialEvolution {
    /// Create a DE optimizer with differential weight `F = 0.8` and crossover `CR = 0.9`.
    /// Population size is raised to the four members DE/rand/1 requires.
    pub fn new(dim: usize, pop_size: usize, seed: u64) -> Self {
        Self {
            dim,
            pop_size: pop_size.max(4),
            f: 0.8,
            cr: 0.9,
            max_generations: 100,
            rng: FastRng::seed_from_u64(seed),
            population: Vec::new(),
            fitness: Vec::new(),
            generation: 0,
            initialised: false,
        }
    }

    /// Override the differential weight `F`.
    pub fn with_f(mut self, f: f64) -> Self {
        self.f = f;
        self
    }

    /// Override the crossover probability `CR`.
    pub fn with_cr(mut self, cr: f64) -> Self {
        self.cr = cr;
        self
    }

    /// Override the generation cap.
    pub fn with_max_generations(mut self, max_generations: usize) -> Self {
        self.max_generations = max_generations;
        self
    }

    /// Best member seen so far, if the population has been scored.
    pub fn best(&self) -> Option<(&[f64], f64)> {
        let (idx, loss) =
            self.fitness
                .iter()
                .enumerate()
                .fold(None, |acc: Option<(usize, f64)>, (i, l)| match acc {
                    Some((_, best)) if best <= *l => acc,
                    _ => Some((i, *l)),
                })?;
        Some((self.population[idx].as_slice(), loss))
    }

    /// Draw three distinct population indices, all different from `exclude`.
    fn pick_three(&mut self, exclude: usize) -> (usize, usize, usize) {
        let n = self.population.len();
        let mut picks = [usize::MAX; 3];
        for slot in 0..3 {
            loop {
                let candidate = (self.rng.next_u64() % n as u64) as usize;
                if candidate != exclude && !picks[..slot].contains(&candidate) {
                    picks[slot] = candidate;
                    break;
                }
            }
        }
        (picks[0], picks[1], picks[2])
    }
}

impl Optimizer for DifferentialEvolution {
    fn ask(&mut self) -> Vec<Vec<f64>> {
        if self.dim == 0 {
            return Vec::new();
        }
        if !self.initialised {
            return (0..self.pop_size)
                .map(|_| (0..self.dim).map(|_| self.rng.next_f64()).collect())
                .collect();
        }
        let mut trials = Vec::with_capacity(self.population.len());
        for i in 0..self.population.len() {
            let (a, b, c) = self.pick_three(i);
            let forced = (self.rng.next_u64() % self.dim as u64) as usize;
            let mut trial = self.population[i].clone();
            for (j, slot) in trial.iter_mut().enumerate() {
                if j == forced || self.rng.next_f64() < self.cr {
                    *slot = reflect(
                        self.population[a][j]
                            + self.f * (self.population[b][j] - self.population[c][j]),
                    );
                }
            }
            trials.push(trial);
        }
        trials
    }

    fn tell(&mut self, batch: &[(Vec<f64>, f64)]) {
        if batch.is_empty() {
            return;
        }
        if !self.initialised {
            self.population = batch.iter().map(|(x, _)| x.clone()).collect();
            self.fitness = batch.iter().map(|(_, loss)| *loss).collect();
            self.initialised = true;
            return;
        }
        // Non-strict acceptance lets the population drift across plateaus instead of freezing.
        for (i, (trial, loss)) in batch.iter().enumerate().take(self.population.len()) {
            if *loss <= self.fitness[i] {
                self.population[i].clone_from(trial);
                self.fitness[i] = *loss;
            }
        }
        self.generation += 1;
    }

    fn is_done(&self) -> bool {
        self.dim == 0 || (self.initialised && self.generation >= self.max_generations)
    }

    fn name(&self) -> &str {
        "differential_evolution"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive an optimizer to completion, returning the best candidate, its loss, and the
    /// number of objective evaluations consumed.
    fn drive<F>(opt: &mut dyn Optimizer, f: F) -> (Vec<f64>, f64, usize)
    where
        F: Fn(&[f64]) -> f64,
    {
        let mut best = (Vec::new(), f64::INFINITY);
        let mut evals = 0;
        while !opt.is_done() {
            let batch = opt.ask();
            if batch.is_empty() {
                break;
            }
            let scored: Vec<(Vec<f64>, f64)> = batch
                .into_iter()
                .map(|x| {
                    let l = f(&x);
                    (x, l)
                })
                .collect();
            evals += scored.len();
            for (x, loss) in &scored {
                if *loss < best.1 {
                    best = (x.clone(), *loss);
                }
            }
            opt.tell(&scored);
        }
        (best.0, best.1, evals)
    }

    /// Map a normalised coordinate onto the symmetric interval `[-span, span]`.
    fn scaled(x: &[f64], span: f64) -> Vec<f64> {
        x.iter().map(|u| (u - 0.5) * 2.0 * span).collect()
    }

    fn sphere(x: &[f64]) -> f64 {
        scaled(x, 5.0).iter().map(|v| v * v).sum()
    }

    fn rastrigin(x: &[f64]) -> f64 {
        let v = scaled(x, 5.12);
        10.0 * v.len() as f64
            + v.iter()
                .map(|z| z * z - 10.0 * (2.0 * std::f64::consts::PI * z).cos())
                .sum::<f64>()
    }

    /// Piecewise-constant staircase: gradient is zero almost everywhere, so simplex and
    /// gradient methods stall. The optimum plateau is `|x - 0.3| < 0.1` in every coordinate.
    fn plateau(x: &[f64]) -> f64 {
        x.iter().map(|u| ((u - 0.3).abs() * 10.0).floor()).sum()
    }

    #[test]
    fn reflection_folds_candidates_into_the_unit_cube() {
        assert!((reflect(-0.25) - 0.25).abs() < 1e-12);
        assert!((reflect(1.25) - 0.75).abs() < 1e-12);
        assert!((reflect(0.5) - 0.5).abs() < 1e-12);
        assert!((reflect(f64::NAN) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn random_search_emits_a_latin_hypercube_design() {
        let samples = 20;
        let mut opt = RandomSearch::new(3, samples, 5, 7);
        let (_, _, evals) = drive(&mut opt, sphere);
        assert_eq!(evals, samples);
        assert!(opt.is_done());

        // Each dimension must hit every stratum exactly once.
        let design = RandomSearch::new(3, samples, 5, 7).design;
        for j in 0..3 {
            let mut strata: Vec<usize> = design
                .iter()
                .map(|r| (r[j] * samples as f64) as usize)
                .collect();
            strata.sort_unstable();
            assert_eq!(strata, (0..samples).collect::<Vec<_>>());
        }
    }

    #[test]
    fn de_recovers_the_sphere_optimum() {
        let mut opt = DifferentialEvolution::new(5, 30, 0xC0FFEE).with_max_generations(400);
        let (best, loss, evals) = drive(&mut opt, sphere);
        assert!(loss < 1e-8, "sphere loss {loss} after {evals} evals");
        for u in &best {
            assert!((u - 0.5).abs() < 1e-4, "coordinate {u} off optimum");
        }
    }

    #[test]
    fn de_recovers_the_rastrigin_optimum() {
        let mut opt = DifferentialEvolution::new(3, 40, 0x5EED).with_max_generations(250);
        let (best, loss, evals) = drive(&mut opt, rastrigin);
        assert!(loss < 1e-4, "rastrigin loss {loss} after {evals} evals");
        for u in &best {
            assert!((u - 0.5).abs() < 1e-3, "coordinate {u} off optimum");
        }
    }

    #[test]
    fn de_descends_a_plateaued_step_function() {
        let dim = 4;
        let seed = 0xABCDEF;

        let initial_best = DifferentialEvolution::new(dim, 30, seed)
            .ask()
            .iter()
            .map(|x| plateau(x))
            .fold(f64::INFINITY, f64::min);
        assert!(
            initial_best > 0.0,
            "plateau test is vacuous if the initial population already solves it"
        );

        let mut opt = DifferentialEvolution::new(dim, 30, seed).with_max_generations(120);
        let (_, loss, evals) = drive(&mut opt, plateau);
        assert_eq!(loss, 0.0, "plateau loss {loss} after {evals} evals");
        assert!(loss < initial_best);
    }

    #[test]
    fn de_stops_at_the_generation_cap() {
        let mut opt = DifferentialEvolution::new(2, 8, 1).with_max_generations(10);
        let (_, _, evals) = drive(&mut opt, sphere);
        // One initialisation batch plus one batch per generation.
        assert_eq!(evals, 8 * 11);
    }
}
