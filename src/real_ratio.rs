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
pub struct P62RealRatioReport {
    pub astra_iteration: String,
    pub mode: String,
    pub program_path: String,
    pub cost_model: String,
    pub measurement_kind: String,
    pub iteration_count: usize,
    pub warmup_count: usize,
    pub machine_info: P62MachineInfo,
    pub commit_hash: Option<String>,
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
    pub warnings: Vec<String>,
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
    let program = validate_file(path)?;
    p62_real_ratio_report_from_program(&program, path.to_string(), mode)
}

pub fn p62_real_ratio_report_json_file(path: &str, mode: WorkloadMode) -> AtlasResult<String> {
    p62_real_ratio_report_file(path, mode).map(|report| p62_real_ratio_report_to_json(&report))
}

pub fn p62_real_ratio_report_from_program(
    program: &AtlasProgram,
    program_path: String,
    mode: WorkloadMode,
) -> AtlasResult<P62RealRatioReport> {
    let profile = P62MeasurementProfile::for_mode(mode);
    let config = RuntimeConfig::from_checked_program(program);
    let virtual_report = p61_virtual_ratio_report_from_program(program, program_path.clone(), mode);
    let temp_dir = create_temp_dir()?;
    let measured = measure_in_temp_dir(&config, mode, profile, &temp_dir);
    let _ = fs::remove_dir_all(&temp_dir);
    let measured = measured?;

    let persisted_bytes = measured.persisted_bytes.clone();
    let total_persisted_bytes = persisted_bytes.total();
    let virtual_effective = virtual_report.metrics.virtual_effective;
    let ratio_effective_per_byte = ratio(virtual_effective, total_persisted_bytes as u128);
    let ratio_effective_per_read_p99_us =
        ratio(virtual_effective, measured.read_timing.p99_us as u128);
    let ratio_effective_per_update_p99_us =
        ratio(virtual_effective, measured.update_timing.p99_us as u128);
    let ratio_effective_per_snapshot_p99_us =
        ratio(virtual_effective, measured.snapshot_timing.p99_us as u128);

    let guard_refused = virtual_report.metrics.guard_refused;
    let dangerous_or_adversarial_refused = virtual_report.metrics.dangerous_or_adversarial_refused;
    let decision = p62_decision(
        virtual_effective,
        total_persisted_bytes,
        &measured,
        guard_refused,
        dangerous_or_adversarial_refused,
    );

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
        iteration_count: profile.iteration_count,
        warmup_count: profile.warmup_count,
        machine_info: P62MachineInfo {
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
            family: env::consts::FAMILY.to_string(),
        },
        commit_hash: None,
        create_timing: measured.create_timing,
        read_timing: measured.read_timing,
        update_timing: measured.update_timing,
        delete_timing: measured.delete_timing,
        snapshot_timing: measured.snapshot_timing,
        rebuild_timing: measured.rebuild_timing,
        audit_timing: measured.audit_timing,
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
        create_count: measured.create_count,
        read_count: measured.read_count,
        update_count: measured.update_count,
        delete_count: measured.delete_count,
        snapshot_count: measured.snapshot_count,
        rebuild_count: measured.rebuild_count,
        audit_count: measured.audit_count,
        guard_refused,
        dangerous_or_adversarial_refused,
        audit_passed: measured.audit_passed,
        rebuild_passed: measured.rebuild_passed,
        snapshot_roundtrip_passed: measured.snapshot_roundtrip_passed,
        decision,
        warnings,
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

fn write_string(path: &Path, content: &str) -> AtlasResult<()> {
    fs::write(path, content).map_err(|err| io_diagnostic(format!("write {:?}: {}", path, err)))
}

fn file_len(path: &Path) -> AtlasResult<u64> {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|err| io_diagnostic(format!("metadata {:?}: {}", path, err)))
}

fn p62_decision(
    virtual_effective: u128,
    total_persisted_bytes: u64,
    measured: &P62MeasuredRuntime,
    guard_refused: bool,
    dangerous_or_adversarial_refused: bool,
) -> P62Decision {
    if !guard_refused
        || !dangerous_or_adversarial_refused
        || !measured.audit_passed
        || !measured.rebuild_passed
        || !measured.snapshot_roundtrip_passed
        || virtual_effective == 0
    {
        return P62Decision::NoGoRealRatio;
    }
    if total_persisted_bytes == 0
        || measured.read_timing.p99_us == 0
        || measured.update_timing.p99_us == 0
        || measured.snapshot_timing.p99_us == 0
    {
        return P62Decision::RecalibrateRuntimePaths;
    }
    P62Decision::RecalibrateMeasurementModel
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
    out.push_str(&json_usize("warmup_count", report.warmup_count, true, 2));
    out.push_str("  \"machine_info\": {\n");
    out.push_str(&json_string("os", &report.machine_info.os, true, 4));
    out.push_str(&json_string("arch", &report.machine_info.arch, true, 4));
    out.push_str(&json_string(
        "family",
        &report.machine_info.family,
        false,
        4,
    ));
    out.push_str("  },\n");
    out.push_str("  \"commit_hash\": null,\n");
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
    out.push_str("  \"warnings\": [\n");
    for (idx, warning) in report.warnings.iter().enumerate() {
        out.push_str(&format!(
            "    \"{}\"{}\n",
            escape_json(warning),
            comma(idx, report.warnings.len())
        ));
    }
    out.push_str("  ],\n");
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

fn timing_json(out: &mut String, prefix: &str, timing: P62TimingStats) {
    out.push_str(&json_u64(
        &format!("{}_p50_us", prefix),
        timing.p50_us,
        true,
        2,
    ));
    out.push_str(&json_u64(
        &format!("{}_p95_us", prefix),
        timing.p95_us,
        true,
        2,
    ));
    out.push_str(&json_u64(
        &format!("{}_p99_us", prefix),
        timing.p99_us,
        true,
        2,
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
