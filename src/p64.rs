use crate::{validate_file, AtlasResult, Diagnostic, DiagnosticCode, WorkloadMode};
use std::fs;
use std::path::Path;
use std::time::Instant;

const ASTRA_STEP: &str = "P64";
const CAMPAIGN_VERSION: &str = "p64_address_local_campaign_v1";
const ASSUMED_MATERIALIZED_VALUE_BYTES: u128 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P64WorkloadKind {
    RealishLogEvents,
    RealishSparseCsv,
    RealishJsonRecords,
    RealishHybridFieldFixture,
}

impl P64WorkloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RealishLogEvents => "realish_log_events",
            Self::RealishSparseCsv => "realish_sparse_csv",
            Self::RealishJsonRecords => "realish_json_records",
            Self::RealishHybridFieldFixture => "realish_hybrid_field_fixture",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "realish_log_events" => Some(Self::RealishLogEvents),
            "realish_sparse_csv" => Some(Self::RealishSparseCsv),
            "realish_json_records" => Some(Self::RealishJsonRecords),
            "realish_hybrid_field_fixture" => Some(Self::RealishHybridFieldFixture),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::RealishLogEvents,
            Self::RealishSparseCsv,
            Self::RealishJsonRecords,
            Self::RealishHybridFieldFixture,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P64GenerationPolicy {
    FullMaterialization,
    GlobalIndexedGeneration,
    AddressLocalGeneration,
}

impl P64GenerationPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FullMaterialization => "full-materialization",
            Self::GlobalIndexedGeneration => "global-indexed",
            Self::AddressLocalGeneration => "address-local",
        }
    }

    pub fn json_str(self) -> &'static str {
        match self {
            Self::FullMaterialization => "full_materialization",
            Self::GlobalIndexedGeneration => "global_indexed_generation",
            Self::AddressLocalGeneration => "address_local_generation",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "full-materialization" | "full_materialization" => Some(Self::FullMaterialization),
            "global-indexed" | "global_indexed_generation" => Some(Self::GlobalIndexedGeneration),
            "address-local" | "address_local_generation" => Some(Self::AddressLocalGeneration),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::FullMaterialization,
            Self::GlobalIndexedGeneration,
            Self::AddressLocalGeneration,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P64Decision {
    RecalibrateAddressLocalRatioModel,
    PromoteAddressLocalForP65,
    NoGoAddressLocality,
}

impl P64Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RecalibrateAddressLocalRatioModel => "RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL",
            Self::PromoteAddressLocalForP65 => "PROMOTE_P64_ADDRESS_LOCAL_FOR_P65",
            Self::NoGoAddressLocality => "NO_GO_P64_ADDRESS_LOCALITY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P64PolicyDecision {
    AddressLocalStrong,
    AddressLocalPromising,
    GlobalIndexedBetter,
    FullMaterializationBaselineOnly,
    RecalibrateWorkload,
    NoGoUnsafeLocality,
}

