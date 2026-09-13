//! Normalised encoding of mixed continuous, integer, and categorical parameter spaces.
//!
//! Optimizers operate exclusively on `[0,1]^n` vectors. All type-specific quantisation
//! lives in [`SearchSpace::decode`], so no optimizer needs to know about parameter kinds.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Domain of a single tunable parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamKind {
    /// Real-valued parameter over the closed interval `[lo, hi]`.
    Continuous { lo: f64, hi: f64 },
    /// Integer-valued parameter over the inclusive range `lo..=hi`.
    Integer { lo: i64, hi: i64 },
    /// Unordered choice from a fixed list of labels.
    Categorical(Vec<String>),
}

/// A named parameter together with its domain and optional raw-space starting value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSpec {
    pub name: String,
    pub kind: ParamKind,
    /// Raw-space hint used by [`SearchSpace::encode`] when a name is absent from the input.
    /// For [`ParamKind::Categorical`] this is the category index.
    pub init: Option<f64>,
}

impl ParamSpec {
    /// Declare a continuous parameter over `[lo, hi]`.
    pub fn continuous(name: &str, lo: f64, hi: f64) -> Self {
        Self {
            name: name.to_string(),
            kind: ParamKind::Continuous { lo, hi },
            init: None,
        }
    }

    /// Declare an integer parameter over `lo..=hi`.
    pub fn integer(name: &str, lo: i64, hi: i64) -> Self {
        Self {
            name: name.to_string(),
            kind: ParamKind::Integer { lo, hi },
            init: None,
        }
    }

    /// Declare a categorical parameter over a fixed label set.
    pub fn categorical(name: &str, labels: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            kind: ParamKind::Categorical(labels.iter().map(|s| s.to_string()).collect()),
            init: None,
        }
    }

    /// Attach a raw-space initial value.
    pub fn with_init(mut self, init: f64) -> Self {
        self.init = Some(init);
        self
    }
}

/// An ordered collection of [`ParamSpec`] defining the coordinates of a search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSpace {
    specs: Vec<ParamSpec>,
}

impl SearchSpace {
    /// Build a space from an ordered list of parameter specifications.
    pub fn new(specs: Vec<ParamSpec>) -> Self {
        Self { specs }
    }

    /// Number of search coordinates.
    pub fn dim(&self) -> usize {
        self.specs.len()
    }

    /// Borrow the underlying parameter specifications.
    pub fn specs(&self) -> &[ParamSpec] {
        &self.specs
    }

    /// Convert a JSON object of named raw values into a normalised `[0,1]^n` vector.
    ///
    /// Missing names fall back to [`ParamSpec::init`], then to the midpoint `0.5`.
    pub fn encode(&self, named: &Value) -> Vec<f64> {
        self.specs
            .iter()
            .map(|spec| {
                let value = named.get(&spec.name);
                encode_one(&spec.kind, value)
                    .or_else(|| spec.init.and_then(|raw| encode_raw(&spec.kind, raw)))
                    .unwrap_or(0.5)
            })
            .collect()
    }

    /// Convert a normalised vector into a JSON object of named raw values.
    ///
    /// Out-of-range and non-finite coordinates are clamped into `[0,1]`; integer and
    /// categorical quantisation happens here.
    pub fn decode(&self, x: &[f64]) -> Value {
        let mut map = Map::with_capacity(self.specs.len());
        for (i, spec) in self.specs.iter().enumerate() {
            let u = sanitise(x.get(i).copied().unwrap_or(0.5));
            map.insert(spec.name.clone(), decode_one(&spec.kind, u));
        }
        Value::Object(map)
    }
}

/// Clamp a normalised coordinate into `[0,1]`, mapping non-finite input to the midpoint.
fn sanitise(u: f64) -> f64 {
    if u.is_nan() { 0.5 } else { u.clamp(0.0, 1.0) }
}

fn decode_one(kind: &ParamKind, u: f64) -> Value {
    match kind {
        ParamKind::Continuous { lo, hi } => {
            let v = if hi <= lo { *lo } else { lo + u * (hi - lo) };
            Value::from(v)
        }
        ParamKind::Integer { lo, hi } => {
            if hi <= lo {
                return Value::from(*lo);
            }
            let span = (hi - lo + 1) as f64;
            let offset = (u * span).floor() as i64;
            Value::from(lo + offset.min(hi - lo))
        }
        ParamKind::Categorical(labels) => {
            if labels.is_empty() {
                return Value::Null;
            }
            let idx = ((u * labels.len() as f64).floor() as usize).min(labels.len() - 1);
            Value::from(labels[idx].clone())
        }
    }
}

