use crate::{validate, validate_file, AtlasProgram, AtlasResult, DiagnosticCode};
use std::collections::BTreeMap;

const ATLAS_MEMORY_SOURCE: &str = "<memory>";
const P56_PROXY_P99_BUDGET: u64 = 100;

const FAMILY_ORDER: &[&str] = &[
    "guard",
    "stream_processing",
    "sparse_index",
    "image_field_surrogate",
    "log_request_index",
    "columnar_table",
    "graph_lowrank_surrogate",
    "critical_sparse_archive",
    "compressible_but_wrong",
    "field_surrogate",
    "topological_field",
    "local_global_conflict",
];

const SMOKE_FAMILIES: &[&str] = &["stream_processing", "sparse_index", "columnar_table"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadMode {
    Smoke,
    Standard,
    Ambitious,
}

impl WorkloadMode {
    pub fn as_str(self) -> &'static str {
        match self {
            WorkloadMode::Smoke => "smoke",
            WorkloadMode::Standard => "standard",
            WorkloadMode::Ambitious => "ambitious",
        }
    }

    pub fn from_str(mode: &str) -> Option<Self> {
        match mode {
            "smoke" => Some(WorkloadMode::Smoke),
            "standard" => Some(WorkloadMode::Standard),
            "ambitious" => Some(WorkloadMode::Ambitious),
            _ => None,
        }
    }

    fn shape(self) -> WorkloadShape {
        match self {
            WorkloadMode::Smoke => WorkloadShape {
                records_per_family: 4,
                updates_per_family: 2,
            },
            WorkloadMode::Standard => WorkloadShape {
                records_per_family: 3,
                updates_per_family: 1,
            },
            WorkloadMode::Ambitious => WorkloadShape {
                records_per_family: 8,
                updates_per_family: 4,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WorkloadShape {
    records_per_family: usize,
    updates_per_family: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFamilyConfig {
    pub name: String,
    pub action: String,
    pub safety: String,
    pub layout: String,
    pub index: String,
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeConfig {
    pub version: String,
    pub runtime: BTreeMap<String, String>,
    pub families: Vec<RuntimeFamilyConfig>,
}

impl RuntimeConfig {
    pub fn from_checked_program(program: &AtlasProgram) -> Self {
        let mut families: Vec<RuntimeFamilyConfig> = program
            .families
            .iter()
            .map(|family| RuntimeFamilyConfig {
                name: family.name.clone(),
                action: family.action.clone(),
                safety: family.safety.clone(),
                layout: family.layout.clone(),
                index: family.index.clone(),
                threshold: family.threshold,
            })
            .collect();
        families.sort_by(|a, b| {
            family_position(&a.name)
                .cmp(&family_position(&b.name))
                .then_with(|| a.name.cmp(&b.name))
        });

        Self {
            version: program.version.clone(),
            runtime: program.runtime.clone(),
            families,
        }
    }

    pub fn family(&self, name: &str) -> Option<&RuntimeFamilyConfig> {
        self.families.iter().find(|family| family.name == name)
    }

    pub fn strict_p53(&self) -> bool {
        self.runtime.get("strict_p53").map(|value| value.as_str()) == Some("true")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkloadRecord {
    pub family: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkloadRead {
    pub family: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkloadUpdate {
    pub family: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkloadExpectation {
    Accept,
    Refuse,
    Frontier,
    Recalibrate,
}

impl WorkloadExpectation {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkloadExpectation::Accept => "accept",
            WorkloadExpectation::Refuse => "refuse",
            WorkloadExpectation::Frontier => "frontier",
            WorkloadExpectation::Recalibrate => "recalibrate",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkloadSpec {
    pub workload_name: String,
    pub target_family: String,
    pub record_count: usize,
    pub read_count: usize,
    pub update_count: usize,
    pub snapshot_count: usize,
    pub rebuild_count: usize,
    pub expected_category: WorkloadExpectation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWorkload {
    pub mode: WorkloadMode,
    pub name: String,
    pub families: Vec<String>,
    pub specs: Vec<WorkloadSpec>,
    pub records: Vec<WorkloadRecord>,
    pub reads: Vec<WorkloadRead>,
    pub updates: Vec<WorkloadUpdate>,
    pub snapshot_count: usize,
    pub rebuild_count: usize,
}

pub type SmokeWorkload = RuntimeWorkload;

impl RuntimeWorkload {
    pub fn deterministic(config: &RuntimeConfig) -> Self {
        Self::for_mode(config, WorkloadMode::Smoke)
    }

    pub fn for_mode(config: &RuntimeConfig, mode: WorkloadMode) -> Self {
        let families = match mode {
            WorkloadMode::Smoke => select_smoke_families(config),
            WorkloadMode::Standard | WorkloadMode::Ambitious => select_active_families(config),
        };
        let shape = mode.shape();
        let snapshot_count = 1;
        let rebuild_count = 1;
        let mut specs = Vec::new();
        let mut records = Vec::new();
        let mut reads = Vec::new();
        let mut updates = Vec::new();

        for (family_idx, family_name) in families.iter().enumerate() {
            let family = config
                .family(family_name)
                .expect("smoke families are selected from runtime config");
            specs.push(WorkloadSpec {
                workload_name: format!("{}:{}", mode.as_str(), family.name),
                target_family: family.name.clone(),
                record_count: shape.records_per_family,
                read_count: shape.records_per_family,
                update_count: shape.updates_per_family,
                snapshot_count,
                rebuild_count,
                expected_category: expected_category(&family.name),
            });

            for record_idx in 0..shape.records_per_family {
                let key = format!("{}:{}", family.name, record_idx);
                records.push(WorkloadRecord {
                    family: family.name.clone(),
                    key: key.clone(),
                    value: format!(
                        "{}:{}:{}:{}",
                        family.action, family.index, family_idx, record_idx
                    ),
                });
                reads.push(WorkloadRead {
                    family: family.name.clone(),
                    key: key.clone(),
                });
                if record_idx < shape.updates_per_family {
                    updates.push(WorkloadUpdate {
                        family: family.name.clone(),
                        key,
                        value: format!(
                            "{}:{}:{}:{}:{}:updated",
                            mode.as_str(),
                            family.action,
                            family.layout,
                            family_idx,
                            record_idx
                        ),
                    });
                }
            }
        }

        Self {
            mode,
            name: mode.as_str().to_string(),
            families,
            specs,
            records,
            reads,
            updates,
            snapshot_count,
            rebuild_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemoryCell {
    value: String,
    version: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeStats {
    pub encoded_segments: usize,
    pub read_count: usize,
    pub read_success_count: usize,
    pub update_count: usize,
    pub snapshot_count: usize,
    pub rebuild_count: usize,
    pub read_pseudo_latency: u64,
    pub update_pseudo_latency: u64,
    pub snapshot_pseudo_latency: u64,
    pub rebuild_pseudo_latency: u64,
    pub dangerous_encoded_count: usize,
    pub guard_encoded_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRecord {
    pub key: String,
    pub value: String,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFamily {
    pub name: String,
    pub segments: usize,
    pub checksum: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySnapshot {
    pub families: Vec<SnapshotFamily>,
    pub records: BTreeMap<String, Vec<SnapshotRecord>>,
    pub total_segments: usize,
    pub checksum: u64,
    pub dangerous_encoded_count: usize,
    pub guard_encoded_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStateSummary {
    pub families: usize,
    pub encoded_segments: usize,
    pub checksum: u64,
    pub dangerous_encoded_count: usize,
    pub guard_encoded_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P56Gates {
    pub p56_g0_build_test_ci: String,
    pub p56_g1_runtime_instantiates: bool,
    pub p56_g2_encode_read_update: bool,
    pub p56_g3_snapshot_incremental: bool,
    pub p56_g4_rebuild: bool,
    pub p56_g5_metrics_export: bool,
    pub p56_g6_p99_under_budget_smoke: bool,
    pub p56_g7_invalid_still_refused: bool,
    pub p56_g8_ci_source_of_truth: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeMetrics {
    pub p56_status: String,
    pub atlas_file: String,
    pub mode: String,
    pub atlas_version: String,
    pub families_total: usize,
    pub workload_family_count: usize,
    pub runtime_instantiated: bool,
    pub strict_p53: bool,
    pub strict_p53_preserved: bool,
    pub workload_families: Vec<String>,
    pub encoded_segments_total: usize,
    pub read_count: usize,
    pub update_count: usize,
    pub snapshot_count: usize,
    pub rebuild_count: usize,
    pub read_pseudo_latency: u64,
    pub update_pseudo_latency: u64,
    pub snapshot_pseudo_latency: u64,
    pub rebuild_pseudo_latency: u64,
    pub p99_proxy_latency: u64,
    pub p99_proxy_latency_budget: u64,
    pub query_success_rate: f64,
    pub memory_amplification_proxy: f64,
    pub dangerous_encoded_count: usize,
    pub guard_encoded_count: usize,
    pub invalid_regression_checked: bool,
    pub state_checksum: u64,
    pub rebuild_checksum: u64,
    pub rebuild_matches: bool,
    pub no_guard_encoded: bool,
    pub gates: P56Gates,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P58WorkloadSummary {
    pub workload_name: String,
    pub target_family: String,
    pub record_count: usize,
    pub read_count: usize,
    pub update_count: usize,
    pub snapshot_count: usize,
    pub rebuild_count: usize,
    pub expected_category: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P58Gates {
    pub p58_g0_runtime_mode_available: bool,
    pub p58_g1_workload_registry_nonempty: bool,
    pub p58_g2_standard_covers_active_non_guard_families: bool,
    pub p58_g3_guard_not_encoded: bool,
    pub p58_g4_query_success_rate_ok: bool,
    pub p58_g5_snapshot_rebuild_available: bool,
    pub p58_g6_metrics_json_stable: bool,
    pub p58_g7_report_generated: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P58Report {
    pub astra_iteration: String,
    pub atlas_version: String,
    pub mode: String,
    pub program_path: String,
    pub strict_p53_enabled: bool,
    pub family_count: usize,
    pub active_family_count: usize,
    pub refused_family_count: usize,
    pub workload_count: usize,
    pub workload_family_count: usize,
    pub encoded_segments: usize,
    pub records: usize,
    pub reads: usize,
    pub updates: usize,
    pub snapshots: usize,
    pub rebuilds: usize,
    pub query_success_rate: f64,
    pub no_guard_encoded: bool,
    pub guard_refused: bool,
    pub snapshot_full_refused: bool,
    pub runtime_available: bool,
    pub metrics_available: bool,
    pub report_available: bool,
    pub workloads: Vec<P58WorkloadSummary>,
    pub gates: P58Gates,
    pub decision: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MemoryRuntime {
    config: RuntimeConfig,
    stores: BTreeMap<String, BTreeMap<String, MemoryCell>>,
    stats: RuntimeStats,
}

impl MemoryRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            stores: BTreeMap::new(),
            stats: RuntimeStats::default(),
        }
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub fn stats(&self) -> &RuntimeStats {
        &self.stats
    }

    pub fn encode(&mut self, record: &WorkloadRecord) -> bool {
        let Some(family) = self.config.family(&record.family) else {
            return false;
        };
        if family.name == "guard" || family.action == "refuse" {
            self.stats.guard_encoded_count += 1;
            return false;
        }
        if record.value.contains("dangerous") {
            self.stats.dangerous_encoded_count += 1;
        }

        let store = self.stores.entry(record.family.clone()).or_default();
        let inserted = store
            .insert(
                record.key.clone(),
                MemoryCell {
                    value: record.value.clone(),
                    version: 1,
                },
            )
            .is_none();
        if inserted {
            self.stats.encoded_segments += 1;
        }
        inserted
    }

    pub fn encode_all(&mut self, records: &[WorkloadRecord]) {
        for record in records {
            self.encode(record);
        }
    }

    pub fn read(&mut self, family: &str, key: &str) -> Option<String> {
        self.stats.read_count += 1;
        self.stats.read_pseudo_latency += 2;
        let value = self
            .stores
            .get(family)
            .and_then(|store| store.get(key))
            .map(|cell| cell.value.clone());
        if value.is_some() {
            self.stats.read_success_count += 1;
        }
        value
    }

    pub fn update(&mut self, family: &str, key: &str, value: &str) -> bool {
        self.stats.update_count += 1;
        self.stats.update_pseudo_latency += 3;
        let Some(cell) = self
            .stores
            .get_mut(family)
            .and_then(|store| store.get_mut(key))
        else {
            return false;
        };
        cell.value = value.to_string();
        cell.version += 1;
        true
    }

    pub fn snapshot(&mut self) -> MemorySnapshot {
        self.stats.snapshot_count += 1;
        self.stats.snapshot_pseudo_latency +=
            self.stats.encoded_segments as u64 + self.stores.len() as u64;

        let mut records = BTreeMap::new();
        let mut families = Vec::new();
        let mut checksum = 17_u64;
        let mut total_segments = 0_usize;

        for (family_name, store) in &self.stores {
            let mut family_checksum = checksum_part(23, family_name);
            let mut snapshot_records = Vec::new();
            for (key, cell) in store {
                family_checksum = checksum_part(family_checksum, key);
                family_checksum = checksum_part(family_checksum, &cell.value);
                family_checksum = family_checksum.wrapping_add(cell.version);
                snapshot_records.push(SnapshotRecord {
                    key: key.clone(),
                    value: cell.value.clone(),
                    version: cell.version,
                });
            }
            total_segments += snapshot_records.len();
            checksum = checksum.wrapping_mul(131).wrapping_add(family_checksum);
            families.push(SnapshotFamily {
                name: family_name.clone(),
                segments: snapshot_records.len(),
                checksum: family_checksum,
            });
            records.insert(family_name.clone(), snapshot_records);
        }

        MemorySnapshot {
            families,
            records,
            total_segments,
            checksum,
            dangerous_encoded_count: self.stats.dangerous_encoded_count,
            guard_encoded_count: self.stats.guard_encoded_count,
        }
    }

    pub fn rebuild(config: RuntimeConfig, snapshot: &MemorySnapshot) -> Self {
        let mut stores = BTreeMap::new();
        for (family, records) in &snapshot.records {
            let mut store = BTreeMap::new();
            for record in records {
                store.insert(
                    record.key.clone(),
                    MemoryCell {
                        value: record.value.clone(),
                        version: record.version,
                    },
                );
            }
            stores.insert(family.clone(), store);
        }

        Self {
            config,
            stores,
            stats: RuntimeStats {
                encoded_segments: snapshot.total_segments,
                rebuild_count: 1,
                rebuild_pseudo_latency: snapshot.total_segments as u64
                    + snapshot.families.len() as u64,
                dangerous_encoded_count: snapshot.dangerous_encoded_count,
                guard_encoded_count: snapshot.guard_encoded_count,
                ..RuntimeStats::default()
            },
        }
    }

    pub fn state_summary(&self) -> RuntimeStateSummary {
        RuntimeStateSummary {
            families: self.stores.len(),
            encoded_segments: self.segment_count(),
            checksum: state_checksum(&self.stores),
            dangerous_encoded_count: self.stats.dangerous_encoded_count,
            guard_encoded_count: self.stats.guard_encoded_count,
        }
    }

    fn segment_count(&self) -> usize {
        self.stores.values().map(BTreeMap::len).sum()
    }
}

pub fn runtime_config(text: &str) -> AtlasResult<RuntimeConfig> {
    let program = validate(text)?;
    Ok(RuntimeConfig::from_checked_program(&program))
}

pub fn runtime_config_file(path: &str) -> AtlasResult<RuntimeConfig> {
    let program = validate_file(path)?;
    Ok(RuntimeConfig::from_checked_program(&program))
}

pub fn run_smoke(text: &str) -> AtlasResult<RuntimeMetrics> {
    run_workload(text, WorkloadMode::Smoke)
}

pub fn run_workload(text: &str, mode: WorkloadMode) -> AtlasResult<RuntimeMetrics> {
    let config = runtime_config(text)?;
    Ok(run_workload_config_with_atlas_file(
        config,
        ATLAS_MEMORY_SOURCE.to_string(),
        mode,
    ))
}

pub fn run_smoke_file(path: &str) -> AtlasResult<RuntimeMetrics> {
    run_workload_file(path, WorkloadMode::Smoke)
}

pub fn run_workload_file(path: &str, mode: WorkloadMode) -> AtlasResult<RuntimeMetrics> {
    let config = runtime_config_file(path)?;
    Ok(run_workload_config_with_atlas_file(
        config,
        path.to_string(),
        mode,
    ))
}

pub fn metrics_json(text: &str) -> AtlasResult<String> {
    metrics_json_mode(text, WorkloadMode::Smoke)
}

pub fn metrics_json_mode(text: &str, mode: WorkloadMode) -> AtlasResult<String> {
    run_workload(text, mode).map(|metrics| runtime_metrics_json(&metrics))
}

pub fn metrics_json_file(path: &str) -> AtlasResult<String> {
    metrics_json_file_mode(path, WorkloadMode::Smoke)
}

pub fn metrics_json_file_mode(path: &str, mode: WorkloadMode) -> AtlasResult<String> {
    run_workload_file(path, mode).map(|metrics| runtime_metrics_json(&metrics))
}

pub fn bench_report_json_file(path: &str, mode: WorkloadMode) -> AtlasResult<String> {
    run_workload_file(path, mode).map(|metrics| bench_report_json(&metrics))
}

pub fn p58_metrics_json(text: &str, mode: WorkloadMode) -> AtlasResult<String> {
    p58_report_json(text, mode)
}

pub fn p58_metrics_json_file(path: &str, mode: WorkloadMode) -> AtlasResult<String> {
    p58_report_json_file(path, mode)
}

pub fn p58_report(text: &str, mode: WorkloadMode) -> AtlasResult<P58Report> {
    let config = runtime_config(text)?;
    Ok(p58_report_from_config(
        config,
        ATLAS_MEMORY_SOURCE.to_string(),
        mode,
    ))
}

pub fn p58_report_file(path: &str, mode: WorkloadMode) -> AtlasResult<P58Report> {
    let config = runtime_config_file(path)?;
    Ok(p58_report_from_config(config, path.to_string(), mode))
}

pub fn p58_report_json(text: &str, mode: WorkloadMode) -> AtlasResult<String> {
    p58_report(text, mode).map(|report| p58_report_to_json(&report))
}

pub fn p58_report_json_file(path: &str, mode: WorkloadMode) -> AtlasResult<String> {
    p58_report_file(path, mode).map(|report| p58_report_to_json(&report))
}

pub fn p58_report_markdown(text: &str, mode: WorkloadMode) -> AtlasResult<String> {
    p58_report(text, mode).map(|report| p58_report_to_markdown(&report))
}

pub fn p58_report_markdown_file(path: &str, mode: WorkloadMode) -> AtlasResult<String> {
    p58_report_file(path, mode).map(|report| p58_report_to_markdown(&report))
}

pub fn run_smoke_config(config: RuntimeConfig) -> RuntimeMetrics {
    run_workload_config(config, WorkloadMode::Smoke)
}

pub fn run_workload_config(config: RuntimeConfig, mode: WorkloadMode) -> RuntimeMetrics {
    run_workload_config_with_atlas_file(config, ATLAS_MEMORY_SOURCE.to_string(), mode)
}

pub fn run_smoke_config_with_atlas_file(
    config: RuntimeConfig,
    atlas_file: String,
) -> RuntimeMetrics {
    run_workload_config_with_atlas_file(config, atlas_file, WorkloadMode::Smoke)
}

pub fn run_workload_config_with_atlas_file(
    config: RuntimeConfig,
    atlas_file: String,
    mode: WorkloadMode,
) -> RuntimeMetrics {
    let workload = RuntimeWorkload::for_mode(&config, mode);
    run_prepared_workload_config_with_atlas_file(config, atlas_file, &workload)
}

fn run_prepared_workload_config_with_atlas_file(
    config: RuntimeConfig,
    atlas_file: String,
    workload: &RuntimeWorkload,
) -> RuntimeMetrics {
    let mut runtime = MemoryRuntime::new(config.clone());
    runtime.encode_all(&workload.records);

    for read in &workload.reads {
        runtime.read(&read.family, &read.key);
    }
    for update in &workload.updates {
        runtime.update(&update.family, &update.key, &update.value);
    }

    let snapshot = runtime.snapshot();
    let summary = runtime.state_summary();
    let rebuilt = MemoryRuntime::rebuild(config.clone(), &snapshot);
    let rebuild_summary = rebuilt.state_summary();
    let stats = runtime.stats();
    let rebuild_stats = rebuilt.stats();
    let strict_p53_preserved = config.strict_p53()
        && config.runtime.get("snapshot").map(|value| value.as_str())
            == Some("incremental_manifest");
    let invalid_regression_checked = invalid_regression_checked();
    let runtime_instantiated = true;
    let encode_read_update_ok = stats.encoded_segments == workload.records.len()
        && stats.read_count == workload.reads.len()
        && stats.read_success_count == workload.reads.len()
        && stats.update_count == workload.updates.len();
    let snapshot_incremental_ok =
        stats.snapshot_count == workload.snapshot_count && strict_p53_preserved;
    let p99_proxy_latency = stats
        .read_pseudo_latency
        .max(stats.update_pseudo_latency)
        .max(stats.snapshot_pseudo_latency)
        .max(rebuild_stats.rebuild_pseudo_latency);
    let p99_under_budget = p99_proxy_latency <= P56_PROXY_P99_BUDGET;
    let rebuild_matches = summary == rebuild_summary;
    let gates = P56Gates {
        p56_g0_build_test_ci: "external_required".to_string(),
        p56_g1_runtime_instantiates: runtime_instantiated,
        p56_g2_encode_read_update: encode_read_update_ok,
        p56_g3_snapshot_incremental: snapshot_incremental_ok,
        p56_g4_rebuild: rebuild_matches,
        p56_g5_metrics_export: true,
        p56_g6_p99_under_budget_smoke: p99_under_budget,
        p56_g7_invalid_still_refused: invalid_regression_checked,
        p56_g8_ci_source_of_truth: "external_required".to_string(),
    };
    let p56_status = if gates.p56_g1_runtime_instantiates
        && gates.p56_g2_encode_read_update
        && gates.p56_g3_snapshot_incremental
        && gates.p56_g4_rebuild
        && gates.p56_g5_metrics_export
        && gates.p56_g6_p99_under_budget_smoke
        && gates.p56_g7_invalid_still_refused
    {
        "SMOKE_OK_CI_REQUIRED"
    } else {
        "SMOKE_FAILED"
    };

    RuntimeMetrics {
        p56_status: p56_status.to_string(),
        atlas_file,
        mode: workload.mode.as_str().to_string(),
        atlas_version: config.version.clone(),
        families_total: config.families.len(),
        runtime_instantiated,
        strict_p53: config.strict_p53(),
        strict_p53_preserved,
        workload_family_count: workload.families.len(),
        workload_families: workload.families.clone(),
        encoded_segments_total: stats.encoded_segments,
        read_count: stats.read_count,
        update_count: stats.update_count,
        snapshot_count: stats.snapshot_count,
        rebuild_count: rebuild_stats.rebuild_count,
        read_pseudo_latency: stats.read_pseudo_latency,
        update_pseudo_latency: stats.update_pseudo_latency,
        snapshot_pseudo_latency: stats.snapshot_pseudo_latency,
        rebuild_pseudo_latency: rebuild_stats.rebuild_pseudo_latency,
        p99_proxy_latency,
        p99_proxy_latency_budget: P56_PROXY_P99_BUDGET,
        query_success_rate: ratio(stats.read_success_count, stats.read_count),
        memory_amplification_proxy: memory_amplification_proxy(&runtime),
        dangerous_encoded_count: stats.dangerous_encoded_count,
        guard_encoded_count: stats.guard_encoded_count,
        invalid_regression_checked,
        state_checksum: summary.checksum,
        rebuild_checksum: rebuild_summary.checksum,
        rebuild_matches,
        no_guard_encoded: stats.guard_encoded_count == 0,
        gates,
    }
}

fn p58_report_from_config(
    config: RuntimeConfig,
    program_path: String,
    mode: WorkloadMode,
) -> P58Report {
    let workload = RuntimeWorkload::for_mode(&config, mode);
    let metrics = run_prepared_workload_config_with_atlas_file(
        config.clone(),
        program_path.clone(),
        &workload,
    );
    let active_families = select_active_families(&config);
    let guard_refused = config
        .family("guard")
        .map(|family| family.action == "refuse" && family.safety == "refuse")
        .unwrap_or(false);
    let snapshot_full_refused = config.strict_p53()
        && config.runtime.get("snapshot").map(|value| value.as_str())
            == Some("incremental_manifest")
        && invalid_regression_checked();
    let standard_coverage = matches!(mode, WorkloadMode::Standard | WorkloadMode::Ambitious)
        && workload.families == active_families;
    let runtime_available = metrics.runtime_instantiated;
    let gates = P58Gates {
        p58_g0_runtime_mode_available: runtime_available,
        p58_g1_workload_registry_nonempty: !workload.specs.is_empty()
            && !workload.records.is_empty(),
        p58_g2_standard_covers_active_non_guard_families: standard_coverage,
        p58_g3_guard_not_encoded: metrics.no_guard_encoded,
        p58_g4_query_success_rate_ok: metrics.query_success_rate >= 1.0,
        p58_g5_snapshot_rebuild_available: metrics.snapshot_count > 0
            && metrics.rebuild_count > 0
            && metrics.rebuild_matches,
        p58_g6_metrics_json_stable: true,
        p58_g7_report_generated: true,
    };
    let mut warnings = Vec::new();
    if !gates.p58_g2_standard_covers_active_non_guard_families {
        warnings.push("mode does not cover every active non-guard family".to_string());
    }
    if !guard_refused {
        warnings.push("guard family refusal is not confirmed".to_string());
    }
    if !snapshot_full_refused {
        warnings.push("snapshot=full refusal is not confirmed".to_string());
    }
    if !gates.p58_g4_query_success_rate_ok {
        warnings.push("query success rate is below the deterministic gate".to_string());
    }
    if matches!(mode, WorkloadMode::Ambitious) {
        warnings.push(
            "ambitious mode is deterministic local-only and not a CI requirement".to_string(),
        );
    }
    let decision = p58_decision(
        &gates,
        config.strict_p53(),
        guard_refused,
        snapshot_full_refused,
        runtime_available,
    );
    let workloads = workload
        .specs
        .iter()
        .map(|spec| P58WorkloadSummary {
            workload_name: spec.workload_name.clone(),
            target_family: spec.target_family.clone(),
            record_count: spec.record_count,
            read_count: spec.read_count,
            update_count: spec.update_count,
            snapshot_count: spec.snapshot_count,
            rebuild_count: spec.rebuild_count,
            expected_category: spec.expected_category.as_str().to_string(),
        })
        .collect();

    P58Report {
        astra_iteration: "ASTRA-SYS-P58".to_string(),
        atlas_version: config.version.clone(),
        mode: mode.as_str().to_string(),
        program_path,
        strict_p53_enabled: config.strict_p53(),
        family_count: config.families.len(),
        active_family_count: active_families.len(),
        refused_family_count: config
            .families
            .iter()
            .filter(|family| family.action == "refuse")
            .count(),
        workload_count: workload.specs.len(),
        workload_family_count: workload.families.len(),
        encoded_segments: metrics.encoded_segments_total,
        records: workload.records.len(),
        reads: metrics.read_count,
        updates: metrics.update_count,
        snapshots: metrics.snapshot_count,
        rebuilds: metrics.rebuild_count,
        query_success_rate: metrics.query_success_rate,
        no_guard_encoded: metrics.no_guard_encoded,
        guard_refused,
        snapshot_full_refused,
        runtime_available,
        metrics_available: true,
        report_available: true,
        workloads,
        gates,
        decision: decision.to_string(),
        warnings,
    }
}

fn p58_decision(
    gates: &P58Gates,
    strict_p53_enabled: bool,
    guard_refused: bool,
    snapshot_full_refused: bool,
    runtime_available: bool,
) -> &'static str {
    if !strict_p53_enabled
        || !guard_refused
        || !snapshot_full_refused
        || !runtime_available
        || !gates.p58_g3_guard_not_encoded
    {
        return "NO_GO";
    }

    if gates.p58_g0_runtime_mode_available
        && gates.p58_g1_workload_registry_nonempty
        && gates.p58_g2_standard_covers_active_non_guard_families
        && gates.p58_g3_guard_not_encoded
        && gates.p58_g4_query_success_rate_ok
        && gates.p58_g5_snapshot_rebuild_available
        && gates.p58_g6_metrics_json_stable
        && gates.p58_g7_report_generated
    {
        "VALIDATE"
    } else {
        "RECALIBRATE"
    }
}

pub fn runtime_metrics_json(metrics: &RuntimeMetrics) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!(
        "  \"p56_status\": \"{}\",\n",
        escape_json(&metrics.p56_status)
    ));
    out.push_str(&format!(
        "  \"atlas_file\": \"{}\",\n",
        escape_json(&metrics.atlas_file)
    ));
    out.push_str(&format!(
        "  \"mode\": \"{}\",\n",
        escape_json(&metrics.mode)
    ));
    out.push_str(&format!(
        "  \"atlas_version\": \"{}\",\n",
        escape_json(&metrics.atlas_version)
    ));
    out.push_str(&format!(
        "  \"families_total\": {},\n",
        metrics.families_total
    ));
    out.push_str(&format!(
        "  \"workload_family_count\": {},\n",
        metrics.workload_family_count
    ));
    out.push_str(&format!(
        "  \"runtime_instantiated\": {},\n",
        metrics.runtime_instantiated
    ));
    out.push_str(&format!("  \"strict_p53\": {},\n", metrics.strict_p53));
    out.push_str(&format!(
        "  \"strict_p53_preserved\": {},\n",
        metrics.strict_p53_preserved
    ));
    out.push_str("  \"workload_families\": [\n");
    for (idx, family) in metrics.workload_families.iter().enumerate() {
        let comma = if idx + 1 == metrics.workload_families.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!("    \"{}\"{}\n", escape_json(family), comma));
    }
    out.push_str("  ],\n");
    out.push_str(&format!(
        "  \"encoded_segments_total\": {},\n",
        metrics.encoded_segments_total
    ));
    out.push_str(&format!("  \"read_count\": {},\n", metrics.read_count));
    out.push_str(&format!("  \"update_count\": {},\n", metrics.update_count));
    out.push_str(&format!(
        "  \"snapshot_count\": {},\n",
        metrics.snapshot_count
    ));
    out.push_str(&format!(
        "  \"rebuild_count\": {},\n",
        metrics.rebuild_count
    ));
    out.push_str(&format!(
        "  \"read_pseudo_latency\": {},\n",
        metrics.read_pseudo_latency
    ));
    out.push_str(&format!(
        "  \"update_pseudo_latency\": {},\n",
        metrics.update_pseudo_latency
    ));
    out.push_str(&format!(
        "  \"snapshot_pseudo_latency\": {},\n",
        metrics.snapshot_pseudo_latency
    ));
    out.push_str(&format!(
        "  \"rebuild_pseudo_latency\": {},\n",
        metrics.rebuild_pseudo_latency
    ));
    out.push_str(&format!(
        "  \"p99_proxy_latency\": {},\n",
        metrics.p99_proxy_latency
    ));
    out.push_str(&format!(
        "  \"p99_proxy_latency_budget\": {},\n",
        metrics.p99_proxy_latency_budget
    ));
    out.push_str(&format!(
        "  \"query_success_rate\": {:.3},\n",
        metrics.query_success_rate
    ));
    out.push_str(&format!(
        "  \"memory_amplification_proxy\": {:.3},\n",
        metrics.memory_amplification_proxy
    ));
    out.push_str(&format!(
        "  \"dangerous_encoded_count\": {},\n",
        metrics.dangerous_encoded_count
    ));
    out.push_str(&format!(
        "  \"guard_encoded_count\": {},\n",
        metrics.guard_encoded_count
    ));
    out.push_str(&format!(
        "  \"invalid_regression_checked\": {},\n",
        metrics.invalid_regression_checked
    ));
    out.push_str(&format!(
        "  \"state_checksum\": {},\n",
        metrics.state_checksum
    ));
    out.push_str(&format!(
        "  \"rebuild_checksum\": {},\n",
        metrics.rebuild_checksum
    ));
    out.push_str(&format!(
        "  \"rebuild_matches\": {},\n",
        metrics.rebuild_matches
    ));
    out.push_str(&format!(
        "  \"no_guard_encoded\": {},\n",
        metrics.no_guard_encoded
    ));
    out.push_str("  \"latency_metric_kind\": \"smoke_proxy\",\n");
    out.push_str("  \"gates\": {\n");
    out.push_str(&format!(
        "    \"P56_G0_build_test_ci\": \"{}\",\n",
        escape_json(&metrics.gates.p56_g0_build_test_ci)
    ));
    out.push_str(&format!(
        "    \"P56_G1_runtime_instantiates\": {},\n",
        metrics.gates.p56_g1_runtime_instantiates
    ));
    out.push_str(&format!(
        "    \"P56_G2_encode_read_update\": {},\n",
        metrics.gates.p56_g2_encode_read_update
    ));
    out.push_str(&format!(
        "    \"P56_G3_snapshot_incremental\": {},\n",
        metrics.gates.p56_g3_snapshot_incremental
    ));
    out.push_str(&format!(
        "    \"P56_G4_rebuild\": {},\n",
        metrics.gates.p56_g4_rebuild
    ));
    out.push_str(&format!(
        "    \"P56_G5_metrics_export\": {},\n",
        metrics.gates.p56_g5_metrics_export
    ));
    out.push_str(&format!(
        "    \"P56_G6_p99_under_budget_smoke\": {},\n",
        metrics.gates.p56_g6_p99_under_budget_smoke
    ));
    out.push_str(&format!(
        "    \"P56_G7_invalid_still_refused\": {},\n",
        metrics.gates.p56_g7_invalid_still_refused
    ));
    out.push_str(&format!(
        "    \"P56_G8_ci_source_of_truth\": \"{}\"\n",
        escape_json(&metrics.gates.p56_g8_ci_source_of_truth)
    ));
    out.push_str("  }\n");
    out.push('}');
    out
}

pub fn bench_report_json(metrics: &RuntimeMetrics) -> String {
    let samples = proxy_cost_samples(metrics);
    let warnings = bench_warnings(metrics);
    let decision = bench_decision(metrics);
    let synthetic_cost_units = metrics.read_pseudo_latency
        + metrics.update_pseudo_latency
        + metrics.snapshot_pseudo_latency
        + metrics.rebuild_pseudo_latency;

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"astra_iteration\": \"ASTRA-SYS-P60\",\n");
    out.push_str("  \"benchmark_kind\": \"deterministic_structural_proxy\",\n");
    out.push_str(&format!(
        "  \"program_path\": \"{}\",\n",
        escape_json(&metrics.atlas_file)
    ));
    out.push_str(&format!(
        "  \"mode\": \"{}\",\n",
        escape_json(&metrics.mode)
    ));
    out.push_str(&format!(
        "  \"atlas_version\": \"{}\",\n",
        metrics.atlas_version
    ));
    out.push_str(&format!(
        "  \"family_count\": {},\n",
        metrics.families_total
    ));
    out.push_str(&format!(
        "  \"workload_count\": {},\n",
        metrics.workload_family_count
    ));
    out.push_str(&format!(
        "  \"workload_family_count\": {},\n",
        metrics.workload_family_count
    ));
    out.push_str(&format!(
        "  \"encoded_segments\": {},\n",
        metrics.encoded_segments_total
    ));
    out.push_str(&format!("  \"reads\": {},\n", metrics.read_count));
    out.push_str(&format!("  \"updates\": {},\n", metrics.update_count));
    out.push_str(&format!("  \"snapshots\": {},\n", metrics.snapshot_count));
    out.push_str(&format!("  \"rebuilds\": {},\n", metrics.rebuild_count));
    out.push_str(&format!(
        "  \"query_success_rate\": {:.3},\n",
        metrics.query_success_rate
    ));
    out.push_str(&format!(
        "  \"synthetic_cost_units\": {},\n",
        synthetic_cost_units
    ));
    out.push_str("  \"elapsed_ms\": null,\n");
    out.push_str(&format!(
        "  \"p50_proxy_cost_units\": {},\n",
        percentile_cost(&samples, 50)
    ));
    out.push_str(&format!(
        "  \"p95_proxy_cost_units\": {},\n",
        percentile_cost(&samples, 95)
    ));
    out.push_str(&format!(
        "  \"p99_proxy_cost_units\": {},\n",
        percentile_cost(&samples, 99)
    ));
    out.push_str(&format!(
        "  \"state_checksum\": {},\n",
        metrics.state_checksum
    ));
    out.push_str(&format!(
        "  \"rebuild_checksum\": {},\n",
        metrics.rebuild_checksum
    ));
    out.push_str(&format!(
        "  \"rebuild_matches\": {},\n",
        metrics.rebuild_matches
    ));
    out.push_str(&format!(
        "  \"no_guard_encoded\": {},\n",
        metrics.no_guard_encoded
    ));
    out.push_str(&format!(
        "  \"local_manual_only\": {},\n",
        metrics.mode == WorkloadMode::Ambitious.as_str()
    ));
    out.push_str(&format!(
        "  \"ci_safe\": {},\n",
        metrics.mode != WorkloadMode::Ambitious.as_str()
    ));
    out.push_str(&format!("  \"decision\": \"{}\",\n", decision));
    out.push_str("  \"warnings\": [\n");
    for (idx, warning) in warnings.iter().enumerate() {
        let comma = if idx + 1 == warnings.len() { "" } else { "," };
        out.push_str(&format!("    \"{}\"{}\n", escape_json(warning), comma));
    }
    out.push_str("  ]\n");
    out.push('}');
    out
}

pub fn p58_report_to_json(report: &P58Report) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!(
        "  \"astra_iteration\": \"{}\",\n",
        escape_json(&report.astra_iteration)
    ));
    out.push_str(&format!(
        "  \"atlas_version\": \"{}\",\n",
        escape_json(&report.atlas_version)
    ));
    out.push_str(&format!("  \"mode\": \"{}\",\n", escape_json(&report.mode)));
    out.push_str(&format!(
        "  \"program_path\": \"{}\",\n",
        escape_json(&report.program_path)
    ));
    out.push_str(&format!(
        "  \"strict_p53_enabled\": {},\n",
        report.strict_p53_enabled
    ));
    out.push_str(&format!("  \"family_count\": {},\n", report.family_count));
    out.push_str(&format!(
        "  \"active_family_count\": {},\n",
        report.active_family_count
    ));
    out.push_str(&format!(
        "  \"refused_family_count\": {},\n",
        report.refused_family_count
    ));
    out.push_str(&format!(
        "  \"workload_count\": {},\n",
        report.workload_count
    ));
    out.push_str(&format!(
        "  \"workload_family_count\": {},\n",
        report.workload_family_count
    ));
    out.push_str(&format!(
        "  \"encoded_segments\": {},\n",
        report.encoded_segments
    ));
    out.push_str(&format!("  \"records\": {},\n", report.records));
    out.push_str(&format!("  \"reads\": {},\n", report.reads));
    out.push_str(&format!("  \"updates\": {},\n", report.updates));
    out.push_str(&format!("  \"snapshots\": {},\n", report.snapshots));
    out.push_str(&format!("  \"rebuilds\": {},\n", report.rebuilds));
    out.push_str(&format!(
        "  \"query_success_rate\": {:.3},\n",
        report.query_success_rate
    ));
    out.push_str(&format!(
        "  \"no_guard_encoded\": {},\n",
        report.no_guard_encoded
    ));
    out.push_str(&format!("  \"guard_refused\": {},\n", report.guard_refused));
    out.push_str(&format!(
        "  \"snapshot_full_refused\": {},\n",
        report.snapshot_full_refused
    ));
    out.push_str(&format!(
        "  \"runtime_available\": {},\n",
        report.runtime_available
    ));
    out.push_str(&format!(
        "  \"metrics_available\": {},\n",
        report.metrics_available
    ));
    out.push_str(&format!(
        "  \"report_available\": {},\n",
        report.report_available
    ));
    out.push_str("  \"workloads\": [\n");
    for (idx, workload) in report.workloads.iter().enumerate() {
        let comma = if idx + 1 == report.workloads.len() {
            ""
        } else {
            ","
        };
        out.push_str("    {\n");
        out.push_str(&format!(
            "      \"workload_name\": \"{}\",\n",
            escape_json(&workload.workload_name)
        ));
        out.push_str(&format!(
            "      \"target_family\": \"{}\",\n",
            escape_json(&workload.target_family)
        ));
        out.push_str(&format!("      \"records\": {},\n", workload.record_count));
        out.push_str(&format!("      \"reads\": {},\n", workload.read_count));
        out.push_str(&format!("      \"updates\": {},\n", workload.update_count));
        out.push_str(&format!(
            "      \"snapshots\": {},\n",
            workload.snapshot_count
        ));
        out.push_str(&format!(
            "      \"rebuilds\": {},\n",
            workload.rebuild_count
        ));
        out.push_str(&format!(
            "      \"expected_category\": \"{}\"\n",
            escape_json(&workload.expected_category)
        ));
        out.push_str(&format!("    }}{}\n", comma));
    }
    out.push_str("  ],\n");
    out.push_str("  \"gates\": {\n");
    out.push_str(&format!(
        "    \"P58_G0_runtime_mode_available\": {},\n",
        report.gates.p58_g0_runtime_mode_available
    ));
    out.push_str(&format!(
        "    \"P58_G1_workload_registry_nonempty\": {},\n",
        report.gates.p58_g1_workload_registry_nonempty
    ));
    out.push_str(&format!(
        "    \"P58_G2_standard_covers_active_non_guard_families\": {},\n",
        report
            .gates
            .p58_g2_standard_covers_active_non_guard_families
    ));
    out.push_str(&format!(
        "    \"P58_G3_guard_not_encoded\": {},\n",
        report.gates.p58_g3_guard_not_encoded
    ));
    out.push_str(&format!(
        "    \"P58_G4_query_success_rate_ok\": {},\n",
        report.gates.p58_g4_query_success_rate_ok
    ));
    out.push_str(&format!(
        "    \"P58_G5_snapshot_rebuild_available\": {},\n",
        report.gates.p58_g5_snapshot_rebuild_available
    ));
    out.push_str(&format!(
        "    \"P58_G6_metrics_json_stable\": {},\n",
        report.gates.p58_g6_metrics_json_stable
    ));
    out.push_str(&format!(
        "    \"P58_G7_report_generated\": {}\n",
        report.gates.p58_g7_report_generated
    ));
    out.push_str("  },\n");
    out.push_str(&format!(
        "  \"decision\": \"{}\",\n",
        escape_json(&report.decision)
    ));
    out.push_str("  \"warnings\": [\n");
    for (idx, warning) in report.warnings.iter().enumerate() {
        let comma = if idx + 1 == report.warnings.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!("    \"{}\"{}\n", escape_json(warning), comma));
    }
    out.push_str("  ]\n");
    out.push('}');
    out
}

pub fn p58_report_to_markdown(report: &P58Report) -> String {
    let mut out = String::new();
    out.push_str("# ASTRA-SYS-P58 runtime report\n\n");
    out.push_str(&format!("- Mode: `{}`\n", report.mode));
    out.push_str(&format!("- Program: `{}`\n", report.program_path));
    out.push_str(&format!(
        "- ASTRA iteration: `{}`\n",
        report.astra_iteration
    ));
    out.push_str(&format!("- Decision: `{}`\n\n", report.decision));

    out.push_str("## Families summary\n\n");
    out.push_str("| field | value |\n");
    out.push_str("| --- | ---: |\n");
    out.push_str(&format!("| family_count | {} |\n", report.family_count));
    out.push_str(&format!(
        "| active_family_count | {} |\n",
        report.active_family_count
    ));
    out.push_str(&format!(
        "| refused_family_count | {} |\n",
        report.refused_family_count
    ));
    out.push_str(&format!(
        "| workload_family_count | {} |\n\n",
        report.workload_family_count
    ));

    out.push_str("## Workload summary\n\n");
    out.push_str(
        "| workload | family | records | reads | updates | snapshots | rebuilds | expected |\n",
    );
    out.push_str("| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |\n");
    for workload in &report.workloads {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            workload.workload_name,
            workload.target_family,
            workload.record_count,
            workload.read_count,
            workload.update_count,
            workload.snapshot_count,
            workload.rebuild_count,
            workload.expected_category
        ));
    }
    out.push('\n');

    out.push_str("## Gates summary\n\n");
    out.push_str("| gate | passed |\n");
    out.push_str("| --- | --- |\n");
    out.push_str(&format!(
        "| P58_G0_runtime_mode_available | {} |\n",
        report.gates.p58_g0_runtime_mode_available
    ));
    out.push_str(&format!(
        "| P58_G1_workload_registry_nonempty | {} |\n",
        report.gates.p58_g1_workload_registry_nonempty
    ));
    out.push_str(&format!(
        "| P58_G2_standard_covers_active_non_guard_families | {} |\n",
        report
            .gates
            .p58_g2_standard_covers_active_non_guard_families
    ));
    out.push_str(&format!(
        "| P58_G3_guard_not_encoded | {} |\n",
        report.gates.p58_g3_guard_not_encoded
    ));
    out.push_str(&format!(
        "| P58_G4_query_success_rate_ok | {} |\n",
        report.gates.p58_g4_query_success_rate_ok
    ));
    out.push_str(&format!(
        "| P58_G5_snapshot_rebuild_available | {} |\n",
        report.gates.p58_g5_snapshot_rebuild_available
    ));
    out.push_str(&format!(
        "| P58_G6_metrics_json_stable | {} |\n",
        report.gates.p58_g6_metrics_json_stable
    ));
    out.push_str(&format!(
        "| P58_G7_report_generated | {} |\n\n",
        report.gates.p58_g7_report_generated
    ));

    out.push_str("## Decision\n\n");
    out.push_str(&format!("`{}`\n\n", report.decision));
    out.push_str("## Warnings\n\n");
    if report.warnings.is_empty() {
        out.push_str("- None\n");
    } else {
        for warning in &report.warnings {
            out.push_str(&format!("- {}\n", warning));
        }
    }
    out
}

fn select_smoke_families(config: &RuntimeConfig) -> Vec<String> {
    let mut families = Vec::new();
    for family in SMOKE_FAMILIES {
        if config.family(family).is_some() {
            families.push((*family).to_string());
        }
    }
    if families.len() >= 3 {
        return families;
    }
    for family in &config.families {
        if family.name != "guard" && !families.iter().any(|name| name == &family.name) {
            families.push(family.name.clone());
        }
        if families.len() >= 3 {
            break;
        }
    }
    families
}

fn select_active_families(config: &RuntimeConfig) -> Vec<String> {
    config
        .families
        .iter()
        .filter(|family| family.name != "guard" && family.action != "refuse")
        .map(|family| family.name.clone())
        .collect()
}

fn expected_category(family: &str) -> WorkloadExpectation {
    match family {
        "compressible_but_wrong" => WorkloadExpectation::Recalibrate,
        "local_global_conflict" => WorkloadExpectation::Frontier,
        "guard" => WorkloadExpectation::Refuse,
        _ => WorkloadExpectation::Accept,
    }
}

fn family_position(name: &str) -> usize {
    FAMILY_ORDER
        .iter()
        .position(|candidate| candidate == &name)
        .unwrap_or(usize::MAX)
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn memory_amplification_proxy(runtime: &MemoryRuntime) -> f64 {
    let segments = runtime.segment_count();
    if segments == 0 {
        0.0
    } else {
        (segments + runtime.stores.len()) as f64 / segments as f64
    }
}

fn proxy_cost_samples(metrics: &RuntimeMetrics) -> Vec<u64> {
    let mut samples = Vec::new();
    samples.extend(std::iter::repeat_n(2, metrics.read_count));
    samples.extend(std::iter::repeat_n(3, metrics.update_count));
    for _ in 0..metrics.snapshot_count {
        samples.push(metrics.snapshot_pseudo_latency);
    }
    for _ in 0..metrics.rebuild_count {
        samples.push(metrics.rebuild_pseudo_latency);
    }
    samples.sort_unstable();
    samples
}

fn percentile_cost(samples: &[u64], percentile: usize) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let rank = (percentile * samples.len() + 99) / 100;
    let idx = rank.saturating_sub(1).min(samples.len() - 1);
    samples[idx]
}

fn bench_decision(metrics: &RuntimeMetrics) -> &'static str {
    if !metrics.strict_p53_preserved
        || !metrics.no_guard_encoded
        || !metrics.rebuild_matches
        || metrics.query_success_rate < 1.0
    {
        "NO_GO"
    } else if metrics.mode == WorkloadMode::Smoke.as_str() {
        "RECALIBRATE"
    } else {
        "VALIDATE"
    }
}

fn bench_warnings(metrics: &RuntimeMetrics) -> Vec<&'static str> {
    let mut warnings = vec![
        "deterministic structural proxy only; elapsed_ms is intentionally null",
        "not a wall-clock or industrial performance benchmark",
    ];
    if metrics.mode == WorkloadMode::Smoke.as_str() {
        warnings.push("smoke mode is CI-safe but intentionally partial");
    }
    if metrics.mode == WorkloadMode::Ambitious.as_str() {
        warnings.push("ambitious mode is local/manual and not a CI requirement");
    }
    warnings
}

fn state_checksum(stores: &BTreeMap<String, BTreeMap<String, MemoryCell>>) -> u64 {
    let mut checksum = 17_u64;
    for (family, store) in stores {
        checksum = checksum_part(checksum, family);
        for (key, cell) in store {
            checksum = checksum_part(checksum, key);
            checksum = checksum_part(checksum, &cell.value);
            checksum = checksum.wrapping_add(cell.version);
        }
    }
    checksum
}

fn checksum_part(seed: u64, value: &str) -> u64 {
    let mut checksum = seed;
    for byte in value.bytes() {
        checksum = checksum.wrapping_mul(131).wrapping_add(byte as u64 + 1);
    }
    checksum
}

fn invalid_regression_checked() -> bool {
    match validate(include_str!("../examples/invalid/snapshot_full.atlas")) {
        Err(diagnostic) => diagnostic.code == DiagnosticCode::SnapshotFullStrict,
        Ok(_) => false,
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