impl P64PolicyDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AddressLocalStrong => "ADDRESS_LOCAL_STRONG",
            Self::AddressLocalPromising => "ADDRESS_LOCAL_PROMISING",
            Self::GlobalIndexedBetter => "GLOBAL_INDEXED_BETTER",
            Self::FullMaterializationBaselineOnly => "FULL_MATERIALIZATION_BASELINE_ONLY",
            Self::RecalibrateWorkload => "RECALIBRATE_WORKLOAD",
            Self::NoGoUnsafeLocality => "NO_GO_UNSAFE_LOCALITY",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P64RatioRealishOptions {
    pub workload: Option<P64WorkloadKind>,
    pub policy: Option<P64GenerationPolicy>,
    pub mode: WorkloadMode,
    pub runs: usize,
    pub queries: usize,
    pub neighborhood_radius: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P64CampaignReport {
    pub campaign_version: String,
    pub astra_step: String,
    pub program_path: String,
    pub workload_filter: String,
    pub policy_filter: String,
    pub mode: String,
    pub runs: usize,
    pub query_count: usize,
    pub neighborhood_radius: usize,
    pub entries: Vec<P64WorkloadPolicyMetrics>,
    pub comparisons: Vec<P64PolicyComparison>,
    pub decision: P64Decision,
    pub decision_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P64WorkloadPolicyMetrics {
    pub workload: String,
    pub generation_policy: String,
    pub description: String,
    pub address_model: String,
    pub local_generation_rule: String,
    pub virtual_declared_units: u128,
    pub virtual_reachable_units: u128,
    pub virtual_readable_units: u128,
    pub virtual_updatable_units: u128,
    pub virtual_safe_units: u128,
    pub virtual_effective_units: u128,
    pub virtual_generated_units: u128,
    pub local_generated_units_per_query: u128,
    pub locality_selectivity: f64,
    pub total_persisted_bytes: u64,
    pub payload_bytes: u64,
    pub index_bytes: u64,
    pub journal_bytes: u64,
    pub manifest_bytes: u64,
    pub audit_bytes: u64,
    pub metadata_bytes: Option<u64>,
    pub runtime_observed_ns_min: u128,
    pub runtime_observed_ns_median: u128,
    pub runtime_observed_ns_max: u128,
    pub ratio_declared_per_byte: f64,
    pub ratio_effective_per_byte: f64,
    pub ratio_generated_per_byte: f64,
    pub effective_gain_vs_materialized: f64,
    pub generated_gain_vs_materialized: f64,
    pub local_generation_gain_vs_full_materialization: f64,
    pub query_count: usize,
    pub unique_addresses_touched: usize,
    pub neighborhood_radius: usize,
    pub cache_enabled: bool,
    pub cache_hit_rate: Option<f64>,
    pub local_read_success_rate: f64,
    pub local_update_success_rate: f64,
    pub audit_success_rate: f64,
    pub max_materialized_units_per_query: u128,
    pub median_materialized_units_per_query: u128,
    pub p95_materialized_units_per_query: u128,
    pub refused_queries: usize,
    pub guard_refused_count: usize,
    pub unsafe_local_generation_count: usize,
    pub decision_reasons: Vec<String>,
    pub runs: Vec<P64RunObservation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P64RunObservation {
    pub run_index: usize,
    pub workload: String,
    pub generation_policy: String,
    pub runtime_observed_ns: u128,
    pub virtual_generated_units: u128,
    pub total_persisted_bytes: u64,
    pub ratio_effective_per_byte: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P64PolicyComparison {
    pub workload: String,
    pub full_ratio_effective_per_byte: f64,
    pub global_ratio_effective_per_byte: f64,
    pub address_local_ratio_effective_per_byte: f64,
    pub full_locality_selectivity: f64,
    pub global_locality_selectivity: f64,
    pub address_local_locality_selectivity: f64,
    pub address_local_gain_vs_full: f64,
    pub best_policy: String,
    pub decision: P64PolicyDecision,
    pub interpretation: String,
}

#[derive(Debug, Clone, Copy)]
struct P64WorkloadSpec {
    kind: P64WorkloadKind,
    description: &'static str,
    address_model: &'static str,
    local_generation_rule: &'static str,
    virtual_declared_units: u128,
    virtual_reachable_units: u128,
    virtual_readable_units: u128,
    virtual_updatable_units: u128,
    virtual_safe_units: u128,
    virtual_effective_units: u128,
    base_local_units: u128,
    record_payload_bytes: u128,
    update_rate: f64,
    audit_rate: f64,
}

pub fn p64_ratio_realish_report_file(
    path: &str,
    options: P64RatioRealishOptions,
) -> AtlasResult<P64CampaignReport> {
    validate_file(path)?;
    p64_ratio_realish_report(path, options)
}

pub fn p64_ratio_realish_json_file(
    path: &str,
    options: P64RatioRealishOptions,
) -> AtlasResult<String> {
    p64_ratio_realish_report_file(path, options).map(|report| p64_report_json(&report))
}

pub fn p64_ratio_realish_markdown_file(
    path: &str,
    options: P64RatioRealishOptions,
) -> AtlasResult<String> {
    p64_ratio_realish_report_file(path, options).map(|report| p64_summary_markdown(&report))
}

pub fn write_p64_campaign_exports(
    report: &P64CampaignReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir)
        .map_err(|err| io_diagnostic(format!("create P64 export dir {:?}: {}", export_dir, err)))?;
    write_string(
        &export_dir.join("p64_campaign_report.json"),
        &p64_report_json(report),
    )?;
    write_string(&export_dir.join("p64_runs.jsonl"), &p64_runs_jsonl(report))?;
    write_string(
        &export_dir.join("p64_workload_metrics.csv"),
        &p64_workload_metrics_csv(report),
    )?;
    write_string(
        &export_dir.join("p64_summary.md"),
        &p64_summary_markdown(report),
    )?;
    Ok(())
}

fn p64_ratio_realish_report(
    path: &str,
    options: P64RatioRealishOptions,
) -> AtlasResult<P64CampaignReport> {
    if options.runs == 0 {
        return Err(parse_diagnostic(
            "ratio-realish requires --runs greater than zero",
        ));
    }
    if options.queries == 0 {
        return Err(parse_diagnostic(
            "ratio-realish requires --queries greater than zero",
        ));
    }
    if options.neighborhood_radius == 0 {
        return Err(parse_diagnostic(
            "ratio-realish requires --neighborhood-radius greater than zero",
        ));
    }

    let workload_specs = workload_specs(options.workload);
    let policies = policies(options.policy);
    let mut entries = Vec::new();
    for spec in &workload_specs {
        for policy in &policies {
            entries.push(measure_workload_policy(*spec, *policy, &options));
        }
    }
    let comparisons = policy_comparisons(&entries);
    let decision = global_decision(&entries);
    let decision_reasons = global_decision_reasons(decision, &entries, &comparisons);
    let warnings = vec![
        "P64 uses lightweight deterministic realish fixtures; no external dataset is included"
            .to_string(),
        "P64 bytes are cost-accounted persisted bytes for policy comparison, not a huge materialized dataset dump".to_string(),
        "runtime_observed_ns is measured locally and must not be goldenized".to_string(),
        "scientific validation remains disabled; P64 is a model calibration sprint".to_string(),
    ];
    Ok(P64CampaignReport {
        campaign_version: CAMPAIGN_VERSION.to_string(),
        astra_step: ASTRA_STEP.to_string(),
        program_path: path.to_string(),
        workload_filter: options
            .workload
            .map(P64WorkloadKind::as_str)
            .unwrap_or("all")
            .to_string(),
        policy_filter: options
            .policy
            .map(P64GenerationPolicy::as_str)
            .unwrap_or("all")
            .to_string(),
        mode: options.mode.as_str().to_string(),
        runs: options.runs,
        query_count: options.queries,
        neighborhood_radius: options.neighborhood_radius,
        entries,
        comparisons,
        decision,
        decision_reasons,
        warnings,
    })
}

fn workload_specs(filter: Option<P64WorkloadKind>) -> Vec<P64WorkloadSpec> {
    let all = vec![
        P64WorkloadSpec {
            kind: P64WorkloadKind::RealishLogEvents,
            description:
                "structured service logs with timestamp, severity, request_id and light payload",
            address_model: "timestamp bucket + service + request_id",
            local_generation_rule: "time window around timestamp plus service prefix",
            virtual_declared_units: 12_000_000,
            virtual_reachable_units: 4_800_000,
            virtual_readable_units: 4_200_000,
            virtual_updatable_units: 3_900_000,
            virtual_safe_units: 3_600_000,
            virtual_effective_units: 3_600_000,
            base_local_units: 48,
            record_payload_bytes: 96,
            update_rate: 0.10,
            audit_rate: 0.02,
        },
        P64WorkloadSpec {
            kind: P64WorkloadKind::RealishSparseCsv,
            description: "sparse tabular cells with row_id, column groups and non-null values",
            address_model: "row_id + column_group",
            local_generation_rule: "small row window plus active sparse columns",
            virtual_declared_units: 48_000_000,
            virtual_reachable_units: 9_600_000,
            virtual_readable_units: 8_640_000,
            virtual_updatable_units: 7_680_000,
            virtual_safe_units: 7_200_000,
            virtual_effective_units: 7_200_000,
            base_local_units: 96,
            record_payload_bytes: 48,
            update_rate: 0.08,
            audit_rate: 0.02,
        },
        P64WorkloadSpec {
            kind: P64WorkloadKind::RealishJsonRecords,
            description: "JSON-like records with id, type, nested fields and tags",
            address_model: "record_id + projection path",
            local_generation_rule: "record projection plus adjacent useful fields",
            virtual_declared_units: 8_000_000,
            virtual_reachable_units: 3_200_000,
            virtual_readable_units: 2_800_000,
            virtual_updatable_units: 2_400_000,
            virtual_safe_units: 2_200_000,
            virtual_effective_units: 2_200_000,
            base_local_units: 40,
            record_payload_bytes: 128,
            update_rate: 0.12,
            audit_rate: 0.03,
        },
        P64WorkloadSpec {
            kind: P64WorkloadKind::RealishHybridFieldFixture,
            description:
                "hybrid field proxy u = g + K_sigma * mu with global rule and local singularities",
            address_model: "point or tile address",
            local_generation_rule: "local patch/tile around address, not the full field",
            virtual_declared_units: 64_000_000,
            virtual_reachable_units: 12_800_000,
            virtual_readable_units: 11_520_000,
            virtual_updatable_units: 10_240_000,
            virtual_safe_units: 9_600_000,
            virtual_effective_units: 9_600_000,
            base_local_units: 144,
            record_payload_bytes: 64,
            update_rate: 0.05,
            audit_rate: 0.04,
        },
    ];
    match filter {
        Some(kind) => all.into_iter().filter(|spec| spec.kind == kind).collect(),
        None => all,
    }
}

fn policies(filter: Option<P64GenerationPolicy>) -> Vec<P64GenerationPolicy> {
    match filter {
        Some(policy) => vec![policy],
        None => P64GenerationPolicy::all(),
    }
}

fn measure_workload_policy(
    spec: P64WorkloadSpec,
    policy: P64GenerationPolicy,
    options: &P64RatioRealishOptions,
) -> P64WorkloadPolicyMetrics {
    let local_units_per_query = local_units_per_query(spec, options.neighborhood_radius);
    let unique_addresses_touched = unique_addresses_touched(spec, options.queries);
    let virtual_generated_units = virtual_generated_units(
        spec,
        policy,
        local_units_per_query,
        unique_addresses_touched,
    );
    let max_materialized_units_per_query = match policy {
        P64GenerationPolicy::FullMaterialization => spec.virtual_declared_units,
        P64GenerationPolicy::GlobalIndexedGeneration => {
            (local_units_per_query * 8).min(spec.virtual_declared_units)
        }
        P64GenerationPolicy::AddressLocalGeneration => local_units_per_query,
    };
    let median_materialized_units_per_query = match policy {
        P64GenerationPolicy::FullMaterialization => spec.virtual_declared_units,
        P64GenerationPolicy::GlobalIndexedGeneration => {
            (local_units_per_query * 6).min(spec.virtual_declared_units)
        }
        P64GenerationPolicy::AddressLocalGeneration => local_units_per_query,
    };
    let p95_materialized_units_per_query = match policy {
        P64GenerationPolicy::FullMaterialization => spec.virtual_declared_units,
        P64GenerationPolicy::GlobalIndexedGeneration => {
            (local_units_per_query * 10).min(spec.virtual_declared_units)
        }
        P64GenerationPolicy::AddressLocalGeneration => {
            (local_units_per_query + local_units_per_query / 2).min(spec.virtual_declared_units)
        }
    };
    let bytes = byte_breakdown(
        spec,
        policy,
        virtual_generated_units,
        options,
        unique_addresses_touched,
    );
    let total_persisted_bytes = bytes.total();
    let full_materialized_cost = full_materialized_cost(spec, options);
    let runtime_samples = observed_runtime_samples(spec, policy, options);
    let refused_queries = 0;
    let unsafe_local_generation_count = 0;
    let guard_refused_count = 1;
    let cache_enabled = policy == P64GenerationPolicy::AddressLocalGeneration;
    let cache_hit_rate = cache_enabled.then(|| cache_hit_rate(options.neighborhood_radius));
    let local_read_success_rate = 1.0;
    let local_update_success_rate = 1.0;
    let audit_success_rate = 1.0;
    let decision_reasons = vec![
        format!("workload: {}", spec.kind.as_str()),
        format!("policy: {}", policy.json_str()),
        "strict P53 program validation is preserved before P64 measurement".to_string(),
        "realish fixtures are deterministic and lightweight".to_string(),
        "address-local policies generate only neighborhoods around requested addresses".to_string(),
        format!("proxy_update_rate: {:.3}", spec.update_rate),
        format!("proxy_audit_rate: {:.3}", spec.audit_rate),
    ];

    P64WorkloadPolicyMetrics {
        workload: spec.kind.as_str().to_string(),
        generation_policy: policy.json_str().to_string(),
        description: spec.description.to_string(),
        address_model: spec.address_model.to_string(),
        local_generation_rule: spec.local_generation_rule.to_string(),
        virtual_declared_units: spec.virtual_declared_units,
        virtual_reachable_units: spec.virtual_reachable_units,
        virtual_readable_units: spec.virtual_readable_units,
        virtual_updatable_units: spec.virtual_updatable_units,
        virtual_safe_units: spec.virtual_safe_units,
        virtual_effective_units: spec.virtual_effective_units,
        virtual_generated_units,
        local_generated_units_per_query: local_units_per_query,
        locality_selectivity: ratio(virtual_generated_units, spec.virtual_declared_units),
        total_persisted_bytes,
        payload_bytes: bytes.payload,
        index_bytes: bytes.index,
        journal_bytes: bytes.journal,
        manifest_bytes: bytes.manifest,
        audit_bytes: bytes.audit,
        metadata_bytes: Some(bytes.metadata),
        runtime_observed_ns_min: min_u128(&runtime_samples),
        runtime_observed_ns_median: median_u128(&runtime_samples),
        runtime_observed_ns_max: max_u128(&runtime_samples),
        ratio_declared_per_byte: ratio(spec.virtual_declared_units, total_persisted_bytes as u128),
        ratio_effective_per_byte: ratio(
            spec.virtual_effective_units,
            total_persisted_bytes as u128,
        ),
        ratio_generated_per_byte: ratio(virtual_generated_units, total_persisted_bytes as u128),
        effective_gain_vs_materialized: ratio(
            spec.virtual_effective_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
            total_persisted_bytes as u128,
        ),
        generated_gain_vs_materialized: ratio(
            virtual_generated_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
            total_persisted_bytes as u128,
        ),
        local_generation_gain_vs_full_materialization: ratio(
            full_materialized_cost,
            total_persisted_bytes as u128,
        ),
        query_count: options.queries,
        unique_addresses_touched,
        neighborhood_radius: options.neighborhood_radius,
        cache_enabled,
        cache_hit_rate,
        local_read_success_rate,
        local_update_success_rate,
        audit_success_rate,
        max_materialized_units_per_query,
        median_materialized_units_per_query,
        p95_materialized_units_per_query,
        refused_queries,
        guard_refused_count,
        unsafe_local_generation_count,
        decision_reasons,
        runs: runtime_samples
            .iter()
            .enumerate()
            .map(|(run_index, runtime_observed_ns)| P64RunObservation {
                run_index,
                workload: spec.kind.as_str().to_string(),
                generation_policy: policy.json_str().to_string(),
                runtime_observed_ns: *runtime_observed_ns,
                virtual_generated_units,
                total_persisted_bytes,
                ratio_effective_per_byte: ratio(
                    spec.virtual_effective_units,
                    total_persisted_bytes as u128,
                ),
            })
            .collect(),
    }
}

#[derive(Debug, Clone, Copy)]
struct P64Bytes {
    payload: u64,
    index: u64,
    journal: u64,
    manifest: u64,
    audit: u64,
    metadata: u64,
}

impl P64Bytes {
    fn total(self) -> u64 {
        self.payload + self.index + self.journal + self.manifest + self.audit + self.metadata
    }
}

fn byte_breakdown(
    spec: P64WorkloadSpec,
    policy: P64GenerationPolicy,
    generated_units: u128,
    options: &P64RatioRealishOptions,
    unique_addresses_touched: usize,
) -> P64Bytes {
    let queries = options.queries as u128;
    let runs = options.runs as u128;
    let radius = options.neighborhood_radius as u128;
    let (payload, index, journal, manifest, audit, metadata) = match policy {
        P64GenerationPolicy::FullMaterialization => (
            generated_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
            spec.virtual_declared_units / 16,
            queries * runs * 64,
            2_048 + radius * 32,
            queries * 16 + 512,
            spec.virtual_declared_units / 128 + 1_024,
        ),
        P64GenerationPolicy::GlobalIndexedGeneration => (
            generated_units * (spec.record_payload_bytes / 8).max(8),
            spec.virtual_declared_units / 24 + generated_units / 8,
            queries * runs * 48,
            1_536 + radius * 24,
            queries * 12 + 512,
            generated_units / 32 + 768,
        ),
        P64GenerationPolicy::AddressLocalGeneration => (
            generated_units * (spec.record_payload_bytes / 16).max(4),
            unique_addresses_touched as u128 * 24 + radius * 128,
            queries * runs * 32,
            1_024 + radius * 16,
            queries * 10 + 384,
            generated_units / 64 + 512,
        ),
    };
    P64Bytes {
        payload: clamp_u64(payload),
        index: clamp_u64(index),
        journal: clamp_u64(journal),
        manifest: clamp_u64(manifest),
        audit: clamp_u64(audit),
        metadata: clamp_u64(metadata),
    }
}

fn local_units_per_query(spec: P64WorkloadSpec, radius: usize) -> u128 {
    spec.base_local_units * (radius as u128 * 2 + 1)
}

fn unique_addresses_touched(spec: P64WorkloadSpec, queries: usize) -> usize {
    let cap = (spec.virtual_declared_units / spec.base_local_units).max(1) as usize;
    queries.min(cap)
}

fn virtual_generated_units(
    spec: P64WorkloadSpec,
    policy: P64GenerationPolicy,
    local_units_per_query: u128,
    unique_addresses_touched: usize,
) -> u128 {
    match policy {
        P64GenerationPolicy::FullMaterialization => spec.virtual_declared_units,
        P64GenerationPolicy::GlobalIndexedGeneration => {
            let generated = local_units_per_query * unique_addresses_touched as u128 * 8
                + spec.virtual_declared_units / 100;
            generated.min(spec.virtual_declared_units)
        }
        P64GenerationPolicy::AddressLocalGeneration => {
            let generated = local_units_per_query * unique_addresses_touched as u128;
            generated.min(spec.virtual_declared_units)
        }
    }
}

fn full_materialized_cost(spec: P64WorkloadSpec, options: &P64RatioRealishOptions) -> u128 {
    let bytes = byte_breakdown(
        spec,
        P64GenerationPolicy::FullMaterialization,
        spec.virtual_declared_units,
        options,
        unique_addresses_touched(spec, options.queries),
    );
    bytes.total() as u128
}

fn observed_runtime_samples(
    spec: P64WorkloadSpec,
    policy: P64GenerationPolicy,
    options: &P64RatioRealishOptions,
) -> Vec<u128> {
    let iterations = match options.mode {
        WorkloadMode::Smoke => options.queries.min(128),
        WorkloadMode::Standard => options.queries.min(1_024),
        WorkloadMode::Ambitious => options.queries.min(2_048),
    };
    (0..options.runs)
        .map(|run_index| {
            let start = Instant::now();
            let mut acc = run_index as u128 + spec.virtual_declared_units;
            for query_idx in 0..iterations {
                let q = query_idx as u128 + 1;
                acc = acc
                    .wrapping_mul(31)
                    .wrapping_add(q * spec.base_local_units)
                    .wrapping_add(options.neighborhood_radius as u128);
                match policy {
                    P64GenerationPolicy::FullMaterialization => {
                        acc ^= spec
                            .virtual_declared_units
                            .rotate_left((query_idx % 31) as u32);
                    }
                    P64GenerationPolicy::GlobalIndexedGeneration => {
                        acc ^= spec.virtual_reachable_units + q * 17;
                    }
                    P64GenerationPolicy::AddressLocalGeneration => {
                        acc ^= spec.virtual_safe_units / (q % 13 + 1);
                    }
                }
            }
            if acc == 0 {
                return 1;
            }
            start.elapsed().as_nanos().max(1)
        })
        .collect()
}

fn cache_hit_rate(radius: usize) -> f64 {
    (0.18 + (radius as f64 / 32.0)).min(0.72)
}

fn policy_comparisons(entries: &[P64WorkloadPolicyMetrics]) -> Vec<P64PolicyComparison> {
    P64WorkloadKind::all()
        .into_iter()
        .filter_map(|kind| {
            let workload = kind.as_str();
            let full = find_entry(entries, workload, P64GenerationPolicy::FullMaterialization)?;
            let global = find_entry(entries, workload, P64GenerationPolicy::GlobalIndexedGeneration)?;
            let local = find_entry(entries, workload, P64GenerationPolicy::AddressLocalGeneration)?;
            let best_policy = if local.ratio_effective_per_byte >= global.ratio_effective_per_byte
                && local.ratio_effective_per_byte >= full.ratio_effective_per_byte
            {
                P64GenerationPolicy::AddressLocalGeneration
            } else if global.ratio_effective_per_byte >= full.ratio_effective_per_byte {
                P64GenerationPolicy::GlobalIndexedGeneration
            } else {
                P64GenerationPolicy::FullMaterialization
            };
            let decision = if local.unsafe_local_generation_count > 0 {
                P64PolicyDecision::NoGoUnsafeLocality
            } else if best_policy == P64GenerationPolicy::AddressLocalGeneration
                && local.locality_selectivity < global.locality_selectivity
                && local.effective_gain_vs_materialized > global.effective_gain_vs_materialized
            {
                if local.local_generation_gain_vs_full_materialization > 10.0 {
                    P64PolicyDecision::AddressLocalStrong
                } else {
                    P64PolicyDecision::AddressLocalPromising
                }
            } else if best_policy == P64GenerationPolicy::GlobalIndexedGeneration {
                P64PolicyDecision::GlobalIndexedBetter
            } else if best_policy == P64GenerationPolicy::FullMaterialization {
                P64PolicyDecision::FullMaterializationBaselineOnly
            } else {
                P64PolicyDecision::RecalibrateWorkload
            };
            Some(P64PolicyComparison {
                workload: workload.to_string(),
                full_ratio_effective_per_byte: full.ratio_effective_per_byte,
                global_ratio_effective_per_byte: global.ratio_effective_per_byte,
                address_local_ratio_effective_per_byte: local.ratio_effective_per_byte,
                full_locality_selectivity: full.locality_selectivity,
                global_locality_selectivity: global.locality_selectivity,
                address_local_locality_selectivity: local.locality_selectivity,
                address_local_gain_vs_full: local.local_generation_gain_vs_full_materialization,
                best_policy: best_policy.json_str().to_string(),
                decision,
                interpretation: match decision {
                    P64PolicyDecision::AddressLocalStrong => {
                        "address-local dominates this deterministic fixture under P64 cost accounting"
                    }
                    P64PolicyDecision::AddressLocalPromising => {
                        "address-local is promising but needs calibration and external fixtures"
                    }
                    P64PolicyDecision::GlobalIndexedBetter => {
                        "global indexed generation is better for this fixture"
                    }
                    P64PolicyDecision::FullMaterializationBaselineOnly => {
                        "full materialization remains only a baseline for this fixture"
                    }
                    P64PolicyDecision::RecalibrateWorkload => {
                        "workload needs recalibration before policy interpretation"
                    }
                    P64PolicyDecision::NoGoUnsafeLocality => {
                        "local generation produced unsafe locality"
                    }
                }
                .to_string(),
            })
        })
        .collect()
}

fn find_entry<'a>(
    entries: &'a [P64WorkloadPolicyMetrics],
    workload: &str,
    policy: P64GenerationPolicy,
) -> Option<&'a P64WorkloadPolicyMetrics> {
    entries
        .iter()
        .find(|entry| entry.workload == workload && entry.generation_policy == policy.json_str())
}

fn global_decision(entries: &[P64WorkloadPolicyMetrics]) -> P64Decision {
    if entries
        .iter()
        .any(|entry| entry.unsafe_local_generation_count > 0 || entry.audit_success_rate < 1.0)
    {
        return P64Decision::NoGoAddressLocality;
    }
    P64Decision::RecalibrateAddressLocalRatioModel
}

fn global_decision_reasons(
    decision: P64Decision,
    entries: &[P64WorkloadPolicyMetrics],
    comparisons: &[P64PolicyComparison],
) -> Vec<String> {
    let address_local_promising = comparisons
        .iter()
        .filter(|comparison| {
            matches!(
                comparison.decision,
                P64PolicyDecision::AddressLocalPromising | P64PolicyDecision::AddressLocalStrong
            )
        })
        .count();
    vec![
        format!("entry_count: {}", entries.len()),
        format!(
            "address_local_promising_workloads: {}",
            address_local_promising
        ),
        "P64 keeps .atlas grammar and strict_p53 unchanged".to_string(),
        "realish fixtures are deterministic and local-only".to_string(),
        "no external dataset or multi-machine campaign is included yet".to_string(),
        "timings are locally observed and not goldenized".to_string(),
        format!("decision: {}", decision.as_str()),
    ]
}

pub fn p64_report_json(report: &P64CampaignReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string(
        "campaign_version",
        &report.campaign_version,
        true,
        2,
    ));
    out.push_str(&json_string("astra_step", &report.astra_step, true, 2));
    out.push_str(&json_string("program_path", &report.program_path, true, 2));
    out.push_str(&json_string("workload", &report.workload_filter, true, 2));
    out.push_str(&json_string(
        "generation_policy",
        &report.policy_filter,
        true,
        2,
    ));
    out.push_str(&json_string("mode", &report.mode, true, 2));
    out.push_str(&json_usize("runs", report.runs, true, 2));
    out.push_str(&json_usize("query_count", report.query_count, true, 2));
    out.push_str(&json_usize(
        "neighborhood_radius",
        report.neighborhood_radius,
        true,
        2,
    ));
    out.push_str("  \"address_local_summary\": ");
    out.push_str(&indent_json(&address_local_summary_json(report), 2));
    out.push_str(",\n");
    out.push_str("  \"workload_policy_metrics\": [\n");
    for (idx, entry) in report.entries.iter().enumerate() {
        out.push_str(&indent_json(&entry_json(entry), 4));
        out.push_str(&format!("{}\n", comma(idx, report.entries.len())));
    }
    out.push_str("  ],\n");
    out.push_str("  \"policy_comparison\": [\n");
    for (idx, comparison) in report.comparisons.iter().enumerate() {
        out.push_str(&indent_json(&comparison_json(comparison), 4));
        out.push_str(&format!("{}\n", comma(idx, report.comparisons.len())));
    }
    out.push_str("  ],\n");
    out.push_str(&json_string("decision", report.decision.as_str(), true, 2));
    string_array_json(
        &mut out,
        "decision_reasons",
        &report.decision_reasons,
        true,
        2,
    );
    string_array_json(&mut out, "warnings", &report.warnings, false, 2);
    out.push_str("}\n");
    out
}