fn encode_one(kind: &ParamKind, value: Option<&Value>) -> Option<f64> {
    let value = value?;
    match kind {
        ParamKind::Categorical(labels) => {
            if let Some(label) = value.as_str() {
                let idx = labels.iter().position(|l| l == label)?;
                Some(bucket_centre(idx, labels.len()))
            } else {
                encode_raw(kind, value.as_f64()?)
            }
        }
        _ => encode_raw(kind, value.as_f64()?),
    }
}

/// Normalise a raw-space scalar. Integer and categorical values map to their bucket centre
/// so that `encode(decode(x))` is a fixed point.
fn encode_raw(kind: &ParamKind, raw: f64) -> Option<f64> {
    if !raw.is_finite() {
        return None;
    }
    match kind {
        ParamKind::Continuous { lo, hi } => {
            if hi <= lo {
                Some(0.0)
            } else {
                Some(((raw - lo) / (hi - lo)).clamp(0.0, 1.0))
            }
        }
        ParamKind::Integer { lo, hi } => {
            if hi <= lo {
                return Some(0.0);
            }
            let v = (raw.round() as i64).clamp(*lo, *hi);
            Some(bucket_centre((v - lo) as usize, (hi - lo + 1) as usize))
        }
        ParamKind::Categorical(labels) => {
            if labels.is_empty() {
                return None;
            }
            let idx = (raw.round().max(0.0) as usize).min(labels.len() - 1);
            Some(bucket_centre(idx, labels.len()))
        }
    }
}

fn bucket_centre(index: usize, count: usize) -> f64 {
    (index as f64 + 0.5) / count as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> SearchSpace {
        SearchSpace::new(vec![
            ParamSpec::continuous("theta", -1.0, 1.0),
            ParamSpec::integer("n_ref", 0, 10),
            ParamSpec::categorical("mode", &["a", "b", "c"]),
        ])
    }

    #[test]
    fn dim_matches_spec_count() {
        assert_eq!(fixture().dim(), 3);
    }

    #[test]
    fn decode_quantises_endpoints() {
        let space = fixture();
        let lo = space.decode(&[0.0, 0.0, 0.0]);
        assert_eq!(lo["theta"], json!(-1.0));
        assert_eq!(lo["n_ref"], json!(0));
        assert_eq!(lo["mode"], json!("a"));

        let hi = space.decode(&[1.0, 1.0, 1.0]);
        assert_eq!(hi["theta"], json!(1.0));
        assert_eq!(hi["n_ref"], json!(10));
        assert_eq!(hi["mode"], json!("c"));
    }

    #[test]
    fn decode_clamps_out_of_range_input() {
        let space = fixture();
        let clamped = space.decode(&[-4.0, 7.5, f64::NAN]);
        assert_eq!(clamped["theta"], json!(-1.0));
        assert_eq!(clamped["n_ref"], json!(10));
        assert_eq!(clamped["mode"], json!("b"));
    }

    #[test]
    fn decode_partitions_integers_uniformly() {
        let space = fixture();
        // 11 integer buckets across [0,1]; the centre of bucket k is (k + 0.5) / 11.
        for k in 0..=10i64 {
            let u = (k as f64 + 0.5) / 11.0;
            assert_eq!(space.decode(&[0.5, u, 0.5])["n_ref"], json!(k));
        }
    }

    #[test]
    fn encode_decode_round_trips() {
        let space = fixture();
        let x = vec![0.25, (3.0 + 0.5) / 11.0, (1.0 + 0.5) / 3.0];
        let named = space.decode(&x);
        assert_eq!(named["n_ref"], json!(3));
        assert_eq!(named["mode"], json!("b"));
        let back = space.encode(&named);
        for (a, b) in x.iter().zip(back.iter()) {
            assert!((a - b).abs() < 1e-12, "{a} != {b}");
        }
        assert_eq!(space.decode(&back), named);
    }

    #[test]
    fn encode_falls_back_to_init_then_midpoint() {
        let space = SearchSpace::new(vec![
            ParamSpec::continuous("with_init", 0.0, 10.0).with_init(2.5),
            ParamSpec::continuous("bare", 0.0, 10.0),
        ]);
        let encoded = space.encode(&json!({}));
        assert!((encoded[0] - 0.25).abs() < 1e-12);
        assert!((encoded[1] - 0.5).abs() < 1e-12);
    }

    #[test]
    fn degenerate_bounds_collapse_to_lower_bound() {
        let space = SearchSpace::new(vec![
            ParamSpec::continuous("fixed", 3.0, 3.0),
            ParamSpec::integer("pinned", 7, 7),
        ]);
        let named = space.decode(&[0.9, 0.9]);
        assert_eq!(named["fixed"], json!(3.0));
        assert_eq!(named["pinned"], json!(7));
    }
}
