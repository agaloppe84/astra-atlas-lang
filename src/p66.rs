use crate::{
    validate_file, AtlasResult, Diagnostic, DiagnosticCode, P64WorkloadKind, WorkloadMode,
};
use std::fs;
use std::path::Path;
use std::time::Instant;

const ASTRA_STEP: &str = "P66";
const CAMPAIGN_VERSION: &str = "p66_address_fiber_campaign_v1";
const ASSUMED_MATERIALIZED_VALUE_BYTES: u128 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P66FiberKind {
    LogEventFiber,
    SparseRowFiber,
    JsonRecordFiber,
    HybridFieldTileFiber,
}

impl P66FiberKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LogEventFiber => "log_event_fiber",
            Self::SparseRowFiber => "sparse_row_fiber",
            Self::JsonRecordFiber => "json_record_fiber",
            Self::HybridFieldTileFiber => "hybrid_field_tile_fiber",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::LogEventFiber,
            Self::SparseRowFiber,
            Self::JsonRecordFiber,
            Self::HybridFieldTileFiber,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberGenerationStrategy {
    PointFiberOnly,
    NeighborhoodFiber,
    ActorManagedFiber,
    ActorManagedNeighborhoodFiber,
}

impl FiberGenerationStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PointFiberOnly => "point-fiber",
            Self::NeighborhoodFiber => "neighborhood-fiber",
            Self::ActorManagedFiber => "actor-fiber",
            Self::ActorManagedNeighborhoodFiber => "actor-neighborhood-fiber",
        }
    }

    pub fn json_str(self) -> &'static str {
        match self {
            Self::PointFiberOnly => "point_fiber_only",
            Self::NeighborhoodFiber => "neighborhood_fiber",
            Self::ActorManagedFiber => "actor_managed_fiber",
            Self::ActorManagedNeighborhoodFiber => "actor_managed_neighborhood_fiber",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "point-fiber" | "point_fiber_only" => Some(Self::PointFiberOnly),
            "neighborhood-fiber" | "neighborhood_fiber" => Some(Self::NeighborhoodFiber),
            "actor-fiber" | "actor_managed_fiber" => Some(Self::ActorManagedFiber),
            "actor-neighborhood-fiber" | "actor_managed_neighborhood_fiber" => {
                Some(Self::ActorManagedNeighborhoodFiber)
            }
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::PointFiberOnly,
            Self::NeighborhoodFiber,
            Self::ActorManagedFiber,
            Self::ActorManagedNeighborhoodFiber,
        ]
    }

    fn uses_actor(self) -> bool {
        matches!(
            self,
            Self::ActorManagedFiber | Self::ActorManagedNeighborhoodFiber
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P66JournalPolicy {
    Eager,
    Lazy,
    Compact,
}

impl P66JournalPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eager => "eager",
            Self::Lazy => "lazy",
            Self::Compact => "compact",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "eager" => Some(Self::Eager),
            "lazy" => Some(Self::Lazy),
            "compact" => Some(Self::Compact),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P66Decision {
    PromoteAddressFiberArchitecture,
    RecalibrateAddressFiberModel,
    NoGoAddressFiber,
}

impl P66Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromoteAddressFiberArchitecture => "PROMOTE_P66_ADDRESS_FIBER_ARCHITECTURE",
            Self::RecalibrateAddressFiberModel => "RECALIBRATE_P66_ADDRESS_FIBER_MODEL",
            Self::NoGoAddressFiber => "NO_GO_P66_ADDRESS_FIBER",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P66FiberWorkloadDecision {
    FiberPointStrong,
    FiberNeighborhoodStrong,
    ActorFiberStrong,
    ActorFiberPromising,
    FiberOverheadTooHigh,
    AddressLocalBaselineBetter,
    NoGoFiberUnsafe,
}

impl P66FiberWorkloadDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FiberPointStrong => "FIBER_POINT_STRONG",
            Self::FiberNeighborhoodStrong => "FIBER_NEIGHBORHOOD_STRONG",
            Self::ActorFiberStrong => "ACTOR_FIBER_STRONG",
            Self::ActorFiberPromising => "ACTOR_FIBER_PROMISING",
            Self::FiberOverheadTooHigh => "FIBER_OVERHEAD_TOO_HIGH",
            Self::AddressLocalBaselineBetter => "ADDRESS_LOCAL_BASELINE_BETTER",
            Self::NoGoFiberUnsafe => "NO_GO_FIBER_UNSAFE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P66RatioFibersOptions {
    pub workload: Option<P64WorkloadKind>,
    pub fiber_strategy: Option<FiberGenerationStrategy>,
    pub mode: WorkloadMode,
    pub runs: usize,
    pub queries: usize,
    pub neighborhood_radius: usize,
    pub budget_bytes: u64,
    pub cache_enabled: bool,
    pub journal_policy: P66JournalPolicy,
    pub update_rate: Option<P66RateProfile>,
    pub audit_rate: Option<P66RateProfile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P66RateProfile {
    Low,
    Medium,
    High,
}

impl P66RateProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    fn multiplier(self) -> f64 {
        match self {
            Self::Low => 0.65,
            Self::Medium => 1.0,
            Self::High => 1.75,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct P66FiberCampaignReport {
    pub campaign_version: String,
    pub astra_step: String,
    pub program_path: String,
    pub workload_filter: String,
    pub fiber_strategy_filter: String,
    pub mode: String,
    pub runs: usize,
    pub query_count: usize,
    pub neighborhood_radius: usize,
    pub budget_bytes: u64,
    pub cache_policy: String,
    pub journal_policy: String,
    pub update_rate: String,
    pub audit_rate: String,
    pub entries: Vec<P66FiberMetrics>,
    pub comparisons: Vec<P66FiberStrategyComparison>,
    pub decision: P66Decision,
    pub decision_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddressFiber {
    pub address_id: String,
    pub base_coordinate: String,
    pub fiber_kind: String,
    pub declared_units: u128,
    pub reachable_units: u128,
    pub readable_units: u128,
    pub updatable_units: u128,
    pub safe_units: u128,
    pub effective_units: u128,
    pub generated_units: u128,
    pub payload_bytes: u64,
    pub index_bytes: u64,
    pub cache_bytes: u64,
    pub journal_bytes: u64,
    pub audit_bytes: u64,
    pub metadata_bytes: u64,
    pub actor_binding: Option<FiberActorBinding>,
    pub safety_status: String,
    pub decision_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FiberPayload {
    pub payload_kind: String,
    pub payload_units: u128,
    pub payload_bytes: u64,
    pub create_count: usize,
    pub read_count: usize,
    pub update_count: usize,
    pub delete_count: usize,
    pub audit_count: usize,
    pub compaction_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FiberActorBinding {
    pub actor_id: String,
    pub actor_strategy: String,
    pub budget_bytes: u64,
    pub budget_actions: usize,
    pub state_bytes: u64,
    pub cache_bytes: u64,
    pub index_bytes: u64,
    pub journal_bytes: u64,
    pub queue_bytes: u64,
    pub audit_bytes: u64,
    pub coordination_bytes: u64,
    pub total_actor_overhead_bytes: u64,
    pub cache_hit_rate: f64,
    pub conflict_count: usize,
    pub stale_read_count: usize,
    pub budget_refusal_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P66FiberMetrics {
    pub workload: String,
    pub fiber_kind: String,
    pub fiber_strategy: String,
    pub description: String,
    pub address_model: String,
    pub fiber_rule: String,
    pub base_address_count: usize,
    pub fiber_count: usize,
    pub virtual_declared_units: u128,
    pub virtual_generated_units: u128,
    pub virtual_effective_units: u128,
    pub total_persisted_bytes: u64,
    pub ratio_effective_per_byte: f64,
    pub effective_gain_vs_materialized: f64,
    pub locality_selectivity: f64,
    pub actor_overhead_ratio: f64,
    pub actor_net_gain: f64,
    pub cache_hit_rate: Option<f64>,
    pub conflicts: usize,
    pub stale_reads: usize,
    pub budget_refusals: usize,
    pub fiber_declared_units: u128,
    pub fiber_generated_units: u128,
    pub fiber_effective_units: u128,
    pub fiber_selectivity: f64,
    pub fiber_effective_ratio: f64,
    pub fiber_payload_bytes: u64,
    pub fiber_index_bytes: u64,
    pub fiber_cache_bytes: u64,
    pub fiber_journal_bytes: u64,
    pub fiber_audit_bytes: u64,
    pub fiber_metadata_bytes: u64,
    pub fiber_actor_bytes: u64,
    pub fiber_total_bytes: u64,
    pub fiber_ratio_effective_per_byte: f64,
    pub fiber_gain_vs_materialized: f64,
    pub fiber_update_success_rate: f64,
    pub fiber_audit_success_rate: f64,
    pub fiber_compaction_count: usize,
    pub fiber_eviction_count: usize,
    pub address_fiber_net_gain: Option<f64>,
    pub baseline_address_local_ratio_effective_per_byte: f64,
    pub baseline_address_local_effective_gain_vs_materialized: f64,
    pub create_count: usize,
    pub read_count: usize,
    pub update_count: usize,
    pub delete_count: usize,
    pub audit_count: usize,
    pub compaction_count: usize,
    pub address_fiber: AddressFiber,
    pub payload: FiberPayload,
    pub runtime_observed_ns_min: u128,
    pub runtime_observed_ns_median: u128,
    pub runtime_observed_ns_max: u128,
    pub decision: P66FiberWorkloadDecision,
    pub decision_reasons: Vec<String>,
    pub runs: Vec<P66FiberRunObservation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P66FiberRunObservation {
    pub run_index: usize,
    pub workload: String,
    pub fiber_strategy: String,
    pub runtime_observed_ns: u128,
    pub fiber_generated_units: u128,
    pub total_persisted_bytes: u64,
    pub fiber_ratio_effective_per_byte: f64,
    pub actor_overhead_ratio: f64,
    pub conflict_count: usize,
    pub stale_read_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P66FiberStrategyComparison {
    pub workload: String,
    pub point_fiber_ratio_effective_per_byte: f64,
    pub neighborhood_fiber_ratio_effective_per_byte: f64,
    pub actor_fiber_ratio_effective_per_byte: f64,
    pub actor_neighborhood_fiber_ratio_effective_per_byte: f64,
    pub point_fiber_selectivity: f64,
    pub neighborhood_fiber_selectivity: f64,
    pub actor_fiber_selectivity: f64,
    pub actor_neighborhood_fiber_selectivity: f64,
    pub best_fiber_strategy: String,
    pub actor_overhead_ratio: f64,
    pub address_fiber_net_gain: Option<f64>,
    pub decision: P66FiberWorkloadDecision,
    pub interpretation: String,
}

#[derive(Debug, Clone, Copy)]
struct P66WorkloadSpec {
    kind: P64WorkloadKind,
    fiber_kind: P66FiberKind,
    description: &'static str,
    address_model: &'static str,
    fiber_rule: &'static str,
    virtual_declared_units: u128,
    virtual_reachable_units: u128,
    virtual_readable_units: u128,
    virtual_updatable_units: u128,
    virtual_safe_units: u128,
    virtual_effective_units: u128,
    base_fiber_units: u128,
    record_payload_bytes: u128,
    update_rate: f64,
    audit_rate: f64,
}

#[derive(Debug, Clone, Copy)]
struct P66Bytes {
    payload: u64,
    index: u64,
    cache: u64,
    journal: u64,
    audit: u64,
    metadata: u64,
    actor: u64,
}

impl P66Bytes {
    fn total(self) -> u64 {
        self.payload
            + self.index
            + self.cache
            + self.journal
            + self.audit
            + self.metadata
            + self.actor
    }
}

pub fn p66_ratio_fibers_report_file(
    path: &str,
    options: P66RatioFibersOptions,
) -> AtlasResult<P66FiberCampaignReport> {
    validate_file(path)?;
    p66_ratio_fibers_report(path, options)
}

pub fn p66_ratio_fibers_json_file(
    path: &str,
    options: P66RatioFibersOptions,
) -> AtlasResult<String> {
    let report = p66_ratio_fibers_report_file(path, options)?;
    Ok(p66_report_json(&report))
}

pub fn p66_ratio_fibers_markdown_file(
    path: &str,
    options: P66RatioFibersOptions,
) -> AtlasResult<String> {
    let report = p66_ratio_fibers_report_file(path, options)?;
    Ok(p66_summary_markdown(&report))
}

pub fn write_p66_fiber_campaign_exports(
    report: &P66FiberCampaignReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    write_file(
        export_dir.join("p66_fiber_campaign_report.json"),
        &p66_report_json(report),
    )?;
    write_file(
        export_dir.join("p66_fiber_runs.jsonl"),
        &p66_runs_jsonl(report),
    )?;
    write_file(
        export_dir.join("p66_fiber_summary.md"),
        &p66_summary_markdown(report),
    )?;
    write_file(
        export_dir.join("p66_fiber_metrics.csv"),
        &p66_metrics_csv(report),
    )?;
    Ok(())
}

fn p66_ratio_fibers_report(
    path: &str,
    options: P66RatioFibersOptions,
) -> AtlasResult<P66FiberCampaignReport> {
    if options.runs == 0
        || options.queries == 0
        || options.neighborhood_radius == 0
        || options.budget_bytes == 0
    {
        return Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            "P66 requires runs, queries, neighborhood_radius and budget_bytes greater than zero",
        ));
    }

    let workloads = workload_specs(options.workload);
    let strategies = strategies(options.fiber_strategy);

    let mut entries = Vec::new();
    for spec in workloads {
        for strategy in &strategies {
            entries.push(measure_fiber_strategy(spec, *strategy, &options));
        }
    }
    let comparisons = fiber_strategy_comparisons(&entries);
    let decision = global_decision(&entries, &comparisons);
    let decision_reasons = global_decision_reasons(decision, &entries, &comparisons);

    Ok(P66FiberCampaignReport {
        campaign_version: CAMPAIGN_VERSION.to_string(),
        astra_step: ASTRA_STEP.to_string(),
        program_path: path.to_string(),
        workload_filter: options
            .workload
            .map(|kind| kind.as_str().to_string())
            .unwrap_or_else(|| "all".to_string()),
        fiber_strategy_filter: options
            .fiber_strategy
            .map(|strategy| strategy.as_str().to_string())
            .unwrap_or_else(|| "all".to_string()),
        mode: options.mode.as_str().to_string(),
        runs: options.runs,
        query_count: options.queries,
        neighborhood_radius: options.neighborhood_radius,
        budget_bytes: options.budget_bytes,
        cache_policy: if options.cache_enabled { "on" } else { "off" }.to_string(),
        journal_policy: options.journal_policy.as_str().to_string(),
        update_rate: options
            .update_rate
            .map(|rate| rate.as_str().to_string())
            .unwrap_or_else(|| "inherited".to_string()),
        audit_rate: options
            .audit_rate
            .map(|rate| rate.as_str().to_string())
            .unwrap_or_else(|| "inherited".to_string()),
        entries,
        comparisons,
        decision,
        decision_reasons,
        warnings: vec![
            "P66 introduces address-fiber accounting but still uses deterministic internal realish fixtures".to_string(),
            "all fiber cache, journal, audit, metadata and actor binding bytes are counted".to_string(),
            "timing observations are local and are not goldenized".to_string(),
            "scientific validation remains conservative until external fixtures and multi-machine repetition exist".to_string(),
        ],
    })
}

fn workload_spec(kind: P64WorkloadKind) -> P66WorkloadSpec {
    match kind {
        P64WorkloadKind::RealishLogEvents => P66WorkloadSpec {
            kind,
            fiber_kind: P66FiberKind::LogEventFiber,
            description: "log event fibers over timestamp/service/request address points",
            address_model: "timestamp bucket + service + request_id",
            fiber_rule: "F_x is the log event projection plus local service/time context",
            virtual_declared_units: 12_000_000,
            virtual_reachable_units: 4_800_000,
            virtual_readable_units: 4_200_000,
            virtual_updatable_units: 3_900_000,
            virtual_safe_units: 3_600_000,
            virtual_effective_units: 3_600_000,
            base_fiber_units: 18,
            record_payload_bytes: 96,
            update_rate: 0.10,
            audit_rate: 0.02,
        },
        P64WorkloadKind::RealishSparseCsv => P66WorkloadSpec {
            kind,
            fiber_kind: P66FiberKind::SparseRowFiber,
            description: "sparse row fibers over row_id/column_group address points",
            address_model: "row_id + column_group",
            fiber_rule: "F_x is the sparse row projection for active columns",
            virtual_declared_units: 48_000_000,
            virtual_reachable_units: 12_000_000,
            virtual_readable_units: 9_600_000,
            virtual_updatable_units: 8_400_000,
            virtual_safe_units: 7_200_000,
            virtual_effective_units: 7_200_000,
            base_fiber_units: 36,
            record_payload_bytes: 48,
            update_rate: 0.18,
            audit_rate: 0.03,
        },
        P64WorkloadKind::RealishJsonRecords => P66WorkloadSpec {
            kind,
            fiber_kind: P66FiberKind::JsonRecordFiber,
            description: "JSON record fibers over record_id/projection address points",
            address_model: "record_id + projection path",
            fiber_rule: "F_x is the projected JSON subtree plus nearby required fields",
            virtual_declared_units: 8_000_000,
            virtual_reachable_units: 3_200_000,
            virtual_readable_units: 2_800_000,
            virtual_updatable_units: 2_400_000,
            virtual_safe_units: 2_200_000,
            virtual_effective_units: 2_200_000,
            base_fiber_units: 16,
            record_payload_bytes: 128,
            update_rate: 0.12,
            audit_rate: 0.02,
        },
        P64WorkloadKind::RealishHybridFieldFixture => P66WorkloadSpec {
            kind,
            fiber_kind: P66FiberKind::HybridFieldTileFiber,
            description: "hybrid field tile fibers for local g + K_sigma * mu patches",
            address_model: "point or tile address",
            fiber_rule: "F_x is the local field patch above a tile address",
            virtual_declared_units: 64_000_000,
            virtual_reachable_units: 16_000_000,
            virtual_readable_units: 11_520_000,
            virtual_updatable_units: 10_240_000,
            virtual_safe_units: 9_600_000,
            virtual_effective_units: 9_600_000,
            base_fiber_units: 54,
            record_payload_bytes: 64,
            update_rate: 0.05,
            audit_rate: 0.04,
        },
    }
}

fn measure_fiber_strategy(
    spec: P66WorkloadSpec,
    strategy: FiberGenerationStrategy,
    options: &P66RatioFibersOptions,
) -> P66FiberMetrics {
    let base_address_count = unique_addresses_touched(spec, options.queries);
    let fiber_count = fiber_count(
        spec,
        strategy,
        options.neighborhood_radius,
        base_address_count,
    );
    let generated_units = fiber_generated_units(
        spec,
        strategy,
        options.neighborhood_radius,
        base_address_count,
    );
    let effective_units = fiber_effective_units(spec, strategy);
    let baseline = baseline_address_local_bytes(spec, options, base_address_count);
    let baseline_total = baseline.total();
    let baseline_ratio = ratio(spec.virtual_effective_units, baseline_total as u128);
    let baseline_gain = effective_gain(spec.virtual_effective_units, baseline_total);
    let actor_binding = actor_binding(spec, strategy, options, fiber_count, generated_units);
    let bytes = fiber_bytes(
        spec,
        strategy,
        options,
        generated_units,
        fiber_count,
        actor_binding.as_ref(),
    );
    let total_persisted_bytes = bytes.total();
    let actor_bytes = actor_binding
        .as_ref()
        .map(|actor| actor.total_actor_overhead_bytes)
        .unwrap_or(0);
    let ratio_effective_per_byte = ratio(effective_units, total_persisted_bytes as u128);
    let effective_gain_vs_materialized = effective_gain(effective_units, total_persisted_bytes);
    let actor_net_gain = if baseline_gain > 0.0 {
        effective_gain_vs_materialized / baseline_gain
    } else {
        0.0
    };
    let fiber_selectivity = ratio(generated_units, spec.virtual_declared_units);
    let fiber_effective_ratio = ratio(effective_units, spec.virtual_declared_units);
    let actor_overhead_ratio = ratio(actor_bytes as u128, total_persisted_bytes as u128);
    let conflicts = actor_binding
        .as_ref()
        .map(|actor| actor.conflict_count)
        .unwrap_or(0);
    let stale_reads = actor_binding
        .as_ref()
        .map(|actor| actor.stale_read_count)
        .unwrap_or(0);
    let budget_refusals = actor_binding
        .as_ref()
        .map(|actor| actor.budget_refusal_count)
        .unwrap_or(0);
    let update_count = scaled_count(
        options.queries * options.runs,
        spec.update_rate * rate_multiplier(options.update_rate),
    );
    let audit_count = scaled_count(
        options.queries * options.runs,
        spec.audit_rate * rate_multiplier(options.audit_rate),
    )
    .max(1);
    let create_count = base_address_count;
    let read_count = options.queries * options.runs;
    let delete_count = update_count / 4;
    let compaction_count = compaction_count(strategy, options, fiber_count);
    let eviction_count = eviction_count(strategy, options, fiber_count);
    let update_success = if conflicts == 0 { 1.0 } else { 0.96 };
    let audit_success = if stale_reads == 0 { 1.0 } else { 0.95 };
    let payload = FiberPayload {
        payload_kind: spec.fiber_kind.as_str().to_string(),
        payload_units: generated_units,
        payload_bytes: bytes.payload,
        create_count,
        read_count,
        update_count,
        delete_count,
        audit_count,
        compaction_count,
    };
    let address_fiber = AddressFiber {
        address_id: format!("{}:address:0", spec.kind.as_str()),
        base_coordinate: spec.address_model.to_string(),
        fiber_kind: spec.fiber_kind.as_str().to_string(),
        declared_units: spec.virtual_declared_units,
        reachable_units: spec.virtual_reachable_units,
        readable_units: spec.virtual_readable_units,
        updatable_units: spec.virtual_updatable_units,
        safe_units: spec.virtual_safe_units,
        effective_units,
        generated_units,
        payload_bytes: bytes.payload,
        index_bytes: bytes.index,
        cache_bytes: bytes.cache,
        journal_bytes: bytes.journal,
        audit_bytes: bytes.audit,
        metadata_bytes: bytes.metadata,
        actor_binding: actor_binding.clone(),
        safety_status: if conflicts == 0 && stale_reads == 0 {
            "safe".to_string()
        } else {
            "unsafe_or_recalibrate".to_string()
        },
        decision_reasons: vec![
            "address point is evaluated as a local fiber".to_string(),
            format!("fiber_strategy: {}", strategy.json_str()),
            "strict P53 validation is performed before P66 measurement".to_string(),
        ],
    };
    let runtime_samples = observed_runtime_samples(spec, strategy, options);
    let address_fiber_net_gain = Some(if baseline_gain > 0.0 {
        effective_gain_vs_materialized / baseline_gain
    } else {
        0.0
    });
    let decision = entry_decision(
        strategy,
        ratio_effective_per_byte,
        baseline_ratio,
        actor_overhead_ratio,
        conflicts,
        stale_reads,
        budget_refusals,
    );

    P66FiberMetrics {
        workload: spec.kind.as_str().to_string(),
        fiber_kind: spec.fiber_kind.as_str().to_string(),
        fiber_strategy: strategy.json_str().to_string(),
        description: spec.description.to_string(),
        address_model: spec.address_model.to_string(),
        fiber_rule: spec.fiber_rule.to_string(),
        base_address_count,
        fiber_count,
        virtual_declared_units: spec.virtual_declared_units,
        virtual_generated_units: generated_units,
        virtual_effective_units: effective_units,
        total_persisted_bytes,
        ratio_effective_per_byte,
        effective_gain_vs_materialized,
        locality_selectivity: fiber_selectivity,
        actor_overhead_ratio,
        actor_net_gain,
        cache_hit_rate: actor_binding.as_ref().map(|actor| actor.cache_hit_rate),
        conflicts,
        stale_reads,
        budget_refusals,
        fiber_declared_units: spec.virtual_declared_units,
        fiber_generated_units: generated_units,
        fiber_effective_units: effective_units,
        fiber_selectivity,
        fiber_effective_ratio,
        fiber_payload_bytes: bytes.payload,
        fiber_index_bytes: bytes.index,
        fiber_cache_bytes: bytes.cache,
        fiber_journal_bytes: bytes.journal,
        fiber_audit_bytes: bytes.audit,
        fiber_metadata_bytes: bytes.metadata,
        fiber_actor_bytes: bytes.actor,
        fiber_total_bytes: total_persisted_bytes,
        fiber_ratio_effective_per_byte: ratio_effective_per_byte,
        fiber_gain_vs_materialized: effective_gain_vs_materialized,
        fiber_update_success_rate: update_success,
        fiber_audit_success_rate: audit_success,
        fiber_compaction_count: compaction_count,
        fiber_eviction_count: eviction_count,
        address_fiber_net_gain,
        baseline_address_local_ratio_effective_per_byte: baseline_ratio,
        baseline_address_local_effective_gain_vs_materialized: baseline_gain,
        create_count,
        read_count,
        update_count,
        delete_count,
        audit_count,
        compaction_count,
        address_fiber,
        payload,
        runtime_observed_ns_min: min_u128(&runtime_samples),
        runtime_observed_ns_median: median_u128(&runtime_samples),
        runtime_observed_ns_max: max_u128(&runtime_samples),
        decision,
        decision_reasons: vec![
            format!("workload: {}", spec.kind.as_str()),
            format!("fiber_kind: {}", spec.fiber_kind.as_str()),
            format!("fiber_strategy: {}", strategy.json_str()),
            format!(
                "address_fiber_net_gain: {:.6}",
                address_fiber_net_gain.unwrap_or(0.0)
            ),
            format!("actor_overhead_ratio: {:.6}", actor_overhead_ratio),
            "fiber, cache, journal, audit and actor bytes are counted".to_string(),
        ],
        runs: runtime_samples
            .iter()
            .enumerate()
            .map(|(run_index, runtime_observed_ns)| P66FiberRunObservation {
                run_index,
                workload: spec.kind.as_str().to_string(),
                fiber_strategy: strategy.json_str().to_string(),
                runtime_observed_ns: *runtime_observed_ns,
                fiber_generated_units: generated_units,
                total_persisted_bytes,
                fiber_ratio_effective_per_byte: ratio_effective_per_byte,
                actor_overhead_ratio,
                conflict_count: conflicts,
                stale_read_count: stale_reads,
            })
            .collect(),
    }
}

fn workload_specs(filter: Option<P64WorkloadKind>) -> Vec<P66WorkloadSpec> {
    match filter {
        Some(kind) => vec![workload_spec(kind)],
        None => P64WorkloadKind::all()
            .into_iter()
            .map(workload_spec)
            .collect(),
    }
}

fn strategies(filter: Option<FiberGenerationStrategy>) -> Vec<FiberGenerationStrategy> {
    match filter {
        Some(strategy) => vec![strategy],
        None => FiberGenerationStrategy::all(),
    }
}

fn unique_addresses_touched(spec: P66WorkloadSpec, queries: usize) -> usize {
    let cap = (spec.virtual_declared_units / spec.base_fiber_units).max(1) as usize;
    queries.min(cap)
}

fn fiber_count(
    _spec: P66WorkloadSpec,
    strategy: FiberGenerationStrategy,
    radius: usize,
    base_address_count: usize,
) -> usize {
    let factor = match strategy {
        FiberGenerationStrategy::PointFiberOnly | FiberGenerationStrategy::ActorManagedFiber => 1,
        FiberGenerationStrategy::NeighborhoodFiber => radius * 2 + 1,
        FiberGenerationStrategy::ActorManagedNeighborhoodFiber => radius + 1,
    };
    base_address_count.saturating_mul(factor.max(1))
}

fn fiber_generated_units(
    spec: P66WorkloadSpec,
    strategy: FiberGenerationStrategy,
    radius: usize,
    base_address_count: usize,
) -> u128 {
    let count = fiber_count(spec, strategy, radius, base_address_count) as u128;
    (spec.base_fiber_units * count).min(spec.virtual_declared_units)
}

fn fiber_effective_units(spec: P66WorkloadSpec, strategy: FiberGenerationStrategy) -> u128 {
    let factor = match strategy {
        FiberGenerationStrategy::PointFiberOnly => 0.22,
        FiberGenerationStrategy::NeighborhoodFiber => 0.86,
        FiberGenerationStrategy::ActorManagedFiber => 0.88,
        FiberGenerationStrategy::ActorManagedNeighborhoodFiber => 1.0,
    };
    ((spec.virtual_effective_units as f64) * factor).round() as u128
}

fn baseline_address_local_bytes(
    spec: P66WorkloadSpec,
    options: &P66RatioFibersOptions,
    unique_addresses_touched: usize,
) -> P66Bytes {
    let local_units = spec.base_fiber_units * 3 * (options.neighborhood_radius as u128 * 2 + 1);
    let generated =
        (local_units * unique_addresses_touched as u128).min(spec.virtual_declared_units);
    P66Bytes {
        payload: clamp_u64(generated * (spec.record_payload_bytes / 16).max(4)),
        index: clamp_u64(
            unique_addresses_touched as u128 * 24 + options.neighborhood_radius as u128 * 128,
        ),
        cache: 0,
        journal: clamp_u64(options.queries as u128 * options.runs as u128 * 32),
        audit: clamp_u64(options.queries as u128 * 10 + 384),
        metadata: clamp_u64(generated / 64 + 512),
        actor: 0,
    }
}

fn fiber_bytes(
    spec: P66WorkloadSpec,
    strategy: FiberGenerationStrategy,
    options: &P66RatioFibersOptions,
    generated_units: u128,
    fiber_count: usize,
    actor_binding: Option<&FiberActorBinding>,
) -> P66Bytes {
    let payload_divisor = match strategy {
        FiberGenerationStrategy::PointFiberOnly => 44,
        FiberGenerationStrategy::NeighborhoodFiber => 30,
        FiberGenerationStrategy::ActorManagedFiber => 52,
        FiberGenerationStrategy::ActorManagedNeighborhoodFiber => 38,
    };
    let index_per_fiber = match strategy {
        FiberGenerationStrategy::PointFiberOnly => 8,
        FiberGenerationStrategy::NeighborhoodFiber => 14,
        FiberGenerationStrategy::ActorManagedFiber => 10,
        FiberGenerationStrategy::ActorManagedNeighborhoodFiber => 16,
    } as u128;
    let journal_per_action = match options.journal_policy {
        P66JournalPolicy::Eager => 34,
        P66JournalPolicy::Lazy => 22,
        P66JournalPolicy::Compact => 14,
    } as u128;
    let actor_bytes = actor_binding
        .map(|actor| actor.total_actor_overhead_bytes)
        .unwrap_or(0);
    let cache_bytes = if options.cache_enabled {
        match strategy {
            FiberGenerationStrategy::PointFiberOnly
            | FiberGenerationStrategy::NeighborhoodFiber => {
                clamp_u64(generated_units / 160 + fiber_count as u128 * 6)
            }
            FiberGenerationStrategy::ActorManagedFiber
            | FiberGenerationStrategy::ActorManagedNeighborhoodFiber => {
                actor_binding.map(|actor| actor.cache_bytes).unwrap_or(0)
            }
        }
    } else {
        0
    };
    P66Bytes {
        payload: clamp_u64(generated_units * (spec.record_payload_bytes / payload_divisor).max(2)),
        index: clamp_u64(
            fiber_count as u128 * index_per_fiber + options.neighborhood_radius as u128 * 96,
        ),
        cache: cache_bytes,
        journal: clamp_u64(options.queries as u128 * options.runs as u128 * journal_per_action),
        audit: clamp_u64(options.queries as u128 * 8 + fiber_count as u128 * 6 + 256),
        metadata: clamp_u64(generated_units / 96 + fiber_count as u128 * 4 + 512),
        actor: actor_bytes,
    }
}

fn actor_binding(
    spec: P66WorkloadSpec,
    strategy: FiberGenerationStrategy,
    options: &P66RatioFibersOptions,
    fiber_count: usize,
    generated_units: u128,
) -> Option<FiberActorBinding> {
    if !strategy.uses_actor() {
        return None;
    }
    let strategy_factor = match strategy {
        FiberGenerationStrategy::ActorManagedFiber => 1_u64,
        FiberGenerationStrategy::ActorManagedNeighborhoodFiber => 2_u64,
        _ => 0_u64,
    };
    let state_bytes = fiber_count as u64 * 64 * strategy_factor;
    let cache_bytes = if options.cache_enabled {
        fiber_count as u64 * 72 * strategy_factor + clamp_u64(generated_units / 512)
    } else {
        0
    };
    let index_bytes =
        fiber_count as u64 * 18 * strategy_factor + options.neighborhood_radius as u64 * 96;
    let journal_factor = match options.journal_policy {
        P66JournalPolicy::Eager => 6,
        P66JournalPolicy::Lazy => 4,
        P66JournalPolicy::Compact => 2,
    };
    let journal_bytes =
        options.queries as u64 * options.runs as u64 * journal_factor * strategy_factor;
    let queue_bytes = fiber_count as u64 * 8 * strategy_factor + options.queries as u64 / 8;
    let audit_bytes = clamp_u64(
        (options.queries as f64
            * options.runs as f64
            * spec.audit_rate
            * rate_multiplier(options.audit_rate)
            * 8.0)
            .round() as u128
            + fiber_count as u128 * 6,
    );
    let coordination_bytes = match strategy {
        FiberGenerationStrategy::ActorManagedFiber => fiber_count as u64 * 8,
        FiberGenerationStrategy::ActorManagedNeighborhoodFiber => fiber_count as u64 * 24,
        _ => 0,
    };
    let total_actor_overhead_bytes = state_bytes
        + cache_bytes
        + index_bytes
        + journal_bytes
        + queue_bytes
        + audit_bytes
        + coordination_bytes;
    let budget_refusal_count = if total_actor_overhead_bytes > options.budget_bytes {
        ((total_actor_overhead_bytes - options.budget_bytes) / options.budget_bytes.max(1)) as usize
            + 1
    } else {
        0
    };
    let cache_hit_rate = if options.cache_enabled {
        let radius_bonus = options.neighborhood_radius as f64 / 40.0;
        let journal_bonus = if options.journal_policy == P66JournalPolicy::Compact {
            0.05
        } else {
            0.0
        };
        match strategy {
            FiberGenerationStrategy::ActorManagedFiber => {
                (0.48 + radius_bonus + journal_bonus).min(0.78)
            }
            FiberGenerationStrategy::ActorManagedNeighborhoodFiber => {
                (0.55 + radius_bonus + journal_bonus).min(0.84)
            }
            _ => 0.0,
        }
    } else {
        0.0
    };
    let conflict_count = if options.budget_bytes < total_actor_overhead_bytes / 8 {
        1
    } else {
        0
    };
    let stale_read_count =
        if !options.cache_enabled && options.journal_policy == P66JournalPolicy::Lazy {
            1
        } else {
            0
        };
    Some(FiberActorBinding {
        actor_id: format!("{}::{}", spec.kind.as_str(), strategy.json_str()),
        actor_strategy: strategy.json_str().to_string(),
        budget_bytes: options.budget_bytes,
        budget_actions: options.queries * options.runs,
        state_bytes,
        cache_bytes,
        index_bytes,
        journal_bytes,
        queue_bytes,
        audit_bytes,
        coordination_bytes,
        total_actor_overhead_bytes,
        cache_hit_rate,
        conflict_count,
        stale_read_count,
        budget_refusal_count,
    })
}

fn compaction_count(
    strategy: FiberGenerationStrategy,
    options: &P66RatioFibersOptions,
    fiber_count: usize,
) -> usize {
    match options.journal_policy {
        P66JournalPolicy::Compact => match strategy {
            FiberGenerationStrategy::ActorManagedFiber
            | FiberGenerationStrategy::ActorManagedNeighborhoodFiber => fiber_count / 8 + 1,
            _ => fiber_count / 16 + 1,
        },
        P66JournalPolicy::Lazy => fiber_count / 32,
        P66JournalPolicy::Eager => 0,
    }
}

fn eviction_count(
    strategy: FiberGenerationStrategy,
    options: &P66RatioFibersOptions,
    fiber_count: usize,
) -> usize {
    if !options.cache_enabled {
        return 0;
    }
    match strategy {
        FiberGenerationStrategy::ActorManagedFiber => fiber_count / 12,
        FiberGenerationStrategy::ActorManagedNeighborhoodFiber => fiber_count / 10,
        FiberGenerationStrategy::NeighborhoodFiber => fiber_count / 20,
        FiberGenerationStrategy::PointFiberOnly => fiber_count / 40,
    }
}

fn entry_decision(
    strategy: FiberGenerationStrategy,
    ratio_effective_per_byte: f64,
    baseline_ratio: f64,
    actor_overhead_ratio: f64,
    conflicts: usize,
    stale_reads: usize,
    budget_refusals: usize,
) -> P66FiberWorkloadDecision {
    if conflicts > 0 || stale_reads > 0 || budget_refusals > 2 {
        return P66FiberWorkloadDecision::NoGoFiberUnsafe;
    }
    if ratio_effective_per_byte < baseline_ratio {
        return P66FiberWorkloadDecision::AddressLocalBaselineBetter;
    }
    match strategy {
        FiberGenerationStrategy::PointFiberOnly => P66FiberWorkloadDecision::FiberPointStrong,
        FiberGenerationStrategy::NeighborhoodFiber => {
            P66FiberWorkloadDecision::FiberNeighborhoodStrong
        }
        FiberGenerationStrategy::ActorManagedFiber
        | FiberGenerationStrategy::ActorManagedNeighborhoodFiber => {
            if actor_overhead_ratio > 0.15 {
                P66FiberWorkloadDecision::FiberOverheadTooHigh
            } else if ratio_effective_per_byte >= baseline_ratio * 1.20 {
                P66FiberWorkloadDecision::ActorFiberStrong
            } else {
                P66FiberWorkloadDecision::ActorFiberPromising
            }
        }
    }
}

fn fiber_strategy_comparisons(entries: &[P66FiberMetrics]) -> Vec<P66FiberStrategyComparison> {
    P64WorkloadKind::all()
        .into_iter()
        .filter_map(|kind| {
            let workload = kind.as_str();
            let point = find_entry(entries, workload, FiberGenerationStrategy::PointFiberOnly)?;
            let neighborhood = find_entry(entries, workload, FiberGenerationStrategy::NeighborhoodFiber)?;
            let actor_fiber = find_entry(entries, workload, FiberGenerationStrategy::ActorManagedFiber)?;
            let actor_neighborhood =
                find_entry(entries, workload, FiberGenerationStrategy::ActorManagedNeighborhoodFiber)?;
            let best = [point, neighborhood, actor_fiber, actor_neighborhood]
                .into_iter()
                .max_by(|a, b| {
                    a.fiber_ratio_effective_per_byte
                        .total_cmp(&b.fiber_ratio_effective_per_byte)
                })?;
            let decision = best.decision;
            let interpretation = if best.fiber_strategy == FiberGenerationStrategy::ActorManagedFiber.json_str()
                || best.fiber_strategy == FiberGenerationStrategy::ActorManagedNeighborhoodFiber.json_str()
            {
                "actor-managed fibers dominate this workload under the current deterministic fixture"
            } else if best.fiber_strategy == FiberGenerationStrategy::PointFiberOnly.json_str() {
                "point fiber is sufficient for this workload under the current deterministic fixture"
            } else {
                "neighborhood fiber is strongest, but actor overhead remains under review"
            };
            Some(P66FiberStrategyComparison {
                workload: workload.to_string(),
                point_fiber_ratio_effective_per_byte: point.fiber_ratio_effective_per_byte,
                neighborhood_fiber_ratio_effective_per_byte: neighborhood.fiber_ratio_effective_per_byte,
                actor_fiber_ratio_effective_per_byte: actor_fiber.fiber_ratio_effective_per_byte,
                actor_neighborhood_fiber_ratio_effective_per_byte: actor_neighborhood.fiber_ratio_effective_per_byte,
                point_fiber_selectivity: point.fiber_selectivity,
                neighborhood_fiber_selectivity: neighborhood.fiber_selectivity,
                actor_fiber_selectivity: actor_fiber.fiber_selectivity,
                actor_neighborhood_fiber_selectivity: actor_neighborhood.fiber_selectivity,
                best_fiber_strategy: best.fiber_strategy.clone(),
                actor_overhead_ratio: best.actor_overhead_ratio,
                address_fiber_net_gain: best.address_fiber_net_gain,
                decision,
                interpretation: interpretation.to_string(),
            })
        })
        .collect()
}

fn find_entry<'a>(
    entries: &'a [P66FiberMetrics],
    workload: &str,
    strategy: FiberGenerationStrategy,
) -> Option<&'a P66FiberMetrics> {
    entries
        .iter()
        .find(|entry| entry.workload == workload && entry.fiber_strategy == strategy.json_str())
}

fn global_decision(
    entries: &[P66FiberMetrics],
    comparisons: &[P66FiberStrategyComparison],
) -> P66Decision {
    if entries
        .iter()
        .all(|entry| entry.decision == P66FiberWorkloadDecision::NoGoFiberUnsafe)
    {
        return P66Decision::NoGoAddressFiber;
    }
    let strict_promote_count = comparisons
        .iter()
        .filter(|comparison| {
            comparison.best_fiber_strategy == FiberGenerationStrategy::ActorManagedFiber.json_str()
                || comparison.best_fiber_strategy
                    == FiberGenerationStrategy::ActorManagedNeighborhoodFiber.json_str()
        })
        .filter(|comparison| comparison.actor_overhead_ratio < 0.15)
        .filter(|comparison| comparison.address_fiber_net_gain.unwrap_or(0.0) > 1.20)
        .count();
    if strict_promote_count >= 4
        && entries
            .iter()
            .filter(|entry| entry.fiber_strategy.contains("actor_managed"))
            .all(|entry| {
                entry.conflicts == 0
                    && entry.stale_reads == 0
                    && entry.budget_refusals <= 1
                    && entry.compaction_count > 0
                    && entry.audit_count > 0
            })
    {
        // P66 keeps scientific decision conservative in the first address-fiber sprint.
        return P66Decision::RecalibrateAddressFiberModel;
    }
    P66Decision::RecalibrateAddressFiberModel
}

fn global_decision_reasons(
    decision: P66Decision,
    entries: &[P66FiberMetrics],
    comparisons: &[P66FiberStrategyComparison],
) -> Vec<String> {
    let best_actor_count = comparisons
        .iter()
        .filter(|comparison| {
            comparison.best_fiber_strategy == FiberGenerationStrategy::ActorManagedFiber.json_str()
                || comparison.best_fiber_strategy
                    == FiberGenerationStrategy::ActorManagedNeighborhoodFiber.json_str()
        })
        .count();
    let unsafe_count = entries
        .iter()
        .filter(|entry| entry.conflicts > 0 || entry.stale_reads > 0 || entry.budget_refusals > 2)
        .count();
    vec![
        format!("entries: {}", entries.len()),
        format!("actor_managed_best_workloads: {}", best_actor_count),
        format!("unsafe_or_rejected_entries: {}", unsafe_count),
        "AddressFiber makes address point plus local fiber explicit".to_string(),
        "update, audit and compaction counts are included in P66 metrics".to_string(),
        "PROMOTE is disabled for the first P66 address-fiber sprint pending further calibration"
            .to_string(),
        format!("decision: {}", decision.as_str()),
    ]
}

fn observed_runtime_samples(
    spec: P66WorkloadSpec,
    strategy: FiberGenerationStrategy,
    options: &P66RatioFibersOptions,
) -> Vec<u128> {
    let iterations = match options.mode {
        WorkloadMode::Smoke => options.queries.min(128),
        WorkloadMode::Standard => options.queries.min(1_024),
        WorkloadMode::Ambitious => options.queries.min(2_048),
    };
    (0..options.runs)
        .map(|run_index| {
            let start = Instant::now();
            let mut acc = spec.virtual_declared_units + run_index as u128 + 1;
            for idx in 0..iterations {
                let q = idx as u128 + 1;
                acc = acc
                    .wrapping_mul(37)
                    .wrapping_add(q * spec.base_fiber_units)
                    .wrapping_add(options.neighborhood_radius as u128);
                match strategy {
                    FiberGenerationStrategy::PointFiberOnly => acc ^= spec.virtual_reachable_units,
                    FiberGenerationStrategy::NeighborhoodFiber => {
                        acc ^= spec.virtual_readable_units + q
                    }
                    FiberGenerationStrategy::ActorManagedFiber => {
                        acc ^= spec.virtual_updatable_units + q * 3
                    }
                    FiberGenerationStrategy::ActorManagedNeighborhoodFiber => {
                        acc ^= spec.virtual_safe_units + q * 5
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

pub fn p66_report_json(report: &P66FiberCampaignReport) -> String {
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
        "fiber_strategy",
        &report.fiber_strategy_filter,
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
    out.push_str(&json_u64("budget_bytes", report.budget_bytes, true, 2));
    out.push_str(&json_string("cache_policy", &report.cache_policy, true, 2));
    out.push_str(&json_string(
        "journal_policy",
        &report.journal_policy,
        true,
        2,
    ));
    out.push_str(&json_string("update_rate", &report.update_rate, true, 2));
    out.push_str(&json_string("audit_rate", &report.audit_rate, true, 2));
    entries_json(&mut out, "entries", &report.entries, true, 2);
    comparisons_json(
        &mut out,
        "strategy_comparison",
        &report.comparisons,
        true,
        2,
    );
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

fn entries_json(
    out: &mut String,
    key: &str,
    entries: &[P66FiberMetrics],
    trailing: bool,
    indent: usize,
) {
    out.push_str(&format!("{}\"{}\": [\n", " ".repeat(indent), key));
    for (idx, entry) in entries.iter().enumerate() {
        out.push_str(&indent_json(&entry_json(entry), indent + 2));
        out.push_str(&format!("{}\n", comma(idx, entries.len())));
    }
    out.push_str(&format!(
        "{}]{}\n",
        " ".repeat(indent),
        if trailing { "," } else { "" }
    ));
}

fn entry_json(entry: &P66FiberMetrics) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("workload", &entry.workload, true, 2));
    out.push_str(&json_string("fiber_kind", &entry.fiber_kind, true, 2));
    out.push_str(&json_string(
        "fiber_strategy",
        &entry.fiber_strategy,
        true,
        2,
    ));
    out.push_str(&json_string("description", &entry.description, true, 2));
    out.push_str(&json_string("address_model", &entry.address_model, true, 2));
    out.push_str(&json_string("fiber_rule", &entry.fiber_rule, true, 2));
    out.push_str(&json_usize(
        "base_address_count",
        entry.base_address_count,
        true,
        2,
    ));
    out.push_str(&json_usize("fiber_count", entry.fiber_count, true, 2));
    out.push_str(&json_u128(
        "virtual_declared_units",
        entry.virtual_declared_units,
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
        "virtual_effective_units",
        entry.virtual_effective_units,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "total_persisted_bytes",
        entry.total_persisted_bytes,
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
        "effective_gain_vs_materialized",
        entry.effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "locality_selectivity",
        entry.locality_selectivity,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "actor_overhead_ratio",
        entry.actor_overhead_ratio,
        true,
        2,
    ));
    out.push_str(&json_f64("actor_net_gain", entry.actor_net_gain, true, 2));
    json_option_f64(&mut out, "cache_hit_rate", entry.cache_hit_rate, true, 2);
    out.push_str(&json_usize("conflicts", entry.conflicts, true, 2));
    out.push_str(&json_usize("stale_reads", entry.stale_reads, true, 2));
    out.push_str(&json_usize(
        "budget_refusals",
        entry.budget_refusals,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "fiber_declared_units",
        entry.fiber_declared_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "fiber_generated_units",
        entry.fiber_generated_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "fiber_effective_units",
        entry.fiber_effective_units,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "fiber_selectivity",
        entry.fiber_selectivity,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "fiber_effective_ratio",
        entry.fiber_effective_ratio,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "fiber_payload_bytes",
        entry.fiber_payload_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "fiber_index_bytes",
        entry.fiber_index_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "fiber_cache_bytes",
        entry.fiber_cache_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "fiber_journal_bytes",
        entry.fiber_journal_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "fiber_audit_bytes",
        entry.fiber_audit_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "fiber_metadata_bytes",
        entry.fiber_metadata_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "fiber_actor_bytes",
        entry.fiber_actor_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "fiber_total_bytes",
        entry.fiber_total_bytes,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "fiber_ratio_effective_per_byte",
        entry.fiber_ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "fiber_gain_vs_materialized",
        entry.fiber_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "fiber_update_success_rate",
        entry.fiber_update_success_rate,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "fiber_audit_success_rate",
        entry.fiber_audit_success_rate,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "fiber_compaction_count",
        entry.fiber_compaction_count,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "fiber_eviction_count",
        entry.fiber_eviction_count,
        true,
        2,
    ));
    json_option_f64(
        &mut out,
        "address_fiber_net_gain",
        entry.address_fiber_net_gain,
        true,
        2,
    );
    out.push_str(&json_f64(
        "baseline_address_local_ratio_effective_per_byte",
        entry.baseline_address_local_ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "baseline_address_local_effective_gain_vs_materialized",
        entry.baseline_address_local_effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_usize("create_count", entry.create_count, true, 2));
    out.push_str(&json_usize("read_count", entry.read_count, true, 2));
    out.push_str(&json_usize("update_count", entry.update_count, true, 2));
    out.push_str(&json_usize("delete_count", entry.delete_count, true, 2));
    out.push_str(&json_usize("audit_count", entry.audit_count, true, 2));
    out.push_str(&json_usize(
        "compaction_count",
        entry.compaction_count,
        true,
        2,
    ));
    out.push_str(&format!(
        "  \"address_fiber\": {},\n",
        indent_json(&address_fiber_json(&entry.address_fiber), 2).trim()
    ));
    out.push_str(&format!(
        "  \"payload\": {},\n",
        indent_json(&payload_json(&entry.payload), 2).trim()
    ));
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
    out.push_str(&json_string("decision", entry.decision.as_str(), true, 2));
    string_array_json(
        &mut out,
        "decision_reasons",
        &entry.decision_reasons,
        true,
        2,
    );
    runs_json(&mut out, "runs", &entry.runs, false, 2);
    out.push_str("}\n");
    out
}

fn address_fiber_json(fiber: &AddressFiber) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("address_id", &fiber.address_id, true, 2));
    out.push_str(&json_string(
        "base_coordinate",
        &fiber.base_coordinate,
        true,
        2,
    ));
    out.push_str(&json_string("fiber_kind", &fiber.fiber_kind, true, 2));
    out.push_str(&json_u128("declared_units", fiber.declared_units, true, 2));
    out.push_str(&json_u128(
        "reachable_units",
        fiber.reachable_units,
        true,
        2,
    ));
    out.push_str(&json_u128("readable_units", fiber.readable_units, true, 2));
    out.push_str(&json_u128(
        "updatable_units",
        fiber.updatable_units,
        true,
        2,
    ));
    out.push_str(&json_u128("safe_units", fiber.safe_units, true, 2));
    out.push_str(&json_u128(
        "effective_units",
        fiber.effective_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "generated_units",
        fiber.generated_units,
        true,
        2,
    ));
    out.push_str(&json_u64("payload_bytes", fiber.payload_bytes, true, 2));
    out.push_str(&json_u64("index_bytes", fiber.index_bytes, true, 2));
    out.push_str(&json_u64("cache_bytes", fiber.cache_bytes, true, 2));
    out.push_str(&json_u64("journal_bytes", fiber.journal_bytes, true, 2));
    out.push_str(&json_u64("audit_bytes", fiber.audit_bytes, true, 2));
    out.push_str(&json_u64("metadata_bytes", fiber.metadata_bytes, true, 2));
    actor_binding_json(&mut out, &fiber.actor_binding, true, 2);
    out.push_str(&json_string("safety_status", &fiber.safety_status, true, 2));
    string_array_json(
        &mut out,
        "decision_reasons",
        &fiber.decision_reasons,
        false,
        2,
    );
    out.push_str("}\n");
    out
}

fn actor_binding_json(
    out: &mut String,
    actor: &Option<FiberActorBinding>,
    trailing: bool,
    indent: usize,
) {
    match actor {
        Some(actor) => {
            out.push_str(&format!("{}\"actor_binding\": {{\n", " ".repeat(indent)));
            out.push_str(&json_string("actor_id", &actor.actor_id, true, indent + 2));
            out.push_str(&json_string(
                "actor_strategy",
                &actor.actor_strategy,
                true,
                indent + 2,
            ));
            out.push_str(&json_u64(
                "budget_bytes",
                actor.budget_bytes,
                true,
                indent + 2,
            ));
            out.push_str(&json_usize(
                "budget_actions",
                actor.budget_actions,
                true,
                indent + 2,
            ));
            out.push_str(&json_u64(
                "state_bytes",
                actor.state_bytes,
                true,
                indent + 2,
            ));
            out.push_str(&json_u64(
                "cache_bytes",
                actor.cache_bytes,
                true,
                indent + 2,
            ));
            out.push_str(&json_u64(
                "index_bytes",
                actor.index_bytes,
                true,
                indent + 2,
            ));
            out.push_str(&json_u64(
                "journal_bytes",
                actor.journal_bytes,
                true,
                indent + 2,
            ));
            out.push_str(&json_u64(
                "queue_bytes",
                actor.queue_bytes,
                true,
                indent + 2,
            ));
            out.push_str(&json_u64(
                "audit_bytes",
                actor.audit_bytes,
                true,
                indent + 2,
            ));
            out.push_str(&json_u64(
                "coordination_bytes",
                actor.coordination_bytes,
                true,
                indent + 2,
            ));
            out.push_str(&json_u64(
                "total_actor_overhead_bytes",
                actor.total_actor_overhead_bytes,
                true,
                indent + 2,
            ));
            out.push_str(&json_f64(
                "cache_hit_rate",
                actor.cache_hit_rate,
                true,
                indent + 2,
            ));
            out.push_str(&json_usize(
                "conflict_count",
                actor.conflict_count,
                true,
                indent + 2,
            ));
            out.push_str(&json_usize(
                "stale_read_count",
                actor.stale_read_count,
                true,
                indent + 2,
            ));
            out.push_str(&json_usize(
                "budget_refusal_count",
                actor.budget_refusal_count,
                false,
                indent + 2,
            ));
            out.push_str(&format!(
                "{}}}{}\n",
                " ".repeat(indent),
                if trailing { "," } else { "" }
            ));
        }
        None => out.push_str(&format!(
            "{}\"actor_binding\": null{}\n",
            " ".repeat(indent),
            if trailing { "," } else { "" }
        )),
    }
}

fn payload_json(payload: &FiberPayload) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("payload_kind", &payload.payload_kind, true, 2));
    out.push_str(&json_u128("payload_units", payload.payload_units, true, 2));
    out.push_str(&json_u64("payload_bytes", payload.payload_bytes, true, 2));
    out.push_str(&json_usize("create_count", payload.create_count, true, 2));
    out.push_str(&json_usize("read_count", payload.read_count, true, 2));
    out.push_str(&json_usize("update_count", payload.update_count, true, 2));
    out.push_str(&json_usize("delete_count", payload.delete_count, true, 2));
    out.push_str(&json_usize("audit_count", payload.audit_count, true, 2));
    out.push_str(&json_usize(
        "compaction_count",
        payload.compaction_count,
        false,
        2,
    ));
    out.push_str("}\n");
    out
}

fn runs_json(
    out: &mut String,
    key: &str,
    runs: &[P66FiberRunObservation],
    trailing: bool,
    indent: usize,
) {
    out.push_str(&format!("{}\"{}\": [\n", " ".repeat(indent), key));
    for (idx, run) in runs.iter().enumerate() {
        out.push_str(&format!("{}{{\n", " ".repeat(indent + 2)));
        out.push_str(&json_usize("run_index", run.run_index, true, indent + 4));
        out.push_str(&json_string("workload", &run.workload, true, indent + 4));
        out.push_str(&json_string(
            "fiber_strategy",
            &run.fiber_strategy,
            true,
            indent + 4,
        ));
        out.push_str(&json_u128(
            "runtime_observed_ns",
            run.runtime_observed_ns,
            true,
            indent + 4,
        ));
        out.push_str(&json_u128(
            "fiber_generated_units",
            run.fiber_generated_units,
            true,
            indent + 4,
        ));
        out.push_str(&json_u64(
            "total_persisted_bytes",
            run.total_persisted_bytes,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "fiber_ratio_effective_per_byte",
            run.fiber_ratio_effective_per_byte,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "actor_overhead_ratio",
            run.actor_overhead_ratio,
            true,
            indent + 4,
        ));
        out.push_str(&json_usize(
            "conflict_count",
            run.conflict_count,
            true,
            indent + 4,
        ));
        out.push_str(&json_usize(
            "stale_read_count",
            run.stale_read_count,
            false,
            indent + 4,
        ));
        out.push_str(&format!(
            "{}}}{}\n",
            " ".repeat(indent + 2),
            comma(idx, runs.len())
        ));
    }
    out.push_str(&format!(
        "{}]{}\n",
        " ".repeat(indent),
        if trailing { "," } else { "" }
    ));
}

fn comparisons_json(
    out: &mut String,
    key: &str,
    comparisons: &[P66FiberStrategyComparison],
    trailing: bool,
    indent: usize,
) {
    out.push_str(&format!("{}\"{}\": [\n", " ".repeat(indent), key));
    for (idx, comparison) in comparisons.iter().enumerate() {
        out.push_str(&format!("{}{{\n", " ".repeat(indent + 2)));
        out.push_str(&json_string(
            "workload",
            &comparison.workload,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "point_fiber_ratio_effective_per_byte",
            comparison.point_fiber_ratio_effective_per_byte,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "neighborhood_fiber_ratio_effective_per_byte",
            comparison.neighborhood_fiber_ratio_effective_per_byte,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "actor_fiber_ratio_effective_per_byte",
            comparison.actor_fiber_ratio_effective_per_byte,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "actor_neighborhood_fiber_ratio_effective_per_byte",
            comparison.actor_neighborhood_fiber_ratio_effective_per_byte,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "point_fiber_selectivity",
            comparison.point_fiber_selectivity,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "neighborhood_fiber_selectivity",
            comparison.neighborhood_fiber_selectivity,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "actor_fiber_selectivity",
            comparison.actor_fiber_selectivity,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "actor_neighborhood_fiber_selectivity",
            comparison.actor_neighborhood_fiber_selectivity,
            true,
            indent + 4,
        ));
        out.push_str(&json_string(
            "best_fiber_strategy",
            &comparison.best_fiber_strategy,
            true,
            indent + 4,
        ));
        out.push_str(&json_f64(
            "actor_overhead_ratio",
            comparison.actor_overhead_ratio,
            true,
            indent + 4,
        ));
        json_option_f64(
            out,
            "address_fiber_net_gain",
            comparison.address_fiber_net_gain,
            true,
            indent + 4,
        );
        out.push_str(&json_string(
            "decision",
            comparison.decision.as_str(),
            true,
            indent + 4,
        ));
        out.push_str(&json_string(
            "interpretation",
            &comparison.interpretation,
            false,
            indent + 4,
        ));
        out.push_str(&format!(
            "{}}}{}\n",
            " ".repeat(indent + 2),
            comma(idx, comparisons.len())
        ));
    }
    out.push_str(&format!(
        "{}]{}\n",
        " ".repeat(indent),
        if trailing { "," } else { "" }
    ));
}

fn p66_runs_jsonl(report: &P66FiberCampaignReport) -> String {
    let mut out = String::new();
    for entry in &report.entries {
        for run in &entry.runs {
            out.push_str(&format!(
                "{{\"astra_step\":\"P66\",\"workload\":\"{}\",\"fiber_strategy\":\"{}\",\"run_index\":{},\"fiber_generated_units\":{},\"total_persisted_bytes\":{},\"fiber_ratio_effective_per_byte\":{:.6},\"actor_overhead_ratio\":{:.6},\"conflicts\":{},\"stale_reads\":{},\"decision\":\"{}\"}}\n",
                escape_json(&run.workload),
                escape_json(&run.fiber_strategy),
                run.run_index,
                run.fiber_generated_units,
                run.total_persisted_bytes,
                run.fiber_ratio_effective_per_byte,
                run.actor_overhead_ratio,
                run.conflict_count,
                run.stale_read_count,
                entry.decision.as_str()
            ));
        }
    }
    out
}

fn p66_metrics_csv(report: &P66FiberCampaignReport) -> String {
    let mut out = String::new();
    out.push_str("workload,fiber_strategy,fiber_declared_units,fiber_generated_units,fiber_effective_units,total_persisted_bytes,fiber_ratio_effective_per_byte,fiber_selectivity,actor_overhead_ratio,address_fiber_net_gain,update_success,audit_success,compaction_count,eviction_count,conflicts,stale_reads,budget_refusals,decision\n");
    for entry in &report.entries {
        out.push_str(&format!(
            "{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{},{},{},{},{}\n",
            entry.workload,
            entry.fiber_strategy,
            entry.fiber_declared_units,
            entry.fiber_generated_units,
            entry.fiber_effective_units,
            entry.total_persisted_bytes,
            entry.fiber_ratio_effective_per_byte,
            entry.fiber_selectivity,
            entry.actor_overhead_ratio,
            entry.address_fiber_net_gain.unwrap_or(0.0),
            entry.fiber_update_success_rate,
            entry.fiber_audit_success_rate,
            entry.compaction_count,
            entry.fiber_eviction_count,
            entry.conflicts,
            entry.stale_reads,
            entry.budget_refusals,
            entry.decision.as_str()
        ));
    }
    out
}

pub fn p66_summary_markdown(report: &P66FiberCampaignReport) -> String {
    let mut out = String::new();
    out.push_str("# ASTRA-P66 Address-Fiber Local Actor Runtime\n\n");
    out.push_str(&format!("- Mode: `{}`\n", report.mode));
    out.push_str(&format!("- Workload: `{}`\n", report.workload_filter));
    out.push_str(&format!(
        "- Fiber strategy: `{}`\n",
        report.fiber_strategy_filter
    ));
    out.push_str(&format!("- Runs: `{}`\n", report.runs));
    out.push_str(&format!("- Query count: `{}`\n", report.query_count));
    out.push_str(&format!(
        "- Neighborhood radius: `{}`\n",
        report.neighborhood_radius
    ));
    out.push_str(&format!("- Decision: `{}`\n\n", report.decision.as_str()));
    out.push_str("| workload | fiber_strategy | fiber_declared | fiber_generated | fiber_effective | real_bytes | ratio | selectivity | overhead | net_gain | decision |\n");
    out.push_str("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---|\n");
    for entry in &report.entries {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} | {} | {:.6} | {:.6} | {:.6} | {:.6} | `{}` |\n",
            entry.workload,
            entry.fiber_strategy,
            entry.fiber_declared_units,
            entry.fiber_generated_units,
            entry.fiber_effective_units,
            entry.total_persisted_bytes,
            entry.fiber_ratio_effective_per_byte,
            entry.fiber_selectivity,
            entry.actor_overhead_ratio,
            entry.address_fiber_net_gain.unwrap_or(0.0),
            entry.decision.as_str()
        ));
    }
    out.push_str("\n## Strategy comparison\n\n");
    for comparison in &report.comparisons {
        out.push_str(&format!(
            "- `{}` best=`{}` net_gain={:.6} decision=`{}`\n",
            comparison.workload,
            comparison.best_fiber_strategy,
            comparison.address_fiber_net_gain.unwrap_or(0.0),
            comparison.decision.as_str()
        ));
    }
    out.push_str("\n## Warnings\n\n");
    for warning in &report.warnings {
        out.push_str(&format!("- {}\n", warning));
    }
    out
}

fn write_file(path: impl AsRef<Path>, content: &str) -> AtlasResult<()> {
    fs::write(path.as_ref(), content).map_err(|err| {
        Diagnostic::new(
            DiagnosticCode::Io,
            format!("could not write '{}': {}", path.as_ref().display(), err),
        )
    })
}

fn io_diagnostic(message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Io, message)
}

fn rate_multiplier(rate: Option<P66RateProfile>) -> f64 {
    rate.map(P66RateProfile::multiplier).unwrap_or(1.0)
}

fn scaled_count(base: usize, rate: f64) -> usize {
    ((base as f64) * rate).round().max(0.0) as usize
}

fn ratio(numerator: u128, denominator: u128) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn effective_gain(effective_units: u128, total_bytes: u64) -> f64 {
    ratio(
        effective_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
        total_bytes as u128,
    )
}

fn clamp_u64(value: u128) -> u64 {
    value.min(u64::MAX as u128) as u64
}

fn min_u128(values: &[u128]) -> u128 {
    values.iter().copied().min().unwrap_or(0)
}

fn max_u128(values: &[u128]) -> u128 {
    values.iter().copied().max().unwrap_or(0)
}

fn median_u128(values: &[u128]) -> u128 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

fn comma(idx: usize, len: usize) -> &'static str {
    if idx + 1 == len {
        ""
    } else {
        ","
    }
}

fn indent_json(json: &str, indent: usize) -> String {
    let prefix = " ".repeat(indent);
    json.lines()
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn string_array_json(
    out: &mut String,
    key: &str,
    values: &[String],
    trailing: bool,
    indent: usize,
) {
    out.push_str(&format!("{}\"{}\": [", " ".repeat(indent), key));
    for (idx, value) in values.iter().enumerate() {
        out.push_str(&format!("\"{}\"", escape_json(value)));
        if idx + 1 != values.len() {
            out.push_str(", ");
        }
    }
    out.push_str(&format!("]{}\n", if trailing { "," } else { "" }));
}

fn json_string(key: &str, value: &str, trailing: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": \"{}\"{}\n",
        " ".repeat(indent),
        key,
        escape_json(value),
        if trailing { "," } else { "" }
    )
}

fn json_usize(key: &str, value: usize, trailing: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        key,
        value,
        if trailing { "," } else { "" }
    )
}

fn json_u64(key: &str, value: u64, trailing: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        key,
        value,
        if trailing { "," } else { "" }
    )
}

fn json_u128(key: &str, value: u128, trailing: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        key,
        value,
        if trailing { "," } else { "" }
    )
}

fn json_f64(key: &str, value: f64, trailing: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {:.6}{}\n",
        " ".repeat(indent),
        key,
        value,
        if trailing { "," } else { "" }
    )
}

fn json_option_f64(out: &mut String, key: &str, value: Option<f64>, trailing: bool, indent: usize) {
    match value {
        Some(value) => out.push_str(&json_f64(key, value, trailing, indent)),
        None => out.push_str(&format!(
            "{}\"{}\": null{}\n",
            " ".repeat(indent),
            key,
            if trailing { "," } else { "" }
        )),
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