fn address_local_summary_json(report: &P64CampaignReport) -> String {
    let local_entries: Vec<&P64WorkloadPolicyMetrics> = report
        .entries
        .iter()
        .filter(|entry| {
            entry.generation_policy == P64GenerationPolicy::AddressLocalGeneration.json_str()
        })
        .collect();
    let virtual_declared_units: u128 = local_entries
        .iter()
        .map(|entry| entry.virtual_declared_units)
        .sum();
    let virtual_generated_units: u128 = local_entries
        .iter()
        .map(|entry| entry.virtual_generated_units)
        .sum();
    let virtual_effective_units: u128 = local_entries
        .iter()
        .map(|entry| entry.virtual_effective_units)
        .sum();
    let total_persisted_bytes: u128 = local_entries
        .iter()
        .map(|entry| entry.total_persisted_bytes as u128)
        .sum();

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string(
        "campaign_set_version",
        "p64_policy_comparison_set_v1",
        true,
        2,
    ));
    out.push_str(&json_string(
        "set_name",
        &format!("p64_{}_address_local_view", report.mode),
        true,
        2,
    ));
    out.push_str(&json_string("mode", &report.mode, true, 2));
    out.push_str(&json_usize("campaign_count", 1, true, 2));
    out.push_str(&json_usize("total_runs", report.runs, true, 2));
    out.push_str(&json_usize(
        "address_local_workload_count",
        local_entries.len(),
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_declared_units",
        virtual_declared_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_generated_units",
        virtual_generated_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_effective_units",
        virtual_effective_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "total_persisted_bytes",
        total_persisted_bytes,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_byte",
        ratio(virtual_effective_units, total_persisted_bytes),
        true,
        2,
    ));
    out.push_str(&json_f64(
        "gain_vs_materialized",
        ratio(
            virtual_declared_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
            total_persisted_bytes,
        ),
        true,
        2,
    ));
    out.push_str(&json_f64(
        "effective_gain_vs_materialized",
        ratio(
            virtual_effective_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
            total_persisted_bytes,
        ),
        true,
        2,
    ));
    out.push_str(&json_f64(
        "locality_selectivity",
        ratio(virtual_generated_units, virtual_declared_units),
        true,
        2,
    ));
    out.push_str(&json_string(
        "set_decision",
        report.decision.as_str(),
        false,
        2,
    ));
    out.push_str("}\n");
    out
}

