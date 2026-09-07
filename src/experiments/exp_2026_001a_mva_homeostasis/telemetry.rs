//! Telemetry emission and data reduction pipeline for EXP-2026-001a.

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub run_id: String,
    pub mode: String,
    pub seed: u64,
    pub ticks: usize,
    pub leak: f64,
    pub n_ref: usize,
    pub r_target: f64,
    pub eta: f64,
    pub w_h: usize,
    pub v_min: f64,
    pub v_max: f64,
    pub survived: bool,
    pub survival_ticks: usize,
    pub mean_firing_density: f64,
    pub std_firing_density: f64,
    pub rmse_target: f64,
    pub saturation_ticks: usize,
    pub extinction_ticks: usize,
    pub mean_final_v_thresh: f64,
    pub branching_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesRecord {
    pub run_id: String,
    pub tick: usize,
    pub instant_density: f64,
    pub rolling_density: f64,
    pub mean_v_thresh: f64,
    pub min_v_thresh: f64,
    pub max_v_thresh: f64,
    pub branching_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReductionMetrics {
    pub total_runs: usize,
    pub active_runs: usize,
    pub fixed_runs: usize,
    pub random_drift_runs: usize,
    pub inverted_runs: usize,

    // P1: Active extinction probability for nominal leaks {0.05, 0.10}
    pub p1_p_ext_active: f64,
    pub p1_verdict: bool,

    // P2: Saturation episode count
    pub p2_saturation_breaches: usize,
    pub p2_verdict: bool,

    // P3: Firing confinement
    pub p3_mean_firing_density: f64,
    pub p3_rmse_target_mean: f64,
    pub p3_verdict: bool,

    // P4: Baseline separation (fixed extinction >= 0.50)
    pub p4_p_ext_fixed: f64,
    pub p4_verdict: bool,

    // P5: Critical branching ratio
    pub p5_mean_branching_ratio: f64,
    pub p5_verdict: bool,

    // Statistical tests
    pub fishers_exact_p_value: f64,
    pub ks_statistic: f64,
    pub ks_p_value: f64,

    // Bootstrap 95% CIs
    pub active_density_ci_low: f64,
    pub active_density_ci_high: f64,
    pub branching_ratio_ci_low: f64,
    pub branching_ratio_ci_high: f64,
}

pub struct TelemetryWriter {
    summary_writer: BufWriter<File>,
    timeseries_writer: Option<BufWriter<File>>,
}

impl TelemetryWriter {
    pub fn new<P: AsRef<Path>>(out_dir: P, emit_timeseries: bool) -> std::io::Result<Self> {
        create_dir_all(&out_dir)?;
        let summary_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(out_dir.as_ref().join("summary.ndjson"))?;
        let summary_writer = BufWriter::new(summary_file);

        let timeseries_writer = if emit_timeseries {
            let ts_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(out_dir.as_ref().join("timeseries.ndjson"))?;
            Some(BufWriter::new(ts_file))
        } else {
            None
        };

        Ok(Self {
            summary_writer,
            timeseries_writer,
        })
    }

    pub fn write_summary(&mut self, record: &RunSummary) -> std::io::Result<()> {
        serde_json::to_writer(&mut self.summary_writer, record)?;
        self.summary_writer.write_all(b"\n")?;
        self.summary_writer.flush()?;
        Ok(())
    }

    pub fn write_timeseries(&mut self, record: &TimeSeriesRecord) -> std::io::Result<()> {
        if let Some(ref mut writer) = self.timeseries_writer {
            serde_json::to_writer(&mut *writer, record)?;
            writer.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn flush_all(&mut self) -> std::io::Result<()> {
        self.summary_writer.flush()?;
        if let Some(ref mut writer) = self.timeseries_writer {
            writer.flush()?;
        }
        Ok(())
    }
}
