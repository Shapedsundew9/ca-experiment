//! Generic experiment execution harness, sweep scheduler, and telemetry serialization.
//!
//! Provides reusable primitives for:
//! - Buffered JSONL and CSV record streaming
//! - Thread pool parallelism configuration
//! - Factorial sweep orchestration across seeds and experimental conditions

use serde::Serialize;
use std::fs::{File, create_dir_all};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::thread;

/// Fast buffered JSONL file streamer.
pub struct JsonlWriter<W: Write> {
    writer: BufWriter<W>,
}

impl<W: Write> JsonlWriter<W> {
    /// Create a new buffered JSONL writer wrapping an existing writer.
    pub fn new(inner: W) -> Self {
        Self {
            writer: BufWriter::new(inner),
        }
    }

    /// Serialize a record as a single JSON line followed by a newline delimiter.
    pub fn write_record<T: Serialize>(&mut self, record: &T) -> std::io::Result<()> {
        serde_json::to_writer(&mut self.writer, record).map_err(std::io::Error::other)?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    /// Flush underlying buffered writer.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

/// Create a new JSONL file at the specified path, ensuring parent directories exist.
pub fn create_jsonl_writer(path: &Path) -> std::io::Result<JsonlWriter<File>> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    Ok(JsonlWriter::new(file))
}

/// Fast buffered CSV writer helper.
pub struct CsvWriter<W: Write> {
    writer: BufWriter<W>,
}

impl<W: Write> CsvWriter<W> {
    /// Create a new CSV writer and write the initial header line.
    pub fn new_with_header(inner: W, header: &str) -> std::io::Result<Self> {
        let mut writer = BufWriter::new(inner);
        writeln!(writer, "{header}")?;
        Ok(Self { writer })
    }

    /// Write a row formatted as a line.
    pub fn write_row(&mut self, row: &str) -> std::io::Result<()> {
        writeln!(self.writer, "{row}")
    }

    /// Flush underlying buffered writer.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

/// Create a new CSV file with the given header, creating parent directories if needed.
pub fn create_csv_writer(path: &Path, header: &str) -> std::io::Result<CsvWriter<File>> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    CsvWriter::new_with_header(file, header)
}

/// Query the recommended concurrency level based on hardware parallelism.
pub fn recommended_concurrency() -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Core trait defining single-trial execution logic for an experiment.
pub trait SubstrateExperimentRunner: Send + Sync {
    type Config: Clone + Send + Sync;
    type Input: Clone + Send + Sync;
    type TrialRecord: Serialize + Send;
    type RunTelemetry: Serialize + Send;

    /// Execute a single trial with input data and produce a structured trial record.
    fn execute_trial(
        &self,
        config: &Self::Config,
        input: &Self::Input,
        trial_id: usize,
    ) -> Self::TrialRecord;
}
