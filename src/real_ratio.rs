use crate::{
    p61_virtual_ratio_report_from_program, validate_file, AtlasProgram, AtlasResult, Diagnostic,
    DiagnosticCode, MemoryRuntime, RuntimeConfig, RuntimeWorkload, WorkloadMode, WorkloadRecord,
};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const COST_MODEL: &str = "measured_real_v1";
const MEASUREMENT_KIND: &str = "real_wall_clock_and_filesystem";
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P62Decision {
    ValidateRealMeasuredRatio,
    RecalibrateMeasurementModel,
    RecalibrateRuntimePaths,
    NoGoRealRatio,
}

impl P62Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            P62Decision::ValidateRealMeasuredRatio => "VALIDATE_P62_REAL_MEASURED_RATIO",
            P62Decision::RecalibrateMeasurementModel => "RECALIBRATE_P62_MEASUREMENT_MODEL",
            P62Decision::RecalibrateRuntimePaths => "RECALIBRATE_P62_RUNTIME_PATHS",
            P62Decision::NoGoRealRatio => "NO_GO_P62_REAL_RATIO",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct P62TimingStats {
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P62MachineInfo {
    pub os: String,
    pub arch: String,
    pub family: String,
    pub rust_version: Option<String>,
    pub cargo_version: Option<String>,
    pub cpu_info: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct P62PersistedBytes {
    pub snapshot_file_bytes: u64,
    pub manifest_file_bytes: u64,
    pub journal_file_bytes: u64,
    pub index_file_bytes: u64,
    pub payload_file_bytes: u64,
    pub audit_file_bytes: u64,
}

impl P62PersistedBytes {
    pub fn total(&self) -> u64 {
        self.snapshot_file_bytes
            + self.manifest_file_bytes
            + self.journal_file_bytes
            + self.index_file_bytes
            + self.payload_file_bytes
            + self.audit_file_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P62WorkloadSummary {
    pub id: String,
    pub kind: String,
    pub mechanism: String,
    pub virtual_declared: u128,
    pub virtual_effective: u128,
    pub accepted: bool,
    pub refused: bool,
    pub refusal_reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P62MeasurementSummary {
    pub total_persisted_bytes_min: u64,
    pub total_persisted_bytes_median: u64,
    pub total_persisted_bytes_max: u64,
    pub ratio_effective_per_byte_min: f64,
    pub ratio_effective_per_byte_median: f64,
    pub ratio_effective_per_byte_max: f64,
    pub read_p99_us_min: u64,
    pub read_p99_us_median: u64,
    pub read_p99_us_max: u64,
    pub update_p99_us_min: u64,
    pub update_p99_us_median: u64,
    pub update_p99_us_max: u64,
    pub snapshot_p99_us_min: u64,
    pub snapshot_p99_us_median: u64,
    pub snapshot_p99_us_max: u64,
    pub rebuild_p99_us_min: u64,
    pub rebuild_p99_us_median: u64,
    pub rebuild_p99_us_max: u64,
    pub audit_p99_us_min: u64,
    pub audit_p99_us_median: u64,
    pub audit_p99_us_max: u64,
    pub all_runs_passed: bool,
    pub run_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P62MeasuredRun {
    pub run_index: usize,
    pub run_id: String,
    pub persisted_bytes: P62PersistedBytes,
    pub total_persisted_bytes: u64,
    pub create_timing: P62TimingStats,
    pub read_timing: P62TimingStats,
    pub update_timing: P62TimingStats,
    pub delete_timing: P62TimingStats,
    pub snapshot_timing: P62TimingStats,
    pub rebuild_timing: P62TimingStats,
    pub audit_timing: P62TimingStats,
    pub virtual_effective: u128,
    pub ratio_effective_per_byte: f64,
    pub guard_refused: bool,
    pub dangerous_or_adversarial_refused: bool,
    pub audit_passed: bool,
    pub rebuild_passed: bool,
    pub snapshot_roundtrip_passed: bool,
    pub create_count: usize,
    pub read_count: usize,
    pub update_count: usize,
    pub delete_count: usize,
    pub snapshot_count: usize,
    pub rebuild_count: usize,
    pub audit_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P62RealRatioReport {
    pub astra_iteration: String,
    pub mode: String,
    pub program_path: String,
    pub cost_model: String,
    pub measurement_kind: String,
    pub repeat_count: usize,
    pub iteration_count: usize,
    pub warmup_count: usize,
    pub operation_count: usize,
    pub measurement_id: String,
    pub timestamp: Option<String>,
    pub machine_info: P62MachineInfo,
    pub commit_hash: Option<String>,
    pub summary: P62MeasurementSummary,
    pub create_timing: P62TimingStats,
    pub read_timing: P62TimingStats,
    pub update_timing: P62TimingStats,
    pub delete_timing: P62TimingStats,
    pub snapshot_timing: P62TimingStats,
    pub rebuild_timing: P62TimingStats,
    pub audit_timing: P62TimingStats,
    pub persisted_bytes: P62PersistedBytes,
    pub virtual_declared: u128,
    pub virtual_reachable: u128,
    pub virtual_readable: u128,
    pub virtual_updatable: u128,
    pub virtual_safe: u128,
    pub virtual_effective: u128,
    pub ratio_effective_per_byte: f64,
    pub ratio_effective_per_read_p99_us: f64,
    pub ratio_effective_per_update_p99_us: f64,
    pub ratio_effective_per_snapshot_p99_us: f64,
    pub create_count: usize,
    pub read_count: usize,
    pub update_count: usize,
    pub delete_count: usize,
    pub snapshot_count: usize,
    pub rebuild_count: usize,
    pub audit_count: usize,
    pub guard_refused: bool,
    pub dangerous_or_adversarial_refused: bool,
    pub audit_passed: bool,
    pub rebuild_passed: bool,
    pub snapshot_roundtrip_passed: bool,
    pub decision: P62Decision,
    pub decision_reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub runs: Vec<P62MeasuredRun>,
    pub workloads: Vec<P62WorkloadSummary>,
}

#[derive(Debug, Clone, Copy)]
struct P62MeasurementProfile {
    iteration_count: usize,
    warmup_count: usize,
    delete_count: usize,
}

impl P62MeasurementProfile {
    fn for_mode(mode: WorkloadMode) -> Self {
        match mode {
            WorkloadMode::Smoke => Self {
                iteration_count: 100,
                warmup_count: 10,
                delete_count: 10,
            },
            WorkloadMode::Standard => Self {
                iteration_count: 1_000,
                warmup_count: 50,
                delete_count: 100,
            },
            WorkloadMode::Ambitious => Self {
                iteration_count: 2_000,
                warmup_count: 100,
                delete_count: 200,
            },
        }
    }
}

pub fn p62_real_ratio_report_file(
    path: &str,
    mode: WorkloadMode,
) -> AtlasResult<P62RealRatioReport> {
    p62_real_ratio_report_file_with_runs(path, mode, 1)
}

pub fn p62_real_ratio_report_file_with_runs(
    path: &str,
    mode: WorkloadMode,
    repeat_count: usize,
) -> AtlasResult<P62RealRatioReport> {
    let program = validate_file(path)?;
    p62_real_ratio_report_from_program_with_runs(&program, path.to_string(), mode, repeat_count)
}

pub fn p62_real_ratio_report_json_file(path: &str, mode: WorkloadMode) -> AtlasResult<String> {
    p62_real_ratio_report_file(path, mode).map(|report| p62_real_ratio_report_to_json(&report))
}

pub fn p62_real_ratio_report_json_file_with_runs(
    path: &str,
    mode: WorkloadMode,
    repeat_count: usize,
) -> AtlasResult<String> {
    p62_real_ratio_report_file_with_runs(path, mode, repeat_count)
        .map(|report| p62_real_ratio_report_to_json(&report))
}

pub fn p62_real_ratio_report_from_program(
    program: &AtlasProgram,
    program_path: String,
    mode: WorkloadMode,
) -> AtlasResult<P62RealRatioReport> {
    p62_real_ratio_report_from_program_with_runs(program, program_path, mode, 1)
}

pub fn p62_real_ratio_report_from_program_with_runs(
    program: &AtlasProgram,
    program_path: String,
    mode: WorkloadMode,
    repeat_count: usize,
) -> AtlasResult<P62RealRatioReport> {
    if repeat_count == 0 {
        return Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            "ratio-real requires --runs greater than zero",
        ));
    }
    let profile = P62MeasurementProfile::for_mode(mode);
    let config = RuntimeConfig::from_checked_program(program);
    let virtual_report = p61_virtual_ratio_report_from_program(program, program_path.clone(), mode);
    let measurement_id = measurement_id();
    let mut measured_runs = Vec::new();

    for run_index in 0..repeat_count {
        let temp_dir = create_temp_dir()?;
        let measured = measure_in_temp_dir(&config, mode, profile, &temp_dir);
        let _ = fs::remove_dir_all(&temp_dir);
        let measured = measured?;
        measured_runs.push(run_from_measured(
            run_index,
            &measurement_id,
            &measured,
            virtual_report.metrics.virtual_effective,
            virtual_report.metrics.guard_refused,
            virtual_report.metrics.dangerous_or_adversarial_refused,
        ));
    }

    let summary = P62MeasurementSummary::from_runs(&measured_runs);
    let first_run = measured_runs
        .first()
        .expect("repeat_count > 0 creates at least one run");

    let persisted_bytes = first_run.persisted_bytes.clone();
    let total_persisted_bytes = persisted_bytes.total();
    let virtual_effective = virtual_report.metrics.virtual_effective;
    let ratio_effective_per_byte = ratio(virtual_effective, total_persisted_bytes as u128);
    let ratio_effective_per_read_p99_us =
        ratio(virtual_effective, first_run.read_timing.p99_us as u128);
    let ratio_effective_per_update_p99_us =
        ratio(virtual_effective, first_run.update_timing.p99_us as u128);
    let ratio_effective_per_snapshot_p99_us =
        ratio(virtual_effective, first_run.snapshot_timing.p99_us as u128);

    let guard_refused = virtual_report.metrics.guard_refused;
    let dangerous_or_adversarial_refused = virtual_report.metrics.dangerous_or_adversarial_refused;
    let decision = p62_decision_for_runs(
        virtual_effective,
        guard_refused,
        dangerous_or_adversarial_refused,
        &summary,
        &measured_runs,
    );
    let decision_reasons = p62_decision_reasons(decision, &summary, &measured_runs);

    let warnings = vec![
        "measured_real_v1 uses real Instant timings and filesystem metadata".to_string(),
        "results are machine-local and should not be treated as scientific validation".to_string(),
        "temporary persistence artifacts are removed after measurement".to_string(),
        "P62 keeps deterministic workloads; external datasets are not measured yet".to_string(),
        "decision remains conservative until the measurement model is calibrated".to_string(),
    ];

    Ok(P62RealRatioReport {
        astra_iteration: "ASTRA-P62".to_string(),
        mode: mode.as_str().to_string(),
        program_path,
        cost_model: COST_MODEL.to_string(),
        measurement_kind: MEASUREMENT_KIND.to_string(),
        repeat_count,
        iteration_count: profile.iteration_count,
        warmup_count: profile.warmup_count,
        operation_count: profile.iteration_count,
        measurement_id,
        timestamp: None,
        machine_info: P62MachineInfo {
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
            family: env::consts::FAMILY.to_string(),
            rust_version: None,
            cargo_version: None,
            cpu_info: None,
        },
        commit_hash: None,
        summary,
        create_timing: first_run.create_timing,
        read_timing: first_run.read_timing,
        update_timing: first_run.update_timing,
        delete_timing: first_run.delete_timing,
        snapshot_timing: first_run.snapshot_timing,
        rebuild_timing: first_run.rebuild_timing,
        audit_timing: first_run.audit_timing,
        persisted_bytes,
        virtual_declared: virtual_report.metrics.virtual_declared,
        virtual_reachable: virtual_report.metrics.virtual_reachable,
        virtual_readable: virtual_report.metrics.virtual_readable,
        virtual_updatable: virtual_report.metrics.virtual_updatable,
        virtual_safe: virtual_report.metrics.virtual_safe,
        virtual_effective,
        ratio_effective_per_byte,
        ratio_effective_per_read_p99_us,
        ratio_effective_per_update_p99_us,
        ratio_effective_per_snapshot_p99_us,
        create_count: first_run.create_count,
        read_count: first_run.read_count,
        update_count: first_run.update_count,
        delete_count: first_run.delete_count,
        snapshot_count: first_run.snapshot_count,
        rebuild_count: first_run.rebuild_count,
        audit_count: first_run.audit_count,
        guard_refused,
        dangerous_or_adversarial_refused,
        audit_passed: first_run.audit_passed,
        rebuild_passed: first_run.rebuild_passed,
        snapshot_roundtrip_passed: first_run.snapshot_roundtrip_passed,
        decision,
        decision_reasons,
        warnings,
        runs: measured_runs,
        workloads: virtual_report
            .workloads
            .iter()
            .map(|workload| P62WorkloadSummary {
                id: workload.id().to_string(),
                kind: workload.kind_label().to_string(),
                mechanism: workload.mechanism().to_string(),
                virtual_declared: workload.virtual_declared,
                virtual_effective: workload.virtual_effective,
                accepted: workload.accepted,
                refused: workload.refused(),
                refusal_reason: workload.refusal_reason.clone(),
            })
            .collect(),
    })
}

#[derive(Debug, Clone)]
struct P62MeasuredRuntime {
    create_timing: P62TimingStats,
    read_timing: P62TimingStats,
    update_timing: P62TimingStats,
    delete_timing: P62TimingStats,
    snapshot_timing: P62TimingStats,
    rebuild_timing: P62TimingStats,
    audit_timing: P62TimingStats,
    persisted_bytes: P62PersistedBytes,
    create_count: usize,
    read_count: usize,
    update_count: usize,
    delete_count: usize,
    snapshot_count: usize,
    rebuild_count: usize,
    audit_count: usize,
    audit_passed: bool,
    rebuild_passed: bool,
    snapshot_roundtrip_passed: bool,
}

impl P62MeasurementSummary {
    fn from_runs(runs: &[P62MeasuredRun]) -> Self {
        Self {
            total_persisted_bytes_min: min_u64(
                runs.iter().map(|run| run.total_persisted_bytes).collect(),
            ),
            total_persisted_bytes_median: median_u64(
                runs.iter().map(|run| run.total_persisted_bytes).collect(),
            ),
            total_persisted_bytes_max: max_u64(
                runs.iter().map(|run| run.total_persisted_bytes).collect(),
            ),
            ratio_effective_per_byte_min: min_f64(
                runs.iter()
                    .map(|run| run.ratio_effective_per_byte)
                    .collect(),
            ),
            ratio_effective_per_byte_median: median_f64(
                runs.iter()
                    .map(|run| run.ratio_effective_per_byte)
                    .collect(),
            ),
            ratio_effective_per_byte_max: max_f64(
                runs.iter()
                    .map(|run| run.ratio_effective_per_byte)
                    .collect(),
            ),
            read_p99_us_min: min_u64(runs.iter().map(|run| run.read_timing.p99_us).collect()),
            read_p99_us_median: median_u64(runs.iter().map(|run| run.read_timing.p99_us).collect()),
            read_p99_us_max: max_u64(runs.iter().map(|run| run.read_timing.p99_us).collect()),
            update_p99_us_min: min_u64(runs.iter().map(|run| run.update_timing.p99_us).collect()),
            update_p99_us_median: median_u64(
                runs.iter().map(|run| run.update_timing.p99_us).collect(),
            ),
            update_p99_us_max: max_u64(runs.iter().map(|run| run.update_timing.p99_us).collect()),
            snapshot_p99_us_min: min_u64(
                runs.iter().map(|run| run.snapshot_timing.p99_us).collect(),
            ),
            snapshot_p99_us_median: median_u64(
                runs.iter().map(|run| run.snapshot_timing.p99_us).collect(),
            ),
            snapshot_p99_us_max: max_u64(
                runs.iter().map(|run| run.snapshot_timing.p99_us).collect(),
            ),
            rebuild_p99_us_min: min_u64(runs.iter().map(|run| run.rebuild_timing.p99_us).collect()),
            rebuild_p99_us_median: median_u64(
                runs.iter().map(|run| run.rebuild_timing.p99_us).collect(),
            ),
            rebuild_p99_us_max: max_u64(runs.iter().map(|run| run.rebuild_timing.p99_us).collect()),
            audit_p99_us_min: min_u64(runs.iter().map(|run| run.audit_timing.p99_us).collect()),
            audit_p99_us_median: median_u64(
                runs.iter().map(|run| run.audit_timing.p99_us).collect(),
            ),
            audit_p99_us_max: max_u64(runs.iter().map(|run| run.audit_timing.p99_us).collect()),
            all_runs_passed: runs.iter().all(run_passed),
            run_count: runs.len(),
        }
    }
}

fn run_from_measured(
    run_index: usize,
    measurement_id: &str,
    measured: &P62MeasuredRuntime,
    virtual_effective: u128,
    guard_refused: bool,
    dangerous_or_adversarial_refused: bool,
) -> P62MeasuredRun {
    let total_persisted_bytes = measured.persisted_bytes.total();
    P62MeasuredRun {
        run_index,
        run_id: format!("{}:{}", measurement_id, run_index),
        persisted_bytes: measured.persisted_bytes.clone(),
        total_persisted_bytes,
        create_timing: measured.create_timing,
        read_timing: measured.read_timing,
        update_timing: measured.update_timing,
        delete_timing: measured.delete_timing,
        snapshot_timing: measured.snapshot_timing,
        rebuild_timing: measured.rebuild_timing,
        audit_timing: measured.audit_timing,
        virtual_effective,
        ratio_effective_per_byte: ratio(virtual_effective, total_persisted_bytes as u128),
        guard_refused,
        dangerous_or_adversarial_refused,
        audit_passed: measured.audit_passed,
        rebuild_passed: measured.rebuild_passed,
        snapshot_roundtrip_passed: measured.snapshot_roundtrip_passed,
        create_count: measured.create_count,
        read_count: measured.read_count,
        update_count: measured.update_count,
        delete_count: measured.delete_count,
        snapshot_count: measured.snapshot_count,
        rebuild_count: measured.rebuild_count,
        audit_count: measured.audit_count,
        warnings: vec![
            "run uses real Instant timings; values are machine-local".to_string(),
            "temporary artifacts were measured via filesystem metadata".to_string(),
        ],
    }
}

fn run_passed(run: &P62MeasuredRun) -> bool {
    run.total_persisted_bytes > 0
        && run.read_timing.p99_us > 0
        && run.update_timing.p99_us > 0
        && run.snapshot_timing.p99_us > 0
        && run.guard_refused
        && run.dangerous_or_adversarial_refused
        && run.audit_passed
        && run.rebuild_passed
        && run.snapshot_roundtrip_passed
        && run.virtual_effective > 0
}

fn measure_in_temp_dir(
    config: &RuntimeConfig,
    mode: WorkloadMode,
    profile: P62MeasurementProfile,
    temp_dir: &Path,
) -> AtlasResult<P62MeasuredRuntime> {
    run_warmup(config, mode, profile.warmup_count);

    let workload = RuntimeWorkload::for_mode(config, mode);
    let seed_records = &workload.records;
    let mut runtime = MemoryRuntime::new(config.clone());
    let mut payload = BTreeMap::new();
    let mut created_keys = Vec::new();
    let mut create_samples = Vec::new();
    let mut read_samples = Vec::new();
    let mut update_samples = Vec::new();
    let mut delete_samples = Vec::new();
    let mut journal = String::new();

    for idx in 0..profile.iteration_count {
        let record = measured_record(seed_records, idx);
        let sample = measure_us(|| runtime.encode(&record));
        create_samples.push(sample);
        created_keys.push((record.family.clone(), record.key.clone()));
        payload.insert(
            format!("{}:{}", record.family, record.key),
            record.value.clone(),
        );
        journal.push_str(&format!("create {} {}\n", record.family, record.key));
    }

    for idx in 0..profile.iteration_count {
        let (family, key) = &created_keys[idx % created_keys.len()];
        let sample = measure_us(|| runtime.read(family, key));
        read_samples.push(sample);
        journal.push_str(&format!("read {} {}\n", family, key));
    }

    for idx in 0..profile.iteration_count {
        let (family, key) = &created_keys[idx % created_keys.len()];
        let value = format!("p62:{}:{}:updated", mode.as_str(), idx);
        let sample = measure_us(|| runtime.update(family, key, &value));
        update_samples.push(sample);
        payload.insert(format!("{}:{}", family, key), value);
        journal.push_str(&format!("update {} {}\n", family, key));
    }

    for (family, key) in created_keys.iter().take(profile.delete_count) {
        let sample = measure_us(|| runtime.delete(family, key));
        delete_samples.push(sample);
        payload.remove(&format!("{}:{}", family, key));
        journal.push_str(&format!("delete {} {}\n", family, key));
    }

    write_string(&temp_dir.join("journal.log"), &journal)?;
    write_string(&temp_dir.join("index.txt"), &index_text(&created_keys))?;
    write_string(&temp_dir.join("payload.txt"), &payload_text(&payload))?;

    let mut snapshot_holder = None;
    let snapshot_us = measure_us(|| {
        let snapshot = runtime.snapshot();
        let text = snapshot_text(&snapshot);
        snapshot_holder = Some((snapshot, text));
    });
    let (snapshot, snapshot_text) = snapshot_holder.expect("snapshot is always captured");
    write_string(&temp_dir.join("snapshot.txt"), &snapshot_text)?;

    let before_rebuild = runtime.state_summary();
    let mut rebuilt_holder = None;
    let rebuild_us = measure_us(|| {
        rebuilt_holder = Some(MemoryRuntime::rebuild(config.clone(), &snapshot));
    });
    let rebuilt = rebuilt_holder.expect("rebuild is always captured");
    let after_rebuild = rebuilt.state_summary();
    let rebuild_passed = before_rebuild == after_rebuild;
    let snapshot_roundtrip_passed = rebuild_passed;

    let audit_us = measure_us(|| {
        let audit_passed = snapshot.total_segments == payload.len() && rebuild_passed;
        let audit = format!(
            "audit_passed={}\nsnapshot_segments={}\npayload_entries={}\nrebuild_passed={}\n",
            audit_passed,
            snapshot.total_segments,
            payload.len(),
            rebuild_passed
        );
        let _ = write_string(&temp_dir.join("audit.txt"), &audit);
    });
    let audit_passed = snapshot.total_segments == payload.len() && rebuild_passed;

    let manifest = format!(
        "mode={}\niteration_count={}\nwarmup_count={}\ncreate_count={}\nread_count={}\nupdate_count={}\ndelete_count={}\nsnapshot_count=1\nrebuild_count=1\naudit_count=1\n",
        mode.as_str(),
        profile.iteration_count,
        profile.warmup_count,
        profile.iteration_count,
        profile.iteration_count,
        profile.iteration_count,
        profile.delete_count
    );
    write_string(&temp_dir.join("manifest.txt"), &manifest)?;

    Ok(P62MeasuredRuntime {
        create_timing: timing_stats(create_samples),
        read_timing: timing_stats(read_samples),
        update_timing: timing_stats(update_samples),
        delete_timing: timing_stats(delete_samples),
        snapshot_timing: timing_stats(vec![snapshot_us]),
        rebuild_timing: timing_stats(vec![rebuild_us]),
        audit_timing: timing_stats(vec![audit_us]),
        persisted_bytes: P62PersistedBytes {
            snapshot_file_bytes: file_len(&temp_dir.join("snapshot.txt"))?,
            manifest_file_bytes: file_len(&temp_dir.join("manifest.txt"))?,
            journal_file_bytes: file_len(&temp_dir.join("journal.log"))?,
            index_file_bytes: file_len(&temp_dir.join("index.txt"))?,
            payload_file_bytes: file_len(&temp_dir.join("payload.txt"))?,
            audit_file_bytes: file_len(&temp_dir.join("audit.txt"))?,
        },
        create_count: profile.iteration_count,
        read_count: profile.iteration_count,
        update_count: profile.iteration_count,
        delete_count: profile.delete_count,
        snapshot_count: 1,
        rebuild_count: 1,
        audit_count: 1,
        audit_passed,
        rebuild_passed,
        snapshot_roundtrip_passed,
    })
}

fn run_warmup(config: &RuntimeConfig, mode: WorkloadMode, warmup_count: usize) {
    let workload = RuntimeWorkload::for_mode(config, mode);
    let seed_records = &workload.records;
    let mut runtime = MemoryRuntime::new(config.clone());
    let mut keys = Vec::new();
    for idx in 0..warmup_count {
        let record = measured_record(seed_records, idx);
        runtime.encode(&record);
        runtime.read(&record.family, &record.key);
        runtime.update(&record.family, &record.key, "warmup:update");
        keys.push((record.family, record.key));
    }
    for (family, key) in keys.iter().take(warmup_count / 10) {
        runtime.delete(family, key);
    }
    let snapshot = runtime.snapshot();
    let _rebuilt = MemoryRuntime::rebuild(config.clone(), &snapshot);
}

fn measured_record(seed_records: &[WorkloadRecord], idx: usize) -> WorkloadRecord {
    let seed = &seed_records[idx % seed_records.len()];
    WorkloadRecord {
        family: seed.family.clone(),
        key: format!("p62:{}:{}", idx, seed.key),
        value: format!("{}:p62:{}", seed.value, idx),
    }
}

fn measure_us<T>(mut operation: impl FnMut() -> T) -> u64 {
    let start = Instant::now();
    let _ = operation();
    nanos_to_micros_ceil(start.elapsed().as_nanos())
}

fn nanos_to_micros_ceil(nanos: u128) -> u64 {
    if nanos == 0 {
        0
    } else {
        ((nanos + 999) / 1_000).min(u64::MAX as u128) as u64
    }
}

fn timing_stats(mut samples: Vec<u64>) -> P62TimingStats {
    samples.sort_unstable();
    P62TimingStats {
        p50_us: percentile(&samples, 50),
        p95_us: percentile(&samples, 95),
        p99_us: percentile(&samples, 99),
    }
}

fn percentile(samples: &[u64], percentile: usize) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let rank = ((samples.len() * percentile).div_ceil(100)).saturating_sub(1);
    samples[rank.min(samples.len() - 1)]
}

fn min_u64(samples: Vec<u64>) -> u64 {
    samples.into_iter().min().unwrap_or(0)
}

fn median_u64(mut samples: Vec<u64>) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    samples.sort_unstable();
    samples[(samples.len() - 1) / 2]
}

fn max_u64(samples: Vec<u64>) -> u64 {
    samples.into_iter().max().unwrap_or(0)
}

fn min_f64(samples: Vec<f64>) -> f64 {
    samples.into_iter().reduce(f64::min).unwrap_or(0.0)
}

fn median_f64(mut samples: Vec<f64>) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.sort_by(|a, b| a.partial_cmp(b).expect("ratio samples are finite"));
    samples[(samples.len() - 1) / 2]
}

fn max_f64(samples: Vec<f64>) -> f64 {
    samples.into_iter().reduce(f64::max).unwrap_or(0.0)
}

fn snapshot_text(snapshot: &crate::MemorySnapshot) -> String {
    let mut out = String::new();
    out.push_str(&format!("checksum={}\n", snapshot.checksum));
    out.push_str(&format!("total_segments={}\n", snapshot.total_segments));
    for family in &snapshot.families {
        out.push_str(&format!(
            "family={} segments={} checksum={}\n",
            family.name, family.segments, family.checksum
        ));
    }
    for (family, records) in &snapshot.records {
        for record in records {
            out.push_str(&format!(
                "record family={} key={} version={} value={}\n",
                family, record.key, record.version, record.value
            ));
        }
    }
    out
}

fn index_text(keys: &[(String, String)]) -> String {
    let mut out = String::new();
    for (family, key) in keys {
        out.push_str(&format!("{} {}\n", family, key));
    }
    out
}

fn payload_text(payload: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (key, value) in payload {
        out.push_str(&format!("{}={}\n", key, value));
    }
    out
}

fn create_temp_dir() -> AtlasResult<PathBuf> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| io_diagnostic(format!("system time error: {}", err)))?
        .as_nanos();
    for _ in 0..100 {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "astra-p62-{}-{}-{}",
            std::process::id(),
            nonce,
            counter
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(io_diagnostic(format!("create temp dir: {}", err)));
            }
        }
    }
    Err(io_diagnostic(
        "create temp dir: exhausted unique path attempts".to_string(),
    ))
}