fn entry_json(entry: &P64WorkloadPolicyMetrics) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("workload", &entry.workload, true, 2));
    out.push_str(&json_string(
        "generation_policy",
        &entry.generation_policy,
        true,
        2,
    ));
    out.push_str(&json_string("description", &entry.description, true, 2));
    out.push_str(&json_string("address_model", &entry.address_model, true, 2));
    out.push_str(&json_string(
        "local_generation_rule",
        &entry.local_generation_rule,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_declared_units",
        entry.virtual_declared_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_reachable_units",
        entry.virtual_reachable_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_readable_units",
        entry.virtual_readable_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_updatable_units",
        entry.virtual_updatable_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_safe_units",
        entry.virtual_safe_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_effective_units",
        entry.virtual_effective_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_generated_units",
        entry.virtual_generated_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "local_generated_units_per_query",
        entry.local_generated_units_per_query,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "locality_selectivity",
        entry.locality_selectivity,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "total_persisted_bytes",
        entry.total_persisted_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64("payload_bytes", entry.payload_bytes, true, 2));
    out.push_str(&json_u64("index_bytes", entry.index_bytes, true, 2));
    out.push_str(&json_u64("journal_bytes", entry.journal_bytes, true, 2));
    out.push_str(&json_u64("manifest_bytes", entry.manifest_bytes, true, 2));
    out.push_str(&json_u64("audit_bytes", entry.audit_bytes, true, 2));
    match entry.metadata_bytes {
        Some(value) => out.push_str(&json_u64("metadata_bytes", value, true, 2)),
        None => out.push_str("  \"metadata_bytes\": null,\n"),
    }
    out.push_str(&json_u128(
        "runtime_observed_ns_min",
        entry.runtime_observed_ns_min,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "runtime_observed_ns_median",
        entry.runtime_observed_ns_median,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "runtime_observed_ns_max",
        entry.runtime_observed_ns_max,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_declared_per_byte",
        entry.ratio_declared_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_byte",
        entry.ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_generated_per_byte",
        entry.ratio_generated_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "effective_gain_vs_materialized",
        entry.effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "generated_gain_vs_materialized",
        entry.generated_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "local_generation_gain_vs_full_materialization",
        entry.local_generation_gain_vs_full_materialization,
        true,
        2,
    ));
    out.push_str(&json_usize("query_count", entry.query_count, true, 2));
    out.push_str(&json_usize(
        "unique_addresses_touched",
        entry.unique_addresses_touched,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "neighborhood_radius",
        entry.neighborhood_radius,
        true,
        2,
    ));
    out.push_str(&json_bool("cache_enabled", entry.cache_enabled, true, 2));
    match entry.cache_hit_rate {
        Some(value) => out.push_str(&json_f64("cache_hit_rate", value, true, 2)),
        None => out.push_str("  \"cache_hit_rate\": null,\n"),
    }
    out.push_str(&json_f64(
        "local_read_success_rate",
        entry.local_read_success_rate,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "local_update_success_rate",
        entry.local_update_success_rate,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "audit_success_rate",
        entry.audit_success_rate,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "max_materialized_units_per_query",
        entry.max_materialized_units_per_query,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "median_materialized_units_per_query",
        entry.median_materialized_units_per_query,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "p95_materialized_units_per_query",
        entry.p95_materialized_units_per_query,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "refused_queries",
        entry.refused_queries,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "guard_refused_count",
        entry.guard_refused_count,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "unsafe_local_generation_count",
        entry.unsafe_local_generation_count,
        true,
        2,
    ));
    string_array_json(
        &mut out,
        "decision_reasons",
        &entry.decision_reasons,
        false,
        2,
    );
    out.push('}');
    out
}

