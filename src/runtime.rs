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
pub struct SmokeWorkload {
    pub families: Vec<String>,
    pub records: Vec<WorkloadRecord>,
    pub reads: Vec<WorkloadRead>,
    pub updates: Vec<WorkloadUpdate>,
}

impl SmokeWorkload {
    pub fn deterministic(config: &RuntimeConfig) -> Self {
        let families = select_smoke_families(config);
        let mut records = Vec::new();
        let mut reads = Vec::new();
        let mut updates = Vec::new();

        for (family_idx, family_name) in families.iter().enumerate() {
            let family = config
                .family(family_name)
                .expect("smoke families are selected from runtime config");
            for record_idx in 0..4 {
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
                if record_idx < 2 {
                    updates.push(WorkloadUpdate {
                        family: family.name.clone(),
                        key,
                        value: format!(
                            "{}:{}:{}:{}:updated",
                            family.action, family.layout, family_idx, record_idx
                        ),
                    });
                }
            }
        }

        Self {
            families,
            records,
            reads,
            updates,
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
    pub gates: P56Gates,
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
    let config = runtime_config(text)?;
    Ok(run_smoke_config_with_atlas_file(
        config,
        ATLAS_MEMORY_SOURCE.to_string(),
    ))
}

pub fn run_smoke_file(path: &str) -> AtlasResult<RuntimeMetrics> {
    let config = runtime_config_file(path)?;
    Ok(run_smoke_config_with_atlas_file(config, path.to_string()))
}

pub fn metrics_json(text: &str) -> AtlasResult<String> {
    run_smoke(text).map(|metrics| runtime_metrics_json(&metrics))
}

pub fn metrics_json_file(path: &str) -> AtlasResult<String> {
    run_smoke_file(path).map(|metrics| runtime_metrics_json(&metrics))
}

pub fn run_smoke_config(config: RuntimeConfig) -> RuntimeMetrics {
    run_smoke_config_with_atlas_file(config, ATLAS_MEMORY_SOURCE.to_string())
}

pub fn run_smoke_config_with_atlas_file(
    config: RuntimeConfig,
    atlas_file: String,
) -> RuntimeMetrics {
    let workload = SmokeWorkload::deterministic(&config);
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
        && config
            .runtime
            .get("snapshot")
            .map(|value| value.as_str())
            == Some("incremental_manifest");
    let invalid_regression_checked = invalid_regression_checked();
    let runtime_instantiated = true;
    let encode_read_update_ok = stats.encoded_segments == workload.records.len()
        && stats.read_count == workload.reads.len()
        && stats.read_success_count == workload.reads.len()
        && stats.update_count == workload.updates.len();
    let snapshot_incremental_ok = stats.snapshot_count == 1 && strict_p53_preserved;
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
        mode: "smoke".to_string(),
        atlas_version: config.version.clone(),
        families_total: config.families.len(),
        runtime_instantiated,
        strict_p53: config.strict_p53(),
        strict_p53_preserved,
        workload_families: workload.families,
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
        gates,
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
    out.push_str(&format!("  \"mode\": \"{}\",\n", escape_json(&metrics.mode)));
    out.push_str(&format!(
        "  \"atlas_version\": \"{}\",\n",
        escape_json(&metrics.atlas_version)
    ));
    out.push_str(&format!(
        "  \"families_total\": {},\n",
        metrics.families_total
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
    out.push_str(&format!("  \"rebuild_count\": {},\n", metrics.rebuild_count));
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