fn measurement_id() -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("p62-{}-{}", std::process::id(), nonce)
}

fn write_string(path: &Path, content: &str) -> AtlasResult<()> {
    fs::write(path, content).map_err(|err| io_diagnostic(format!("write {:?}: {}", path, err)))
}

fn file_len(path: &Path) -> AtlasResult<u64> {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|err| io_diagnostic(format!("metadata {:?}: {}", path, err)))
}

fn p62_decision_for_runs(
    virtual_effective: u128,
    guard_refused: bool,
    dangerous_or_adversarial_refused: bool,
    summary: &P62MeasurementSummary,
    runs: &[P62MeasuredRun],
) -> P62Decision {
    if !guard_refused
        || !dangerous_or_adversarial_refused
        || virtual_effective == 0
        || runs.is_empty()
    {
        return P62Decision::NoGoRealRatio;
    }
    if !summary.all_runs_passed
        || summary.total_persisted_bytes_min == 0
        || summary.read_p99_us_min == 0
        || summary.update_p99_us_min == 0
        || summary.snapshot_p99_us_min == 0
    {
        return P62Decision::RecalibrateRuntimePaths;
    }
    P62Decision::RecalibrateMeasurementModel
}

fn p62_decision_reasons(
    decision: P62Decision,
    summary: &P62MeasurementSummary,
    runs: &[P62MeasuredRun],
) -> Vec<String> {
    let mut reasons = vec![
        format!("measured run count: {}", summary.run_count),
        "timing fields are measured with std::time::Instant".to_string(),
        "persisted bytes are measured with filesystem metadata".to_string(),
        "workloads are deterministic internal workloads, not external datasets".to_string(),
        "thresholds are not calibrated yet".to_string(),
    ];
    if runs.iter().all(|run| run.total_persisted_bytes > 0) {
        reasons.push("all measured runs produced persisted bytes".to_string());
    }
    if runs.iter().all(|run| {
        run.read_timing.p99_us > 0 && run.update_timing.p99_us > 0 && run.snapshot_timing.p99_us > 0
    }) {
        reasons.push("all measured runs produced non-zero p99 timings".to_string());
    }
    if summary.all_runs_passed {
        reasons.push("all measured runs passed audit/rebuild/snapshot gates".to_string());
    }
    if decision == P62Decision::RecalibrateMeasurementModel {
        reasons
            .push("decision remains recalibrate until cost thresholds are calibrated".to_string());
    }
    reasons
}