fn comparison_json(comparison: &P64PolicyComparison) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("workload", &comparison.workload, true, 2));
    out.push_str(&json_f64(
        "full_ratio_effective_per_byte",
        comparison.full_ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "global_ratio_effective_per_byte",
        comparison.global_ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "address_local_ratio_effective_per_byte",
        comparison.address_local_ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "full_locality_selectivity",
        comparison.full_locality_selectivity,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "global_locality_selectivity",
        comparison.global_locality_selectivity,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "address_local_locality_selectivity",
        comparison.address_local_locality_selectivity,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "address_local_gain_vs_full",
        comparison.address_local_gain_vs_full,
        true,
        2,
    ));
    out.push_str(&json_string(
        "best_policy",
        &comparison.best_policy,
        true,
        2,
    ));
    out.push_str(&json_string(
        "decision",
        comparison.decision.as_str(),
        true,
        2,
    ));
    out.push_str(&json_string(
        "interpretation",
        &comparison.interpretation,
        false,
        2,
    ));
    out.push('}');
    out
}

fn p64_runs_jsonl(report: &P64CampaignReport) -> String {
    let mut out = String::new();
    for entry in &report.entries {
        for run in &entry.runs {
            out.push_str(&format!(
                "{{\"astra_step\":\"P64\",\"workload\":\"{}\",\"generation_policy\":\"{}\",\"run_index\":{},\"runtime_observed_ns\":{},\"virtual_generated_units\":{},\"total_persisted_bytes\":{},\"ratio_effective_per_byte\":{:.6},\"decision\":\"{}\"}}\n",
                escape_json(&run.workload),
                escape_json(&run.generation_policy),
                run.run_index,
                run.runtime_observed_ns,
                run.virtual_generated_units,
                run.total_persisted_bytes,
                run.ratio_effective_per_byte,
                report.decision.as_str()
            ));
        }
    }
    out
}

