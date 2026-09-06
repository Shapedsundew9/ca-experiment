# Experiment Run Manifest & Execution Audit Log Template

This template defines the standard provenance record generated whenever an experiment implementation specification is executed. It bridges the gap between pre-registered scientific protocols and empirical diagnostics by recording the exact environment, parameters actually executed, runtime deviations, and data checksums.

Save completed run logs to `docs/research/runs/RUN-EXP-YYYY-NNNa-[run-id].md`.

---

````markdown
# Experiment Run Manifest: [EXP-ID] (Run [N])

- **Run Identifier**: RUN-EXP-YYYY-NNNa-[run-id]
- **Protocol Reference**: `docs/research/protocols/EXP-YYYY-NNNa.md`
- **Hypothesis Reference**: `docs/research/hypotheses/HYP-YYYY-NNN.md`
- **Experiment Package**: `python/experiments/exp_yyyy_nnna_[slug]/`
- **Lead Execution Agent**: Sci: Execution & Analysis | Manual Operator CLI
- **Date & Duration**: YYYY-MM-DD HH:MM:SS UTC (Duration: [X]h [Y]m [Z]s)

---

## 1. System & Execution Environment

| Dimension | Specification |
|---|---|
| Git Commit SHA | `[40-character git commit hash]` |
| Git Tag | `exp/EXP-YYYY-NNNa-[run-id]` |
| Git Status Dirty? | No (Must be clean for official run) |
| Runtime / Compiler | Python 3.13.x / Rust 1.85+ |
| Hardware Profile | [e.g., 1x NVIDIA RTX 4090 (24GB VRAM), 64GB System RAM, 16 vCPU] |
| Operating System | Linux 6.x (x86_64) |
| Core Dependencies | [e.g., PyTorch 2.x, numpy 2.x, tokio 1.x] |

---

## 2. Command-Line Invocation & Entry Point

```bash
# Exact execution command line:
python -m scripts.run_sweep --protocol EXP-2025-014a --config configs/exp014a.json --seeds 0..29 --output-dir data/telemetry/EXP-2025-014a/
```

---

## 3. Parameter Space & Execution Truth (Planned vs. Actual)

Document any difference between the pre-registered protocol and what was physically executed.

| Parameter / Factor | Protocol Specification | Actual Executed Value | Status |
| --- | --- | --- | --- |
| Network Size $N$ | {64, 128, 256, 512} | {64, 128, 256, 512} | MATCH |
| Conservation $\lambda$ | {0.0, 0.5, 1.0} | {0.0, 0.5, 1.0} | MATCH |
| Batch Size / Steps | 1000 steps | 1000 steps | MATCH |

### Runtime Parameter Deviations & Overrides

- **Deviations**: [None, or describe explicit parameter modifications made during execution]
- **Technical Driver**: [e.g., Memory ceiling exceeded, process deadlock, numerical instability]
- **Approval / Justification**: [e.g., Approved by Operator at runtime gate]

---

## 4. Inner-Loop Parameter Discovery & Search Trajectory

Document how the inner loop explored the parameter space (e.g. adaptive bisection, probe points, homing in on phase transitions):

- **Exploration Strategy**: [e.g., Adaptive bisection around critical threshold λ_c, probe points followed by local density sweep]
- **Observations during Exploration**: [e.g., Attractor collapse observed for λ > 0.6; concentrated seed evaluations in [0.2, 0.5] interval]
- **Search Trajectory Summary**: [e.g., Evaluated 6 initial probes, identified stable band at [0.30, 0.45], executed full seed battery on optimal points]

---

## 5. Seed Inventory & Completion Matrix

| Seed Range | Scheduled | Completed | Aborted / Timed Out | Anomaly Notes |
| --- | --- | --- | --- | --- |
| Seeds 00–09 | 10 | 10 | 0 | Clean execution |
| Seeds 10–19 | 10 | 10 | 0 | Clean execution |
| Seeds 20–29 | 10 | 10 | 0 | Clean execution |

- **Total Completion Rate**: [30 / 30 (100%)]
- **Total Compute Consumed**: [14.2 GPU-hours / 180 compute-hours budget]

---

## 6. Telemetry Artifacts & Checksums

All emitted telemetry files and their cryptographic checksums:

| Artifact File | Size | Format | SHA-256 Checksum |
| --- | --- | --- | --- |
| `data/telemetry/EXP-014a/sweep_metrics.jsonl` | 42 MB | JSONL | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `data/telemetry/EXP-014a/state_trajectories.h5` | 1.8 GB | HDF5 | `a85c409198fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `data/telemetry/EXP-014a/summary_reduced.json` | 180 KB | JSON | `c15c409198fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

---

## 7. Telemetry Data Reduction Summary

Summary computed programmatically by reduction tool before diagnostic evaluation:

- Data Reduction Script: `python/scripts/reduce_telemetry.py`
- Pre-checks: Missing values = 0, NaNs = 0, Corrupted runs = 0
- Reduced Summary Path: `data/telemetry/EXP-014a/summary_reduced.json`

---

## 8. Execution Sign-Off

- [x] Run conforms to pre-registered protocol (or documented deviations above).
- [x] Clean working tree verified (`git status --porcelain` is empty, Git Status Dirty is No).
- [x] Git tag applied: `exp/EXP-YYYY-NNNa-[run-id]`.
- [x] Telemetry integrity validated and verified non-empty.
- [x] Telemetry reduced to summary format for diagnostic evaluation.
- **Status**: READY FOR DIAGNOSTIC EVALUATION
````