pub fn p62_real_ratio_report_to_json(report: &P62RealRatioReport) -> String {
    let bytes = &report.persisted_bytes;
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string(
        "astra_iteration",
        &report.astra_iteration,
        true,
        2,
    ));
    out.push_str(&json_string("mode", &report.mode, true, 2));
    out.push_str(&json_string("program_path", &report.program_path, true, 2));
    out.push_str(&json_string("cost_model", &report.cost_model, true, 2));
    out.push_str(&json_string(
        "measurement_kind",
        &report.measurement_kind,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "iteration_count",
        report.iteration_count,
        true,
        2,
    ));
    out.push_str(&json_usize("repeat_count", report.repeat_count, true, 2));
    out.push_str(&json_usize("warmup_count", report.warmup_count, true, 2));
    out.push_str(&json_usize(
        "operation_count",
        report.operation_count,
        true,
        2,
    ));
    out.push_str(&json_string(
        "measurement_id",
        &report.measurement_id,
        true,
        2,
    ));
    out.push_str("  \"timestamp\": null,\n");
    out.push_str("  \"machine_info\": {\n");
    out.push_str(&json_string("os", &report.machine_info.os, true, 4));
    out.push_str(&json_string("arch", &report.machine_info.arch, true, 4));
    out.push_str(&json_string("family", &report.machine_info.family, true, 4));
    out.push_str("    \"rust_version\": null,\n");
    out.push_str("    \"cargo_version\": null,\n");
    out.push_str("    \"cpu_info\": null\n");
    out.push_str("  },\n");
    out.push_str("  \"commit_hash\": null,\n");
    summary_json(&mut out, &report.summary);
    timing_json(&mut out, "create", report.create_timing);
    timing_json(&mut out, "read", report.read_timing);
    timing_json(&mut out, "update", report.update_timing);
    timing_json(&mut out, "delete", report.delete_timing);
    timing_json(&mut out, "snapshot", report.snapshot_timing);
    timing_json(&mut out, "rebuild", report.rebuild_timing);
    timing_json(&mut out, "audit", report.audit_timing);
    out.push_str(&json_u64(
        "snapshot_file_bytes",
        bytes.snapshot_file_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "manifest_file_bytes",
        bytes.manifest_file_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "journal_file_bytes",
        bytes.journal_file_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "index_file_bytes",
        bytes.index_file_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "payload_file_bytes",
        bytes.payload_file_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "audit_file_bytes",
        bytes.audit_file_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64("total_persisted_bytes", bytes.total(), true, 2));
    out.push_str(&json_u128(
        "virtual_declared",
        report.virtual_declared,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_reachable",
        report.virtual_reachable,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_readable",
        report.virtual_readable,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_updatable",
        report.virtual_updatable,
        true,
        2,
    ));
    out.push_str(&json_u128("virtual_safe", report.virtual_safe, true, 2));
    out.push_str(&json_u128(
        "virtual_effective",
        report.virtual_effective,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_byte",
        report.ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_read_p99_us",
        report.ratio_effective_per_read_p99_us,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_update_p99_us",
        report.ratio_effective_per_update_p99_us,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_snapshot_p99_us",
        report.ratio_effective_per_snapshot_p99_us,
        true,
        2,
    ));
    out.push_str(&json_usize("create_count", report.create_count, true, 2));
    out.push_str(&json_usize("read_count", report.read_count, true, 2));
    out.push_str(&json_usize("update_count", report.update_count, true, 2));
    out.push_str(&json_usize("delete_count", report.delete_count, true, 2));
    out.push_str(&json_usize(
        "snapshot_count",
        report.snapshot_count,
        true,
        2,
    ));
    out.push_str(&json_usize("rebuild_count", report.rebuild_count, true, 2));
    out.push_str(&json_usize("audit_count", report.audit_count, true, 2));
    out.push_str(&json_bool("guard_refused", report.guard_refused, true, 2));
    out.push_str(&json_bool(
        "dangerous_or_adversarial_refused",
        report.dangerous_or_adversarial_refused,
        true,
        2,
    ));
    out.push_str(&json_bool("audit_passed", report.audit_passed, true, 2));
    out.push_str(&json_bool("rebuild_passed", report.rebuild_passed, true, 2));
    out.push_str(&json_bool(
        "snapshot_roundtrip_passed",
        report.snapshot_roundtrip_passed,
        true,
        2,
    ));
    out.push_str(&json_string("decision", report.decision.as_str(), true, 2));
    out.push_str("  \"decision_reasons\": [\n");
    for (idx, reason) in report.decision_reasons.iter().enumerate() {
        out.push_str(&format!(
            "    \"{}\"{}\n",
            escape_json(reason),
            comma(idx, report.decision_reasons.len())
        ));
    }
    out.push_str("  ],\n");
    out.push_str("  \"warnings\": [\n");
    for (idx, warning) in report.warnings.iter().enumerate() {
        out.push_str(&format!(
            "    \"{}\"{}\n",
            escape_json(warning),
            comma(idx, report.warnings.len())
        ));
    }
    out.push_str("  ],\n");
    runs_json(&mut out, &report.runs);
    out.push_str("  \"workloads\": [\n");
    for (idx, workload) in report.workloads.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&json_string("id", &workload.id, true, 6));
        out.push_str(&json_string("kind", &workload.kind, true, 6));
        out.push_str(&json_string("mechanism", &workload.mechanism, true, 6));
        out.push_str(&json_u128(
            "virtual_declared",
            workload.virtual_declared,
            true,
            6,
        ));
        out.push_str(&json_u128(
            "virtual_effective",
            workload.virtual_effective,
            true,
            6,
        ));
        out.push_str(&json_bool("accepted", workload.accepted, true, 6));
        out.push_str(&json_bool("refused", workload.refused, true, 6));
        out.push_str(&json_string(
            "refusal_reason",
            &workload.refusal_reason,
            false,
            6,
        ));
        out.push_str(&format!("    }}{}\n", comma(idx, report.workloads.len())));
    }
    out.push_str("  ]\n");
    out.push('}');
    out
}