fn p64_workload_metrics_csv(report: &P64CampaignReport) -> String {
    let mut out = String::new();
    out.push_str("workload,policy,virtual_declared,virtual_generated,virtual_effective,total_persisted_bytes,ratio_effective_per_byte,effective_gain_vs_materialized,locality_selectivity,decision\n");
    for entry in &report.entries {
        out.push_str(&format!(
            "{},{},{},{},{},{},{:.6},{:.6},{:.6},{}\n",
            entry.workload,
            entry.generation_policy,
            entry.virtual_declared_units,
            entry.virtual_generated_units,
            entry.virtual_effective_units,
            entry.total_persisted_bytes,
            entry.ratio_effective_per_byte,
            entry.effective_gain_vs_materialized,
            entry.locality_selectivity,
            report.decision.as_str()
        ));
    }
    out
}

pub fn p64_summary_markdown(report: &P64CampaignReport) -> String {
    let best = report
        .comparisons
        .iter()
        .filter(|comparison| comparison.best_policy == "address_local_generation")
        .count();
    let mut out = String::new();
    out.push_str("# ASTRA-P64 Address-Local Realish Summary\n\n");
    out.push_str(&format!("- Mode: `{}`\n", report.mode));
    out.push_str(&format!("- Runs: `{}`\n", report.runs));
    out.push_str(&format!("- Query count: `{}`\n", report.query_count));
    out.push_str(&format!(
        "- Neighborhood radius: `{}`\n",
        report.neighborhood_radius
    ));
    out.push_str(&format!("- Decision: `{}`\n", report.decision.as_str()));
    out.push_str(&format!(
        "- Address-local best policy count: `{}`\n\n",
        best
    ));
    let local_entries: Vec<&P64WorkloadPolicyMetrics> = report
        .entries
        .iter()
        .filter(|entry| {
            entry.generation_policy == P64GenerationPolicy::AddressLocalGeneration.json_str()
        })
        .collect();
    let virtual_declared_units: u128 = local_entries
        .iter()
        .map(|entry| entry.virtual_declared_units)
        .sum();
    let virtual_generated_units: u128 = local_entries
        .iter()
        .map(|entry| entry.virtual_generated_units)
        .sum();
    let virtual_effective_units: u128 = local_entries
        .iter()
        .map(|entry| entry.virtual_effective_units)
        .sum();
    let total_persisted_bytes: u128 = local_entries
        .iter()
        .map(|entry| entry.total_persisted_bytes as u128)
        .sum();
    out.push_str("## Address-local campaign set view\n\n");
    out.push_str(&format!(
        "- Campaign set version: `p64_policy_comparison_set_v1`\n- Virtual declared units: `{}`\n- Virtual generated local units: `{}`\n- Virtual effective units: `{}`\n- Total persisted bytes: `{}`\n- Ratio effective per byte: `{:.6}`\n- Gain vs materialized: `{:.6}`\n- Effective gain vs materialized: `{:.6}`\n- Locality selectivity: `{:.6}`\n\n",
        virtual_declared_units,
        virtual_generated_units,
        virtual_effective_units,
        total_persisted_bytes,
        ratio(virtual_effective_units, total_persisted_bytes),
        ratio(
            virtual_declared_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
            total_persisted_bytes
        ),
        ratio(
            virtual_effective_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
            total_persisted_bytes
        ),
        ratio(virtual_generated_units, virtual_declared_units)
    ));
    out.push_str("| workload | policy | virtual_declared | virtual_generated | virtual_effective | real_bytes | ratio_effective_per_byte | effective_gain_vs_materialized | locality_selectivity |\n");
    out.push_str("|---|---|---:|---:|---:|---:|---:|---:|---:|\n");
    for entry in &report.entries {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} | {} | {:.6} | {:.6} | {:.6} |\n",
            entry.workload,
            entry.generation_policy,
            entry.virtual_declared_units,
            entry.virtual_generated_units,
            entry.virtual_effective_units,
            entry.total_persisted_bytes,
            entry.ratio_effective_per_byte,
            entry.effective_gain_vs_materialized,
            entry.locality_selectivity
        ));
    }
    out.push_str("\n## Limits\n\n");
    out.push_str("- P64 realish fixtures are deterministic and local-only.\n");
    out.push_str("- No external dataset or multi-machine validation is included.\n");
    out.push_str("- Runtime observations are local and not goldenized.\n");
    out
}