fn summary_json(out: &mut String, summary: &P62MeasurementSummary) {
    out.push_str("  \"summary\": {\n");
    out.push_str(&json_u64(
        "total_persisted_bytes_min",
        summary.total_persisted_bytes_min,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "total_persisted_bytes_median",
        summary.total_persisted_bytes_median,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "total_persisted_bytes_max",
        summary.total_persisted_bytes_max,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_byte_min",
        summary.ratio_effective_per_byte_min,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_byte_median",
        summary.ratio_effective_per_byte_median,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_byte_max",
        summary.ratio_effective_per_byte_max,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "read_p99_us_min",
        summary.read_p99_us_min,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "read_p99_us_median",
        summary.read_p99_us_median,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "read_p99_us_max",
        summary.read_p99_us_max,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "update_p99_us_min",
        summary.update_p99_us_min,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "update_p99_us_median",
        summary.update_p99_us_median,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "update_p99_us_max",
        summary.update_p99_us_max,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "snapshot_p99_us_min",
        summary.snapshot_p99_us_min,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "snapshot_p99_us_median",
        summary.snapshot_p99_us_median,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "snapshot_p99_us_max",
        summary.snapshot_p99_us_max,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "rebuild_p99_us_min",
        summary.rebuild_p99_us_min,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "rebuild_p99_us_median",
        summary.rebuild_p99_us_median,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "rebuild_p99_us_max",
        summary.rebuild_p99_us_max,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "audit_p99_us_min",
        summary.audit_p99_us_min,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "audit_p99_us_median",
        summary.audit_p99_us_median,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "audit_p99_us_max",
        summary.audit_p99_us_max,
        true,
        4,
    ));
    out.push_str(&json_bool(
        "all_runs_passed",
        summary.all_runs_passed,
        true,
        4,
    ));
    out.push_str(&json_usize("run_count", summary.run_count, false, 4));
    out.push_str("  },\n");
}

fn runs_json(out: &mut String, runs: &[P62MeasuredRun]) {
    out.push_str("  \"runs\": [\n");
    for (idx, run) in runs.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&json_usize("run_index", run.run_index, true, 6));
        out.push_str(&json_string("run_id", &run.run_id, true, 6));
        out.push_str(&json_u64(
            "total_persisted_bytes",
            run.total_persisted_bytes,
            true,
            6,
        ));
        timing_json_with_indent(out, "create", run.create_timing, 6);
        timing_json_with_indent(out, "read", run.read_timing, 6);
        timing_json_with_indent(out, "update", run.update_timing, 6);
        timing_json_with_indent(out, "delete", run.delete_timing, 6);
        timing_json_with_indent(out, "snapshot", run.snapshot_timing, 6);
        timing_json_with_indent(out, "rebuild", run.rebuild_timing, 6);
        timing_json_with_indent(out, "audit", run.audit_timing, 6);
        out.push_str(&json_u128(
            "virtual_effective",
            run.virtual_effective,
            true,
            6,
        ));
        out.push_str(&json_f64(
            "ratio_effective_per_byte",
            run.ratio_effective_per_byte,
            true,
            6,
        ));
        out.push_str(&json_bool("guard_refused", run.guard_refused, true, 6));
        out.push_str(&json_bool(
            "dangerous_or_adversarial_refused",
            run.dangerous_or_adversarial_refused,
            true,
            6,
        ));
        out.push_str(&json_bool("audit_passed", run.audit_passed, true, 6));
        out.push_str(&json_bool("rebuild_passed", run.rebuild_passed, true, 6));
        out.push_str(&json_bool(
            "snapshot_roundtrip_passed",
            run.snapshot_roundtrip_passed,
            true,
            6,
        ));
        out.push_str("      \"warnings\": [\n");
        for (warning_idx, warning) in run.warnings.iter().enumerate() {
            out.push_str(&format!(
                "        \"{}\"{}\n",
                escape_json(warning),
                comma(warning_idx, run.warnings.len())
            ));
        }
        out.push_str("      ]\n");
        out.push_str(&format!("    }}{}\n", comma(idx, runs.len())));
    }
    out.push_str("  ],\n");
}