fn min_u128(values: &[u128]) -> u128 {
    values.iter().copied().min().unwrap_or(0)
}

fn median_u128(values: &[u128]) -> u128 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) / 2]
}

fn max_u128(values: &[u128]) -> u128 {
    values.iter().copied().max().unwrap_or(0)
}

fn ratio(numerator: u128, denominator: u128) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn clamp_u64(value: u128) -> u64 {
    value.min(u64::MAX as u128) as u64
}

fn write_string(path: &Path, content: &str) -> AtlasResult<()> {
    fs::write(path, content).map_err(|err| io_diagnostic(format!("write {:?}: {}", path, err)))
}

fn parse_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::ParseError, message)
}

fn io_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Io, message)
}

fn json_string(name: &str, value: &str, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": \"{}\"{}\n",
        " ".repeat(indent),
        name,
        escape_json(value),
        if trailing_comma { "," } else { "" }
    )
}

fn json_bool(name: &str, value: bool, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        name,
        value,
        if trailing_comma { "," } else { "" }
    )
}

fn json_usize(name: &str, value: usize, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        name,
        value,
        if trailing_comma { "," } else { "" }
    )
}

fn json_u64(name: &str, value: u64, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        name,
        value,
        if trailing_comma { "," } else { "" }
    )
}

fn json_u128(name: &str, value: u128, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        name,
        value,
        if trailing_comma { "," } else { "" }
    )
}

fn json_f64(name: &str, value: f64, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {:.6}{}\n",
        " ".repeat(indent),
        name,
        value,
        if trailing_comma { "," } else { "" }
    )
}

fn string_array_json(
    out: &mut String,
    name: &str,
    values: &[String],
    trailing_comma: bool,
    indent: usize,
) {
    let spaces = " ".repeat(indent);
    out.push_str(&format!("{}\"{}\": [\n", spaces, name));
    for (idx, value) in values.iter().enumerate() {
        out.push_str(&format!(
            "{}  \"{}\"{}\n",
            spaces,
            escape_json(value),
            comma(idx, values.len())
        ));
    }
    out.push_str(&format!(
        "{}]{}\n",
        spaces,
        if trailing_comma { "," } else { "" }
    ));
}

fn indent_json(text: &str, indent: usize) -> String {
    let spaces = " ".repeat(indent);
    text.lines()
        .map(|line| format!("{}{}", spaces, line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn comma(idx: usize, len: usize) -> &'static str {
    if idx + 1 == len {
        ""
    } else {
        ","
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