fn timing_json(out: &mut String, prefix: &str, timing: P62TimingStats) {
    timing_json_with_indent(out, prefix, timing, 2);
}

fn timing_json_with_indent(out: &mut String, prefix: &str, timing: P62TimingStats, indent: usize) {
    out.push_str(&json_u64(
        &format!("{}_p50_us", prefix),
        timing.p50_us,
        true,
        indent,
    ));
    out.push_str(&json_u64(
        &format!("{}_p95_us", prefix),
        timing.p95_us,
        true,
        indent,
    ));
    out.push_str(&json_u64(
        &format!("{}_p99_us", prefix),
        timing.p99_us,
        true,
        indent,
    ));
}

fn ratio(numerator: u128, denominator: u128) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn json_string(key: &str, value: &str, comma: bool, indent: usize) -> String {
    json_line(key, &format!("\"{}\"", escape_json(value)), comma, indent)
}

fn json_bool(key: &str, value: bool, comma: bool, indent: usize) -> String {
    json_line(key, if value { "true" } else { "false" }, comma, indent)
}

fn json_usize(key: &str, value: usize, comma: bool, indent: usize) -> String {
    json_line(key, &value.to_string(), comma, indent)
}

fn json_u64(key: &str, value: u64, comma: bool, indent: usize) -> String {
    json_line(key, &value.to_string(), comma, indent)
}

fn json_u128(key: &str, value: u128, comma: bool, indent: usize) -> String {
    json_line(key, &value.to_string(), comma, indent)
}

fn json_f64(key: &str, value: f64, comma: bool, indent: usize) -> String {
    json_line(key, &format!("{:.6}", value), comma, indent)
}

fn json_line(key: &str, value: &str, comma: bool, indent: usize) -> String {
    format!(
        "{:indent$}\"{}\": {}{}\n",
        "",
        escape_json(key),
        value,
        if comma { "," } else { "" },
        indent = indent
    )
}

fn comma(idx: usize, len: usize) -> &'static str {
    if idx + 1 == len {
        ""
    } else {
        ","
    }
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn io_diagnostic(message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Io, message)
}
