use crate::{
    validate_file, AtlasResult, Diagnostic, DiagnosticCode, P64WorkloadKind, WorkloadMode,
};
use std::fs;
use std::path::Path;
use std::time::Instant;

const ASTRA_STEP: &str = "P65";
const CAMPAIGN_VERSION: &str = "p65_local_actor_campaign_v1";
const ASSUMED_MATERIALIZED_VALUE_BYTES: u128 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P65ActorStrategy {
    NoActorAddressLocal,
    SingleLocalActor,
    SpecializedCrudActors,
    OverAgenticStress,
}

impl P65ActorStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoActorAddressLocal => "no-actor",
            Self::SingleLocalActor => "single-local",
            Self::SpecializedCrudActors => "specialized-crud",
            Self::OverAgenticStress => "over-agentic",
        }
    }

    pub fn json_str(self) -> &'static str {
        match self {
            Self::NoActorAddressLocal => "no_actor_address_local",
            Self::SingleLocalActor => "single_local_actor",
            Self::SpecializedCrudActors => "specialized_crud_actors",
            Self::OverAgenticStress => "over_agentic_stress",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "no-actor" | "no_actor_address_local" => Some(Self::NoActorAddressLocal),
            "single-local" | "single_local_actor" => Some(Self::SingleLocalActor),
            "specialized-crud" | "specialized_crud_actors" => Some(Self::SpecializedCrudActors),
            "over-agentic" | "over_agentic_stress" => Some(Self::OverAgenticStress),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::NoActorAddressLocal,
            Self::SingleLocalActor,
            Self::SpecializedCrudActors,
            Self::OverAgenticStress,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P65JournalPolicy {
    Lazy,
    Compact,
}

impl P65JournalPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lazy => "lazy",
            Self::Compact => "compact",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "lazy" => Some(Self::Lazy),
            "compact" => Some(Self::Compact),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P65QueryLocality {
    Clustered,
    Random,
    Mixed,
}

impl P65QueryLocality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clustered => "clustered",
            Self::Random => "random",
            Self::Mixed => "mixed",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "clustered" => Some(Self::Clustered),
            "random" => Some(Self::Random),
            "mixed" => Some(Self::Mixed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P65CalibrationDecision {
    PromoteP66LocalActorArchitecture,
    RecalibrateP65ActorOverhead,
    NoGoP65ActorOverhead,
}

impl P65CalibrationDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromoteP66LocalActorArchitecture => "PROMOTE_P66_LOCAL_ACTOR_ARCHITECTURE",
            Self::RecalibrateP65ActorOverhead => "RECALIBRATE_P65_ACTOR_OVERHEAD",
            Self::NoGoP65ActorOverhead => "NO_GO_P65_ACTOR_OVERHEAD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P65Decision {
    PromoteLocalActors,
    RecalibrateActorOverhead,
    NoGoLocalActors,
}

impl P65Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromoteLocalActors => "PROMOTE_P65_LOCAL_ACTORS",
            Self::RecalibrateActorOverhead => "RECALIBRATE_P65_ACTOR_OVERHEAD",
            Self::NoGoLocalActors => "NO_GO_P65_LOCAL_ACTORS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P65ActorWorkloadDecision {
    LocalActorStrong,
    LocalActorPromising,
    LocalActorOverheadTooHigh,
    SpecializedActorsTooExpensive,
    NoActorBaselineBetter,
    NoGoActorConflicts,
}

impl P65ActorWorkloadDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalActorStrong => "LOCAL_ACTOR_STRONG",
            Self::LocalActorPromising => "LOCAL_ACTOR_PROMISING",
            Self::LocalActorOverheadTooHigh => "LOCAL_ACTOR_OVERHEAD_TOO_HIGH",
            Self::SpecializedActorsTooExpensive => "SPECIALIZED_ACTORS_TOO_EXPENSIVE",
            Self::NoActorBaselineBetter => "NO_ACTOR_BASELINE_BETTER",
            Self::NoGoActorConflicts => "NO_GO_ACTOR_CONFLICTS",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P65RatioActorsOptions {
    pub workload: Option<P64WorkloadKind>,
    pub actor_strategy: Option<P65ActorStrategy>,
    pub mode: WorkloadMode,
    pub runs: usize,
    pub queries: usize,
    pub neighborhood_radius: usize,
    pub budget_bytes: u64,
    pub cache_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P65ActorCalibrationOptions {
    pub workload: Option<P64WorkloadKind>,
    pub mode: WorkloadMode,
    pub runs: usize,
    pub queries: usize,
    pub radius_grid: Vec<usize>,
    pub budget_grid: Vec<u64>,
    pub cache_grid: Vec<bool>,
    pub journal_grid: Vec<P65JournalPolicy>,
    pub query_locality_grid: Vec<P65QueryLocality>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P65ActorCalibrationReport {
    pub astra_step: String,
    pub calibration_version: String,
    pub program_path: String,
    pub mode: String,
    pub workload_filter: String,
    pub runs: usize,
    pub query_count: usize,
    pub radius_grid: Vec<usize>,
    pub budget_grid: Vec<u64>,
    pub cache_grid: Vec<bool>,
    pub journal_grid: Vec<P65JournalPolicy>,
    pub query_locality_grid: Vec<P65QueryLocality>,
    pub configurations_tested: usize,
    pub best_by_ratio: Option<P65CalibrationConfigMetrics>,
    pub best_by_overhead: Option<P65CalibrationConfigMetrics>,
    pub best_balanced: Option<P65CalibrationConfigMetrics>,
    pub pareto_front: Vec<P65CalibrationConfigMetrics>,
    pub no_go_configs: Vec<P65CalibrationConfigMetrics>,
    pub rejected_config_count: usize,
    pub decision: P65CalibrationDecision,
    pub decision_reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub configurations: Vec<P65CalibrationConfigMetrics>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P65CalibrationConfigMetrics {
    pub config_id: String,
    pub workload: String,
    pub neighborhood_radius: usize,
    pub budget_bytes: u64,
    pub cache_policy: String,
    pub journal_policy: String,
    pub compaction_policy: String,
    pub query_locality: String,
    pub update_rate: String,
    pub audit_rate: String,
    pub ratio_effective_per_byte: f64,
    pub effective_gain_vs_materialized: f64,
    pub actor_net_gain: f64,
    pub actor_overhead_ratio: f64,
    pub actor_overhead_bytes: u64,
    pub cache_hit_rate: f64,
    pub conflicts: usize,
    pub stale_reads: usize,
    pub budget_refusal_count: usize,
    pub budget_refusal_rate: f64,
    pub generated_units_per_query: u128,
    pub bytes_per_query: f64,
    pub balanced_score: f64,
    pub promotion_candidate: bool,
    pub decision: P65CalibrationDecision,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P65ActorCampaignReport {
    pub campaign_version: String,
    pub astra_step: String,
    pub program_path: String,
    pub workload_filter: String,
    pub actor_strategy_filter: String,
    pub mode: String,
    pub runs: usize,
    pub query_count: usize,
    pub neighborhood_radius: usize,
    pub budget_bytes: u64,
    pub cache_enabled: bool,
    pub entries: Vec<P65ActorMetrics>,
    pub comparisons: Vec<P65ActorStrategyComparison>,
    pub decision: P65Decision,
    pub decision_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalActor {
    pub actor_id: String,
    pub anchor_address: String,
    pub neighborhood_radius: usize,
    pub assigned_workload: String,
    pub budget_bytes: u64,
    pub budget_actions: usize,
    pub cache_enabled: bool,
    pub journal_enabled: bool,
    pub audit_enabled: bool,
    pub compaction_enabled: bool,
    pub state_bytes: u64,
    pub cache_bytes: u64,
    pub index_bytes: u64,
    pub journal_bytes: u64,
    pub queue_bytes: u64,
    pub audit_bytes: u64,
    pub coordination_bytes: u64,
    pub total_actor_overhead_bytes: u64,
    pub action_count: usize,
    pub read_count: usize,
    pub update_count: usize,
    pub delete_count: usize,
    pub audit_count: usize,
    pub cache_hit_count: usize,
    pub cache_miss_count: usize,
    pub eviction_count: usize,
    pub compaction_count: usize,
    pub conflict_count: usize,
    pub stale_read_count: usize,
    pub budget_refusal_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P65ActorMetrics {
    pub workload: String,
    pub actor_strategy: String,
    pub description: String,
    pub virtual_declared_units: u128,
    pub virtual_reachable_units: u128,
    pub virtual_readable_units: u128,
    pub virtual_updatable_units: u128,
    pub virtual_safe_units: u128,
    pub virtual_effective_units: u128,
    pub virtual_generated_local_units: u128,
    pub locality_selectivity: f64,
    pub baseline_no_actor_persisted_bytes: u64,
    pub total_persisted_bytes: u64,
    pub payload_bytes: u64,
    pub index_bytes: u64,
    pub journal_bytes: u64,
    pub manifest_bytes: u64,
    pub audit_bytes: u64,
    pub metadata_bytes: u64,
    pub actor_count: usize,
    pub actor_state_bytes: u64,
    pub actor_cache_bytes: u64,
    pub actor_index_bytes: u64,
    pub actor_journal_bytes: u64,
    pub actor_queue_bytes: u64,
    pub actor_audit_bytes: u64,
    pub actor_coordination_bytes: u64,
    pub total_actor_overhead_bytes: u64,
    pub actor_overhead_ratio: f64,
    pub cache_hit_rate: Option<f64>,
    pub actor_action_count: usize,
    pub coordination_events: usize,
    pub stale_read_count: usize,
    pub conflict_count: usize,
    pub eviction_count: usize,
    pub compaction_count: usize,
    pub budget_refusal_count: usize,
    pub ratio_effective_per_byte: f64,
    pub gain_vs_materialized: f64,
    pub effective_gain_vs_materialized: f64,
    pub actor_net_gain: f64,
    pub actor_ratio_delta: f64,
    pub actor_bytes_delta: i128,
    pub runtime_observed_ns_min: u128,
    pub runtime_observed_ns_median: u128,
    pub runtime_observed_ns_max: u128,
    pub local_actor: Option<LocalActor>,
    pub decision: P65ActorWorkloadDecision,
    pub decision_reasons: Vec<String>,
    pub runs: Vec<P65ActorRunObservation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P65ActorRunObservation {
    pub run_index: usize,
    pub workload: String,
    pub actor_strategy: String,
    pub runtime_observed_ns: u128,
    pub total_persisted_bytes: u64,
    pub total_actor_overhead_bytes: u64,
    pub ratio_effective_per_byte: f64,
    pub cache_hit_rate: Option<f64>,
    pub conflict_count: usize,
    pub stale_read_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P65ActorStrategyComparison {
    pub workload: String,
    pub baseline_ratio_effective_per_byte: f64,
    pub single_local_actor_ratio_effective_per_byte: f64,
    pub specialized_crud_actors_ratio_effective_per_byte: f64,
    pub over_agentic_stress_ratio_effective_per_byte: f64,
    pub baseline_effective_gain_vs_materialized: f64,
    pub single_local_actor_effective_gain_vs_materialized: f64,
    pub specialized_crud_actors_effective_gain_vs_materialized: f64,
    pub over_agentic_stress_effective_gain_vs_materialized: f64,
    pub best_actor_strategy: String,
    pub actor_net_gain: f64,
    pub actor_overhead_ratio: f64,
    pub cache_hit_rate: Option<f64>,
    pub conflicts: usize,
    pub stale_reads: usize,
    pub decision: P65ActorWorkloadDecision,
    pub interpretation: String,
}

#[derive(Debug, Clone, Copy)]
struct P65WorkloadSpec {
    kind: P64WorkloadKind,
    description: &'static str,
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

#[derive(Debug, Clone, Copy)]
struct BaseAddressLocalCost {
    payload_bytes: u64,
    index_bytes: u64,
    journal_bytes: u64,
    manifest_bytes: u64,
    audit_bytes: u64,
    metadata_bytes: u64,
}

impl BaseAddressLocalCost {
    fn total(self) -> u64 {
        self.payload_bytes
            + self.index_bytes
            + self.journal_bytes
            + self.manifest_bytes
            + self.audit_bytes
            + self.metadata_bytes
    }
}

pub fn p65_ratio_actors_report_file(
    path: &str,
    options: P65RatioActorsOptions,
) -> AtlasResult<P65ActorCampaignReport> {
    validate_file(path)?;
    p65_ratio_actors_report(path, options)
}

pub fn p65_ratio_actors_json_file(
    path: &str,
    options: P65RatioActorsOptions,
) -> AtlasResult<String> {
    let report = p65_ratio_actors_report_file(path, options)?;
    Ok(p65_report_json(&report))
}

pub fn p65_ratio_actors_markdown_file(
    path: &str,
    options: P65RatioActorsOptions,
) -> AtlasResult<String> {
    let report = p65_ratio_actors_report_file(path, options)?;
    Ok(p65_summary_markdown(&report))
}

fn p65_ratio_actors_report(
    path: &str,
    options: P65RatioActorsOptions,
) -> AtlasResult<P65ActorCampaignReport> {
    if options.runs == 0 || options.queries == 0 || options.neighborhood_radius == 0 {
        return Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            "P65 requires runs, queries and neighborhood_radius greater than zero",
        ));
    }
    if options.budget_bytes == 0 {
        return Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            "P65 requires budget_bytes greater than zero",
        ));
    }

    let workloads = match options.workload {
        Some(kind) => vec![kind],
        None => P64WorkloadKind::all(),
    };
    let strategies = match options.actor_strategy {
        Some(strategy) => vec![strategy],
        None => P65ActorStrategy::all(),
    };

    let mut entries = Vec::new();
    for workload in workloads {
        let spec = workload_spec(workload);
        for strategy in &strategies {
            entries.push(measure_actor_strategy(spec, *strategy, &options));
        }
    }
    let comparisons = actor_strategy_comparisons(&entries);
    let decision = global_p65_decision(&comparisons);
    let decision_reasons = global_decision_reasons(decision, &entries, &comparisons);

    Ok(P65ActorCampaignReport {
        campaign_version: CAMPAIGN_VERSION.to_string(),
        astra_step: ASTRA_STEP.to_string(),
        program_path: path.to_string(),
        workload_filter: options
            .workload
            .map(|kind| kind.as_str().to_string())
            .unwrap_or_else(|| "all".to_string()),
        actor_strategy_filter: options
            .actor_strategy
            .map(|strategy| strategy.as_str().to_string())
            .unwrap_or_else(|| "all".to_string()),
        mode: options.mode.as_str().to_string(),
        runs: options.runs,
        query_count: options.queries,
        neighborhood_radius: options.neighborhood_radius,
        budget_bytes: options.budget_bytes,
        cache_enabled: options.cache_enabled,
        entries,
        comparisons,
        decision,
        decision_reasons,
        warnings: vec![
            "P65 local actors are deterministic budgeted runtime fixtures, not autonomous agents"
                .to_string(),
            "all actor state, cache, journal, queue, audit and coordination bytes are counted as real cost"
                .to_string(),
            "timing observations are local and are not goldenized".to_string(),
            "scientific validation remains disabled until external fixtures and calibrated thresholds exist"
                .to_string(),
        ],
    })
}

fn workload_spec(kind: P64WorkloadKind) -> P65WorkloadSpec {
    match kind {
        P64WorkloadKind::RealishLogEvents => P65WorkloadSpec {
            kind,
            description: "structured service logs with local actor cache around timestamp/service neighborhoods",
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
        P64WorkloadKind::RealishSparseCsv => P65WorkloadSpec {
            kind,
            description: "sparse rows and column groups with local actors around active row windows",
            virtual_declared_units: 48_000_000,
            virtual_reachable_units: 12_000_000,
            virtual_readable_units: 9_600_000,
            virtual_updatable_units: 8_400_000,
            virtual_safe_units: 7_200_000,
            virtual_effective_units: 7_200_000,
            base_local_units: 96,
            record_payload_bytes: 48,
            update_rate: 0.18,
            audit_rate: 0.03,
        },
        P64WorkloadKind::RealishJsonRecords => P65WorkloadSpec {
            kind,
            description: "JSON-like records with local projection actors",
            virtual_declared_units: 8_000_000,
            virtual_reachable_units: 3_200_000,
            virtual_readable_units: 2_800_000,
            virtual_updatable_units: 2_400_000,
            virtual_safe_units: 2_200_000,
            virtual_effective_units: 2_200_000,
            base_local_units: 40,
            record_payload_bytes: 128,
            update_rate: 0.12,
            audit_rate: 0.02,
        },
        P64WorkloadKind::RealishHybridFieldFixture => P65WorkloadSpec {
            kind,
            description: "hybrid field proxy with local tile actors for g + K_sigma * mu patches",
            virtual_declared_units: 64_000_000,
            virtual_reachable_units: 16_000_000,
            virtual_readable_units: 11_520_000,
            virtual_updatable_units: 10_240_000,
            virtual_safe_units: 9_600_000,
            virtual_effective_units: 9_600_000,
            base_local_units: 144,
            record_payload_bytes: 64,
            update_rate: 0.05,
            audit_rate: 0.04,
        },
    }
}

fn measure_actor_strategy(
    spec: P65WorkloadSpec,
    strategy: P65ActorStrategy,
    options: &P65RatioActorsOptions,
) -> P65ActorMetrics {
    let local_units_per_query = local_units_per_query(spec, options.neighborhood_radius);
    let unique_addresses_touched = unique_addresses_touched(spec, options.queries);
    let virtual_generated_local_units =
        (local_units_per_query * unique_addresses_touched as u128).min(spec.virtual_declared_units);
    let locality_selectivity = ratio(virtual_generated_local_units, spec.virtual_declared_units);
    let baseline_cost = base_address_local_cost(
        spec,
        virtual_generated_local_units,
        options,
        unique_addresses_touched,
    );
    let baseline_total = baseline_cost.total();
    let baseline_gain = effective_gain(spec.virtual_effective_units, baseline_total);
    let baseline_ratio = ratio(spec.virtual_effective_units, baseline_total as u128);
    let actor = local_actor(
        spec,
        strategy,
        options,
        unique_addresses_touched,
        virtual_generated_local_units,
    );
    let actor_overhead = actor
        .as_ref()
        .map(|actor| actor.total_actor_overhead_bytes)
        .unwrap_or(0);
    let adjusted_cost = adjusted_persisted_cost(baseline_cost, strategy, actor_overhead);
    let total_persisted_bytes = adjusted_cost.total();
    let ratio_effective_per_byte =
        ratio(spec.virtual_effective_units, total_persisted_bytes as u128);
    let effective_gain_vs_materialized =
        effective_gain(spec.virtual_effective_units, total_persisted_bytes);
    let gain_vs_materialized = ratio(
        spec.virtual_declared_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
        total_persisted_bytes as u128,
    );
    let actor_net_gain = if baseline_gain > 0.0 {
        effective_gain_vs_materialized / baseline_gain
    } else {
        0.0
    };
    let actor_ratio_delta = ratio_effective_per_byte - baseline_ratio;
    let actor_bytes_delta = total_persisted_bytes as i128 - baseline_total as i128;
    let runtime_samples = observed_runtime_samples(spec, strategy, options);

    let (
        actor_count,
        actor_state_bytes,
        actor_cache_bytes,
        actor_index_bytes,
        actor_journal_bytes,
        actor_queue_bytes,
        actor_audit_bytes,
        actor_coordination_bytes,
        cache_hit_rate,
        actor_action_count,
        coordination_events,
        stale_read_count,
        conflict_count,
        eviction_count,
        compaction_count,
        budget_refusal_count,
    ) = actor
        .as_ref()
        .map(|actor| {
            (
                actor_count_for_strategy(strategy, unique_addresses_touched),
                actor.state_bytes,
                actor.cache_bytes,
                actor.index_bytes,
                actor.journal_bytes,
                actor.queue_bytes,
                actor.audit_bytes,
                actor.coordination_bytes,
                cache_hit_rate_for_actor(actor),
                actor.action_count,
                actor.coordination_bytes as usize / 64,
                actor.stale_read_count,
                actor.conflict_count,
                actor.eviction_count,
                actor.compaction_count,
                actor.budget_refusal_count,
            )
        })
        .unwrap_or((0, 0, 0, 0, 0, 0, 0, 0, None, 0, 0, 0, 0, 0, 0, 0));

    let decision = strategy_decision(
        strategy,
        actor_net_gain,
        actor_overhead,
        total_persisted_bytes,
        conflict_count,
        stale_read_count,
    );
    let decision_reasons = vec![
        format!("workload: {}", spec.kind.as_str()),
        format!("actor_strategy: {}", strategy.json_str()),
        "LocalActor is counted as memory, queue, journal, audit and coordination cost".to_string(),
        format!("actor_net_gain: {:.6}", actor_net_gain),
        format!("actor_overhead_bytes: {}", actor_overhead),
        format!("budget_bytes: {}", options.budget_bytes),
    ];

    P65ActorMetrics {
        workload: spec.kind.as_str().to_string(),
        actor_strategy: strategy.json_str().to_string(),
        description: spec.description.to_string(),
        virtual_declared_units: spec.virtual_declared_units,
        virtual_reachable_units: spec.virtual_reachable_units,
        virtual_readable_units: spec.virtual_readable_units,
        virtual_updatable_units: spec.virtual_updatable_units,
        virtual_safe_units: spec.virtual_safe_units,
        virtual_effective_units: spec.virtual_effective_units,
        virtual_generated_local_units,
        locality_selectivity,
        baseline_no_actor_persisted_bytes: baseline_total,
        total_persisted_bytes,
        payload_bytes: adjusted_cost.payload_bytes,
        index_bytes: adjusted_cost.index_bytes,
        journal_bytes: adjusted_cost.journal_bytes,
        manifest_bytes: adjusted_cost.manifest_bytes,
        audit_bytes: adjusted_cost.audit_bytes,
        metadata_bytes: adjusted_cost.metadata_bytes,
        actor_count,
        actor_state_bytes,
        actor_cache_bytes,
        actor_index_bytes,
        actor_journal_bytes,
        actor_queue_bytes,
        actor_audit_bytes,
        actor_coordination_bytes,
        total_actor_overhead_bytes: actor_overhead,
        actor_overhead_ratio: ratio(actor_overhead as u128, total_persisted_bytes as u128),
        cache_hit_rate,
        actor_action_count,
        coordination_events,
        stale_read_count,
        conflict_count,
        eviction_count,
        compaction_count,
        budget_refusal_count,
        ratio_effective_per_byte,
        gain_vs_materialized,
        effective_gain_vs_materialized,
        actor_net_gain,
        actor_ratio_delta,
        actor_bytes_delta,
        runtime_observed_ns_min: min_u128(&runtime_samples),
        runtime_observed_ns_median: median_u128(&runtime_samples),
        runtime_observed_ns_max: max_u128(&runtime_samples),
        local_actor: actor,
        decision,
        decision_reasons,
        runs: runtime_samples
            .iter()
            .enumerate()
            .map(|(run_index, runtime_observed_ns)| P65ActorRunObservation {
                run_index,
                workload: spec.kind.as_str().to_string(),
                actor_strategy: strategy.json_str().to_string(),
                runtime_observed_ns: *runtime_observed_ns,
                total_persisted_bytes,
                total_actor_overhead_bytes: actor_overhead,
                ratio_effective_per_byte,
                cache_hit_rate,
                conflict_count,
                stale_read_count,
            })
            .collect(),
    }
}

fn local_actor(
    spec: P65WorkloadSpec,
    strategy: P65ActorStrategy,
    options: &P65RatioActorsOptions,
    unique_addresses_touched: usize,
    virtual_generated_local_units: u128,
) -> Option<LocalActor> {
    if strategy == P65ActorStrategy::NoActorAddressLocal {
        return None;
    }
    let actor_count = actor_count_for_strategy(strategy, unique_addresses_touched).max(1);
    let strategy_multiplier = match strategy {
        P65ActorStrategy::NoActorAddressLocal => 0,
        P65ActorStrategy::SingleLocalActor => 1,
        P65ActorStrategy::SpecializedCrudActors => 4,
        P65ActorStrategy::OverAgenticStress => 12,
    } as u64;
    let generated_units = clamp_u64(virtual_generated_local_units);
    let state_bytes = actor_count as u64 * 512 * strategy_multiplier;
    let cache_bytes = if options.cache_enabled {
        actor_count as u64 * 1024 + generated_units / 128
    } else {
        0
    } * strategy_multiplier.max(1);
    let index_bytes = actor_count as u64 * 256 * strategy_multiplier
        + unique_addresses_touched as u64 * 16 * strategy_multiplier.max(1);
    let journal_bytes =
        (options.queries as u64 * options.runs as u64 * 18 * strategy_multiplier.max(1)).max(1);
    let queue_bytes = actor_count as u64 * 128 * strategy_multiplier
        + options.queries as u64 * strategy_multiplier.max(1) / 3;
    let audit_bytes = options.queries as u64 * 8 * strategy_multiplier.max(1)
        + actor_count as u64 * 96 * strategy_multiplier;
    let coordination_bytes = match strategy {
        P65ActorStrategy::NoActorAddressLocal => 0,
        P65ActorStrategy::SingleLocalActor => actor_count as u64 * 64,
        P65ActorStrategy::SpecializedCrudActors => actor_count as u64 * 512,
        P65ActorStrategy::OverAgenticStress => actor_count as u64 * 2048,
    };
    let total_actor_overhead_bytes = state_bytes
        + cache_bytes
        + index_bytes
        + journal_bytes
        + queue_bytes
        + audit_bytes
        + coordination_bytes;
    let read_count = options.queries * options.runs;
    let update_count = ((read_count as f64) * spec.update_rate).round() as usize;
    let delete_count = update_count / 4;
    let audit_count = ((read_count as f64) * spec.audit_rate).round().max(1.0) as usize;
    let cache_hit_count = if options.cache_enabled {
        ((read_count as f64) * cache_hit_rate_for_strategy(strategy, options)).round() as usize
    } else {
        0
    };
    let cache_miss_count = read_count.saturating_sub(cache_hit_count);
    let eviction_count = match strategy {
        P65ActorStrategy::SingleLocalActor => actor_count / 4,
        P65ActorStrategy::SpecializedCrudActors => actor_count / 2,
        P65ActorStrategy::OverAgenticStress => actor_count * 2,
        P65ActorStrategy::NoActorAddressLocal => 0,
    };
    let compaction_count = match strategy {
        P65ActorStrategy::SingleLocalActor => actor_count / 3 + 1,
        P65ActorStrategy::SpecializedCrudActors => actor_count + 1,
        P65ActorStrategy::OverAgenticStress => actor_count * 3,
        P65ActorStrategy::NoActorAddressLocal => 0,
    };
    let conflict_count = match strategy {
        P65ActorStrategy::SingleLocalActor => 0,
        P65ActorStrategy::SpecializedCrudActors => actor_count / 32,
        P65ActorStrategy::OverAgenticStress => actor_count * 2 + options.queries / 100,
        P65ActorStrategy::NoActorAddressLocal => 0,
    };
    let stale_read_count = match strategy {
        P65ActorStrategy::SingleLocalActor => 0,
        P65ActorStrategy::SpecializedCrudActors => actor_count / 64,
        P65ActorStrategy::OverAgenticStress => actor_count + options.queries / 200,
        P65ActorStrategy::NoActorAddressLocal => 0,
    };
    let budget_refusal_count = if total_actor_overhead_bytes > options.budget_bytes {
        ((total_actor_overhead_bytes - options.budget_bytes) / options.budget_bytes.max(1)) as usize
            + 1
    } else {
        0
    };

    Some(LocalActor {
        actor_id: format!("{}::{}", spec.kind.as_str(), strategy.json_str()),
        anchor_address: format!("{}:anchor:0", spec.kind.as_str()),
        neighborhood_radius: options.neighborhood_radius,
        assigned_workload: spec.kind.as_str().to_string(),
        budget_bytes: options.budget_bytes,
        budget_actions: read_count + update_count + delete_count + audit_count,
        cache_enabled: options.cache_enabled,
        journal_enabled: true,
        audit_enabled: true,
        compaction_enabled: strategy != P65ActorStrategy::NoActorAddressLocal,
        state_bytes,
        cache_bytes,
        index_bytes,
        journal_bytes,
        queue_bytes,
        audit_bytes,
        coordination_bytes,
        total_actor_overhead_bytes,
        action_count: read_count + update_count + delete_count + audit_count,
        read_count,
        update_count,
        delete_count,
        audit_count,
        cache_hit_count,
        cache_miss_count,
        eviction_count,
        compaction_count,
        conflict_count,
        stale_read_count,
        budget_refusal_count,
    })
}

fn actor_count_for_strategy(strategy: P65ActorStrategy, unique_addresses_touched: usize) -> usize {
    match strategy {
        P65ActorStrategy::NoActorAddressLocal => 0,
        P65ActorStrategy::SingleLocalActor => (unique_addresses_touched / 64).max(1),
        P65ActorStrategy::SpecializedCrudActors => (unique_addresses_touched / 64).max(1) * 4,
        P65ActorStrategy::OverAgenticStress => (unique_addresses_touched / 8).max(8) * 4,
    }
}

fn adjusted_persisted_cost(
    baseline: BaseAddressLocalCost,
    strategy: P65ActorStrategy,
    actor_overhead: u64,
) -> BaseAddressLocalCost {
    if strategy == P65ActorStrategy::NoActorAddressLocal {
        return baseline;
    }
    let (payload_factor, index_factor, journal_factor, metadata_factor) = match strategy {
        P65ActorStrategy::NoActorAddressLocal => (100, 100, 100, 100),
        P65ActorStrategy::SingleLocalActor => (58, 72, 62, 70),
        P65ActorStrategy::SpecializedCrudActors => (54, 82, 68, 82),
        P65ActorStrategy::OverAgenticStress => (120, 160, 180, 160),
    };
    BaseAddressLocalCost {
        payload_bytes: scale_u64(baseline.payload_bytes, payload_factor) + actor_overhead,
        index_bytes: scale_u64(baseline.index_bytes, index_factor),
        journal_bytes: scale_u64(baseline.journal_bytes, journal_factor),
        manifest_bytes: baseline.manifest_bytes,
        audit_bytes: baseline.audit_bytes,
        metadata_bytes: scale_u64(baseline.metadata_bytes, metadata_factor),
    }
}

fn strategy_decision(
    strategy: P65ActorStrategy,
    actor_net_gain: f64,
    actor_overhead: u64,
    total_persisted_bytes: u64,
    conflict_count: usize,
    stale_read_count: usize,
) -> P65ActorWorkloadDecision {
    if conflict_count > 100 || stale_read_count > 100 {
        return P65ActorWorkloadDecision::NoGoActorConflicts;
    }
    match strategy {
        P65ActorStrategy::NoActorAddressLocal => P65ActorWorkloadDecision::NoActorBaselineBetter,
        P65ActorStrategy::SingleLocalActor => {
            if actor_net_gain > 1.15
                && ratio(actor_overhead as u128, total_persisted_bytes as u128) < 0.45
            {
                P65ActorWorkloadDecision::LocalActorStrong
            } else if actor_net_gain > 1.0 {
                P65ActorWorkloadDecision::LocalActorPromising
            } else {
                P65ActorWorkloadDecision::LocalActorOverheadTooHigh
            }
        }
        P65ActorStrategy::SpecializedCrudActors => {
            if actor_net_gain > 1.05
                && ratio(actor_overhead as u128, total_persisted_bytes as u128) < 0.55
            {
                P65ActorWorkloadDecision::LocalActorPromising
            } else {
                P65ActorWorkloadDecision::SpecializedActorsTooExpensive
            }
        }
        P65ActorStrategy::OverAgenticStress => P65ActorWorkloadDecision::LocalActorOverheadTooHigh,
    }
}

fn base_address_local_cost(
    spec: P65WorkloadSpec,
    generated_units: u128,
    options: &P65RatioActorsOptions,
    unique_addresses_touched: usize,
) -> BaseAddressLocalCost {
    let queries = options.queries as u128;
    let runs = options.runs as u128;
    let radius = options.neighborhood_radius as u128;
    BaseAddressLocalCost {
        payload_bytes: clamp_u64(generated_units * (spec.record_payload_bytes / 16).max(4)),
        index_bytes: clamp_u64(unique_addresses_touched as u128 * 24 + radius * 128),
        journal_bytes: clamp_u64(queries * runs * 32),
        manifest_bytes: clamp_u64(1_024 + radius * 16),
        audit_bytes: clamp_u64(queries * 10 + 384),
        metadata_bytes: clamp_u64(generated_units / 64 + 512),
    }
}

fn actor_strategy_comparisons(entries: &[P65ActorMetrics]) -> Vec<P65ActorStrategyComparison> {
    P64WorkloadKind::all()
        .into_iter()
        .filter_map(|kind| {
            let workload = kind.as_str();
            let baseline = find_entry(entries, workload, P65ActorStrategy::NoActorAddressLocal)?;
            let single = find_entry(entries, workload, P65ActorStrategy::SingleLocalActor)?;
            let specialized =
                find_entry(entries, workload, P65ActorStrategy::SpecializedCrudActors)?;
            let over = find_entry(entries, workload, P65ActorStrategy::OverAgenticStress)?;
            let mut best = single;
            if specialized.ratio_effective_per_byte > best.ratio_effective_per_byte {
                best = specialized;
            }
            if baseline.ratio_effective_per_byte > best.ratio_effective_per_byte {
                best = baseline;
            }
            let decision =
                if best.actor_strategy == P65ActorStrategy::NoActorAddressLocal.json_str() {
                    P65ActorWorkloadDecision::NoActorBaselineBetter
                } else if best.conflict_count > 100 || best.stale_read_count > 100 {
                    P65ActorWorkloadDecision::NoGoActorConflicts
                } else if best.actor_strategy == P65ActorStrategy::SingleLocalActor.json_str()
                    && best.actor_net_gain > 1.15
                {
                    P65ActorWorkloadDecision::LocalActorStrong
                } else if best.actor_net_gain > 1.0 {
                    P65ActorWorkloadDecision::LocalActorPromising
                } else {
                    P65ActorWorkloadDecision::LocalActorOverheadTooHigh
                };
            Some(P65ActorStrategyComparison {
                workload: workload.to_string(),
                baseline_ratio_effective_per_byte: baseline.ratio_effective_per_byte,
                single_local_actor_ratio_effective_per_byte: single.ratio_effective_per_byte,
                specialized_crud_actors_ratio_effective_per_byte: specialized
                    .ratio_effective_per_byte,
                over_agentic_stress_ratio_effective_per_byte: over.ratio_effective_per_byte,
                baseline_effective_gain_vs_materialized: baseline.effective_gain_vs_materialized,
                single_local_actor_effective_gain_vs_materialized: single
                    .effective_gain_vs_materialized,
                specialized_crud_actors_effective_gain_vs_materialized: specialized
                    .effective_gain_vs_materialized,
                over_agentic_stress_effective_gain_vs_materialized: over
                    .effective_gain_vs_materialized,
                best_actor_strategy: best.actor_strategy.clone(),
                actor_net_gain: best.actor_net_gain,
                actor_overhead_ratio: best.actor_overhead_ratio,
                cache_hit_rate: best.cache_hit_rate,
                conflicts: best.conflict_count,
                stale_reads: best.stale_read_count,
                decision,
                interpretation: match decision {
                    P65ActorWorkloadDecision::LocalActorStrong => {
                        "budgeted local actor amortizes enough cost on this fixture"
                    }
                    P65ActorWorkloadDecision::LocalActorPromising => {
                        "local actor improves this fixture but still needs overhead calibration"
                    }
                    P65ActorWorkloadDecision::LocalActorOverheadTooHigh => {
                        "actor overhead is too high for this fixture"
                    }
                    P65ActorWorkloadDecision::SpecializedActorsTooExpensive => {
                        "specialized actor coordination is too expensive"
                    }
                    P65ActorWorkloadDecision::NoActorBaselineBetter => {
                        "direct address-local baseline is better for this fixture"
                    }
                    P65ActorWorkloadDecision::NoGoActorConflicts => {
                        "actor conflicts or stale reads break the locality budget"
                    }
                }
                .to_string(),
            })
        })
        .collect()
}

fn find_entry<'a>(
    entries: &'a [P65ActorMetrics],
    workload: &str,
    strategy: P65ActorStrategy,
) -> Option<&'a P65ActorMetrics> {
    entries
        .iter()
        .find(|entry| entry.workload == workload && entry.actor_strategy == strategy.json_str())
}

fn global_p65_decision(comparisons: &[P65ActorStrategyComparison]) -> P65Decision {
    if comparisons
        .iter()
        .any(|comparison| comparison.conflicts > 250 || comparison.stale_reads > 250)
    {
        return P65Decision::NoGoLocalActors;
    }
    P65Decision::RecalibrateActorOverhead
}

fn global_decision_reasons(
    decision: P65Decision,
    entries: &[P65ActorMetrics],
    comparisons: &[P65ActorStrategyComparison],
) -> Vec<String> {
    let improved_workloads = comparisons
        .iter()
        .filter(|comparison| {
            comparison.best_actor_strategy != P65ActorStrategy::NoActorAddressLocal.json_str()
                && comparison.actor_net_gain > 1.0
        })
        .count();
    vec![
        format!("entry_count: {}", entries.len()),
        format!("actor_improved_workloads: {}", improved_workloads),
        "P65 keeps .atlas grammar and strict_p53 unchanged".to_string(),
        "actors are deterministic budgeted runtime structures, not autonomous intelligence"
            .to_string(),
        "all actor state and coordination are counted as real cost".to_string(),
        "no external dataset or multi-machine campaign is included yet".to_string(),
        format!("decision: {}", decision.as_str()),
    ]
}

fn local_units_per_query(spec: P65WorkloadSpec, radius: usize) -> u128 {
    spec.base_local_units * (radius as u128 * 2 + 1)
}

fn unique_addresses_touched(spec: P65WorkloadSpec, queries: usize) -> usize {
    let cap = (spec.virtual_declared_units / spec.base_local_units).max(1) as usize;
    queries.min(cap)
}

fn effective_gain(virtual_effective_units: u128, total_persisted_bytes: u64) -> f64 {
    ratio(
        virtual_effective_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
        total_persisted_bytes as u128,
    )
}

fn cache_hit_rate_for_strategy(strategy: P65ActorStrategy, options: &P65RatioActorsOptions) -> f64 {
    if !options.cache_enabled {
        return 0.0;
    }
    let radius_factor = options.neighborhood_radius as f64 / 32.0;
    match strategy {
        P65ActorStrategy::NoActorAddressLocal => 0.0,
        P65ActorStrategy::SingleLocalActor => (0.38 + radius_factor).min(0.84),
        P65ActorStrategy::SpecializedCrudActors => (0.30 + radius_factor).min(0.78),
        P65ActorStrategy::OverAgenticStress => (0.12 + radius_factor / 2.0).min(0.42),
    }
}

fn cache_hit_rate_for_actor(actor: &LocalActor) -> Option<f64> {
    if !actor.cache_enabled || actor.read_count == 0 {
        return None;
    }
    Some(actor.cache_hit_count as f64 / actor.read_count as f64)
}

fn observed_runtime_samples(
    spec: P65WorkloadSpec,
    strategy: P65ActorStrategy,
    options: &P65RatioActorsOptions,
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
                    .wrapping_mul(37)
                    .wrapping_add(q * spec.base_local_units)
                    .wrapping_add(options.neighborhood_radius as u128)
                    .wrapping_add(match strategy {
                        P65ActorStrategy::NoActorAddressLocal => 3,
                        P65ActorStrategy::SingleLocalActor => 11,
                        P65ActorStrategy::SpecializedCrudActors => 23,
                        P65ActorStrategy::OverAgenticStress => 97,
                    });
                if strategy == P65ActorStrategy::OverAgenticStress {
                    acc ^= spec
                        .virtual_declared_units
                        .rotate_left((query_idx % 31) as u32);
                } else {
                    acc ^= spec.virtual_safe_units / (q % 17 + 1);
                }
            }
            if acc == 0 {
                return 1;
            }
            start.elapsed().as_nanos().max(1)
        })
        .collect()
}

pub fn write_p65_actor_campaign_exports(
    report: &P65ActorCampaignReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir).map_err(|e| {
        Diagnostic::new(
            DiagnosticCode::Io,
            format!("could not create '{}': {}", export_dir.display(), e),
        )
    })?;
    write_file(
        export_dir.join("p65_actor_campaign_report.json"),
        &p65_report_json(report),
    )?;
    write_file(
        export_dir.join("p65_actor_runs.jsonl"),
        &p65_actor_runs_jsonl(report),
    )?;
    write_file(
        export_dir.join("p65_actor_summary.md"),
        &p65_summary_markdown(report),
    )?;
    write_file(
        export_dir.join("p65_actor_metrics.csv"),
        &p65_actor_metrics_csv(report),
    )?;
    Ok(())
}

fn write_file(path: impl AsRef<Path>, content: &str) -> AtlasResult<()> {
    let path = path.as_ref();
    fs::write(path, content).map_err(|e| {
        Diagnostic::new(
            DiagnosticCode::Io,
            format!("could not write '{}': {}", path.display(), e),
        )
    })
}

pub fn p65_report_json(report: &P65ActorCampaignReport) -> String {
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
        "actor_strategy",
        &report.actor_strategy_filter,
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
    out.push_str(&json_bool("cache_enabled", report.cache_enabled, true, 2));
    out.push_str("  \"actor_strategy_metrics\": [\n");
    for (idx, entry) in report.entries.iter().enumerate() {
        out.push_str(&indent_json(&actor_metrics_json(entry), 4));
        out.push_str(&format!("{}\n", comma(idx, report.entries.len())));
    }
    out.push_str("  ],\n");
    out.push_str("  \"strategy_comparison\": [\n");
    for (idx, comparison) in report.comparisons.iter().enumerate() {
        out.push_str(&indent_json(&strategy_comparison_json(comparison), 4));
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

fn actor_metrics_json(entry: &P65ActorMetrics) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("workload", &entry.workload, true, 2));
    out.push_str(&json_string(
        "actor_strategy",
        &entry.actor_strategy,
        true,
        2,
    ));
    out.push_str(&json_string("description", &entry.description, true, 2));
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
        "virtual_generated_local_units",
        entry.virtual_generated_local_units,
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
        "baseline_no_actor_persisted_bytes",
        entry.baseline_no_actor_persisted_bytes,
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
    out.push_str(&json_u64("metadata_bytes", entry.metadata_bytes, true, 2));
    out.push_str(&json_usize("actor_count", entry.actor_count, true, 2));
    out.push_str(&json_u64(
        "actor_state_bytes",
        entry.actor_state_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "actor_cache_bytes",
        entry.actor_cache_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "actor_index_bytes",
        entry.actor_index_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "actor_journal_bytes",
        entry.actor_journal_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "actor_queue_bytes",
        entry.actor_queue_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "actor_audit_bytes",
        entry.actor_audit_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "actor_coordination_bytes",
        entry.actor_coordination_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "total_actor_overhead_bytes",
        entry.total_actor_overhead_bytes,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "actor_overhead_ratio",
        entry.actor_overhead_ratio,
        true,
        2,
    ));
    match entry.cache_hit_rate {
        Some(value) => out.push_str(&json_f64("cache_hit_rate", value, true, 2)),
        None => out.push_str("  \"cache_hit_rate\": null,\n"),
    }
    out.push_str(&json_usize(
        "actor_action_count",
        entry.actor_action_count,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "coordination_events",
        entry.coordination_events,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "stale_read_count",
        entry.stale_read_count,
        true,
        2,
    ));
    out.push_str(&json_usize("conflict_count", entry.conflict_count, true, 2));
    out.push_str(&json_usize("eviction_count", entry.eviction_count, true, 2));
    out.push_str(&json_usize(
        "compaction_count",
        entry.compaction_count,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "budget_refusal_count",
        entry.budget_refusal_count,
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
        "gain_vs_materialized",
        entry.gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "effective_gain_vs_materialized",
        entry.effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64("actor_net_gain", entry.actor_net_gain, true, 2));
    out.push_str(&json_f64(
        "actor_ratio_delta",
        entry.actor_ratio_delta,
        true,
        2,
    ));
    out.push_str(&json_i128(
        "actor_bytes_delta",
        entry.actor_bytes_delta,
        true,
        2,
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
    if let Some(actor) = &entry.local_actor {
        out.push_str("  \"local_actor\": ");
        out.push_str(&indent_json(&local_actor_json(actor), 2));
        out.push_str(",\n");
    } else {
        out.push_str("  \"local_actor\": null,\n");
    }
    out.push_str(&json_string("decision", entry.decision.as_str(), true, 2));
    string_array_json(
        &mut out,
        "decision_reasons",
        &entry.decision_reasons,
        false,
        2,
    );
    out.push_str("}\n");
    out
}

fn local_actor_json(actor: &LocalActor) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("actor_id", &actor.actor_id, true, 2));
    out.push_str(&json_string(
        "anchor_address",
        &actor.anchor_address,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "neighborhood_radius",
        actor.neighborhood_radius,
        true,
        2,
    ));
    out.push_str(&json_string(
        "assigned_workload",
        &actor.assigned_workload,
        true,
        2,
    ));
    out.push_str(&json_u64("budget_bytes", actor.budget_bytes, true, 2));
    out.push_str(&json_usize("budget_actions", actor.budget_actions, true, 2));
    out.push_str(&json_bool("cache_enabled", actor.cache_enabled, true, 2));
    out.push_str(&json_bool(
        "journal_enabled",
        actor.journal_enabled,
        true,
        2,
    ));
    out.push_str(&json_bool("audit_enabled", actor.audit_enabled, true, 2));
    out.push_str(&json_bool(
        "compaction_enabled",
        actor.compaction_enabled,
        true,
        2,
    ));
    out.push_str(&json_u64("state_bytes", actor.state_bytes, true, 2));
    out.push_str(&json_u64("cache_bytes", actor.cache_bytes, true, 2));
    out.push_str(&json_u64("index_bytes", actor.index_bytes, true, 2));
    out.push_str(&json_u64("journal_bytes", actor.journal_bytes, true, 2));
    out.push_str(&json_u64("queue_bytes", actor.queue_bytes, true, 2));
    out.push_str(&json_u64("audit_bytes", actor.audit_bytes, true, 2));
    out.push_str(&json_u64(
        "coordination_bytes",
        actor.coordination_bytes,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "total_actor_overhead_bytes",
        actor.total_actor_overhead_bytes,
        true,
        2,
    ));
    out.push_str(&json_usize("action_count", actor.action_count, true, 2));
    out.push_str(&json_usize("read_count", actor.read_count, true, 2));
    out.push_str(&json_usize("update_count", actor.update_count, true, 2));
    out.push_str(&json_usize("delete_count", actor.delete_count, true, 2));
    out.push_str(&json_usize("audit_count", actor.audit_count, true, 2));
    out.push_str(&json_usize(
        "cache_hit_count",
        actor.cache_hit_count,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "cache_miss_count",
        actor.cache_miss_count,
        true,
        2,
    ));
    out.push_str(&json_usize("eviction_count", actor.eviction_count, true, 2));
    out.push_str(&json_usize(
        "compaction_count",
        actor.compaction_count,
        true,
        2,
    ));
    out.push_str(&json_usize("conflict_count", actor.conflict_count, true, 2));
    out.push_str(&json_usize(
        "stale_read_count",
        actor.stale_read_count,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "budget_refusal_count",
        actor.budget_refusal_count,
        false,
        2,
    ));
    out.push_str("}\n");
    out
}

fn strategy_comparison_json(comparison: &P65ActorStrategyComparison) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("workload", &comparison.workload, true, 2));
    out.push_str(&json_f64(
        "baseline_ratio_effective_per_byte",
        comparison.baseline_ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "single_local_actor_ratio_effective_per_byte",
        comparison.single_local_actor_ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "specialized_crud_actors_ratio_effective_per_byte",
        comparison.specialized_crud_actors_ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "over_agentic_stress_ratio_effective_per_byte",
        comparison.over_agentic_stress_ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "baseline_effective_gain_vs_materialized",
        comparison.baseline_effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "single_local_actor_effective_gain_vs_materialized",
        comparison.single_local_actor_effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "specialized_crud_actors_effective_gain_vs_materialized",
        comparison.specialized_crud_actors_effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "over_agentic_stress_effective_gain_vs_materialized",
        comparison.over_agentic_stress_effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_string(
        "best_actor_strategy",
        &comparison.best_actor_strategy,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "actor_net_gain",
        comparison.actor_net_gain,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "actor_overhead_ratio",
        comparison.actor_overhead_ratio,
        true,
        2,
    ));
    match comparison.cache_hit_rate {
        Some(value) => out.push_str(&json_f64("cache_hit_rate", value, true, 2)),
        None => out.push_str("  \"cache_hit_rate\": null,\n"),
    }
    out.push_str(&json_usize("conflicts", comparison.conflicts, true, 2));
    out.push_str(&json_usize("stale_reads", comparison.stale_reads, true, 2));
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
    out.push_str("}\n");
    out
}

fn p65_actor_runs_jsonl(report: &P65ActorCampaignReport) -> String {
    let mut out = String::new();
    for entry in &report.entries {
        for run in &entry.runs {
            out.push_str(&format!(
                "{{\"astra_step\":\"P65\",\"workload\":\"{}\",\"actor_strategy\":\"{}\",\"run_index\":{},\"runtime_observed_ns\":{},\"total_persisted_bytes\":{},\"total_actor_overhead_bytes\":{},\"ratio_effective_per_byte\":{:.6},\"conflict_count\":{},\"stale_read_count\":{},\"decision\":\"{}\"}}\n",
                escape_json(&run.workload),
                escape_json(&run.actor_strategy),
                run.run_index,
                run.runtime_observed_ns,
                run.total_persisted_bytes,
                run.total_actor_overhead_bytes,
                run.ratio_effective_per_byte,
                run.conflict_count,
                run.stale_read_count,
                report.decision.as_str()
            ));
        }
    }
    out
}

fn p65_actor_metrics_csv(report: &P65ActorCampaignReport) -> String {
    let mut out = String::new();
    out.push_str("workload,actor_strategy,virtual_effective,total_persisted_bytes,total_actor_overhead_bytes,actor_overhead_ratio,ratio_effective_per_byte,effective_gain_vs_materialized,actor_net_gain,cache_hit_rate,conflict_count,stale_read_count,decision\n");
    for entry in &report.entries {
        out.push_str(&format!(
            "{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{},{},{},{}\n",
            entry.workload,
            entry.actor_strategy,
            entry.virtual_effective_units,
            entry.total_persisted_bytes,
            entry.total_actor_overhead_bytes,
            entry.actor_overhead_ratio,
            entry.ratio_effective_per_byte,
            entry.effective_gain_vs_materialized,
            entry.actor_net_gain,
            entry
                .cache_hit_rate
                .map(|value| format!("{:.6}", value))
                .unwrap_or_else(|| "null".to_string()),
            entry.conflict_count,
            entry.stale_read_count,
            entry.decision.as_str()
        ));
    }
    out
}

pub fn p65_summary_markdown(report: &P65ActorCampaignReport) -> String {
    let best_actor = report
        .comparisons
        .iter()
        .filter(|comparison| {
            comparison.best_actor_strategy != P65ActorStrategy::NoActorAddressLocal.json_str()
        })
        .count();
    let mut out = String::new();
    out.push_str("# ASTRA-P65 Address-Local Actor Summary\n\n");
    out.push_str(&format!("- Mode: `{}`\n", report.mode));
    out.push_str(&format!("- Runs: `{}`\n", report.runs));
    out.push_str(&format!("- Query count: `{}`\n", report.query_count));
    out.push_str(&format!(
        "- Neighborhood radius: `{}`\n",
        report.neighborhood_radius
    ));
    out.push_str(&format!("- Budget bytes: `{}`\n", report.budget_bytes));
    out.push_str(&format!("- Cache enabled: `{}`\n", report.cache_enabled));
    out.push_str(&format!("- Decision: `{}`\n", report.decision.as_str()));
    out.push_str(&format!(
        "- Workloads where an actor beats baseline: `{}`\n\n",
        best_actor
    ));
    out.push_str("| workload | actor_strategy | virtual_effective | real_bytes | actor_overhead_bytes | ratio_effective_per_byte | effective_gain_vs_materialized | actor_net_gain | cache_hit_rate | conflicts | stale_reads | decision |\n");
    out.push_str("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|\n");
    for entry in &report.entries {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} | {:.6} | {:.6} | {:.6} | {} | {} | {} | `{}` |\n",
            entry.workload,
            entry.actor_strategy,
            entry.virtual_effective_units,
            entry.total_persisted_bytes,
            entry.total_actor_overhead_bytes,
            entry.ratio_effective_per_byte,
            entry.effective_gain_vs_materialized,
            entry.actor_net_gain,
            entry
                .cache_hit_rate
                .map(|value| format!("{:.6}", value))
                .unwrap_or_else(|| "null".to_string()),
            entry.conflict_count,
            entry.stale_read_count,
            entry.decision.as_str()
        ));
    }
    out.push_str("\n## Limits\n\n");
    for warning in &report.warnings {
        out.push_str(&format!("- {}\n", warning));
    }
    out
}

pub fn p65_actor_calibration_report_file(
    path: &str,
    options: P65ActorCalibrationOptions,
) -> AtlasResult<P65ActorCalibrationReport> {
    validate_file(path)?;
    p65_actor_calibration_report(path, options)
}

pub fn p65_actor_calibration_json_file(
    path: &str,
    options: P65ActorCalibrationOptions,
) -> AtlasResult<String> {
    let report = p65_actor_calibration_report_file(path, options)?;
    Ok(p65_actor_calibration_json(&report))
}

pub fn p65_actor_calibration_markdown_file(
    path: &str,
    options: P65ActorCalibrationOptions,
) -> AtlasResult<String> {
    let report = p65_actor_calibration_report_file(path, options)?;
    Ok(p65_actor_calibration_markdown(&report))
}

fn p65_actor_calibration_report(
    path: &str,
    options: P65ActorCalibrationOptions,
) -> AtlasResult<P65ActorCalibrationReport> {
    if options.runs == 0
        || options.queries == 0
        || options.radius_grid.is_empty()
        || options.budget_grid.is_empty()
        || options.cache_grid.is_empty()
        || options.journal_grid.is_empty()
        || options.query_locality_grid.is_empty()
    {
        return Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            "P65-2 calibration requires non-empty grids and positive runs/queries",
        ));
    }
    if options.radius_grid.iter().any(|value| *value == 0)
        || options.budget_grid.iter().any(|value| *value == 0)
    {
        return Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            "P65-2 calibration requires radius and budget values greater than zero",
        ));
    }

    let workloads = match options.workload {
        Some(kind) => vec![kind],
        None => P64WorkloadKind::all(),
    };
    let mut configurations = Vec::new();
    for workload in workloads {
        let spec = workload_spec(workload);
        for radius in &options.radius_grid {
            for budget in &options.budget_grid {
                for cache_enabled in &options.cache_grid {
                    for journal_policy in &options.journal_grid {
                        for query_locality in &options.query_locality_grid {
                            configurations.push(calibrate_single_actor_config(
                                spec,
                                &options,
                                *radius,
                                *budget,
                                *cache_enabled,
                                *journal_policy,
                                *query_locality,
                            ));
                        }
                    }
                }
            }
        }
    }

    let best_by_ratio = configurations.iter().cloned().max_by(|a, b| {
        a.ratio_effective_per_byte
            .total_cmp(&b.ratio_effective_per_byte)
    });
    let best_by_overhead = configurations
        .iter()
        .filter(|config| config.conflicts == 0 && config.stale_reads == 0)
        .cloned()
        .min_by(|a, b| a.actor_overhead_ratio.total_cmp(&b.actor_overhead_ratio));
    let best_balanced = configurations
        .iter()
        .cloned()
        .max_by(|a, b| a.balanced_score.total_cmp(&b.balanced_score));
    let pareto_front = pareto_front(&configurations);
    let no_go_configs: Vec<P65CalibrationConfigMetrics> = configurations
        .iter()
        .filter(|config| {
            config.conflicts > 0
                || config.stale_reads > 0
                || config.budget_refusal_rate > 0.10
                || config.actor_overhead_ratio > 0.60
        })
        .take(32)
        .cloned()
        .collect();
    let rejected_config_count = configurations
        .iter()
        .filter(|config| {
            config.conflicts > 0
                || config.stale_reads > 0
                || config.budget_refusal_rate > 0.10
                || config.actor_overhead_ratio > 0.60
        })
        .count();
    let decision = calibration_decision(&best_balanced, &configurations);
    let decision_reasons = calibration_decision_reasons(decision, &best_balanced, &configurations);

    Ok(P65ActorCalibrationReport {
        astra_step: "P65-2".to_string(),
        calibration_version: "p65_actor_overhead_calibration_v1".to_string(),
        program_path: path.to_string(),
        mode: options.mode.as_str().to_string(),
        workload_filter: options
            .workload
            .map(|kind| kind.as_str().to_string())
            .unwrap_or_else(|| "all".to_string()),
        runs: options.runs,
        query_count: options.queries,
        radius_grid: options.radius_grid,
        budget_grid: options.budget_grid,
        cache_grid: options.cache_grid,
        journal_grid: options.journal_grid,
        query_locality_grid: options.query_locality_grid,
        configurations_tested: configurations.len(),
        best_by_ratio,
        best_by_overhead,
        best_balanced,
        pareto_front,
        no_go_configs,
        rejected_config_count,
        decision,
        decision_reasons,
        warnings: vec![
            "P65-2 calibrates deterministic single_local_actor parameters only".to_string(),
            "compaction_policy, update_rate and audit_rate are inherited/not exposed as grids in this prompt"
                .to_string(),
            "balanced_score is an experimental selection heuristic, not a scientific law"
                .to_string(),
            "promotion requires standard and ambitious evidence; this single report remains conservative"
                .to_string(),
        ],
        configurations,
    })
}

fn calibrate_single_actor_config(
    spec: P65WorkloadSpec,
    options: &P65ActorCalibrationOptions,
    radius: usize,
    budget_bytes: u64,
    cache_enabled: bool,
    journal_policy: P65JournalPolicy,
    query_locality: P65QueryLocality,
) -> P65CalibrationConfigMetrics {
    let locality_multiplier = match query_locality {
        P65QueryLocality::Clustered => 0.70,
        P65QueryLocality::Mixed => 1.0,
        P65QueryLocality::Random => 1.28,
    };
    let effective_queries = ((options.queries as f64) * locality_multiplier)
        .round()
        .max(1.0) as usize;
    let actor_options = P65RatioActorsOptions {
        workload: Some(spec.kind),
        actor_strategy: Some(P65ActorStrategy::SingleLocalActor),
        mode: options.mode,
        runs: options.runs,
        queries: effective_queries,
        neighborhood_radius: radius,
        budget_bytes,
        cache_enabled,
    };
    let mut metrics =
        measure_actor_strategy(spec, P65ActorStrategy::SingleLocalActor, &actor_options);

    let journal_factor = match journal_policy {
        P65JournalPolicy::Lazy => 0.82,
        P65JournalPolicy::Compact => 0.64,
    };
    let overhead_factor = match journal_policy {
        P65JournalPolicy::Lazy => 0.92,
        P65JournalPolicy::Compact => 0.78,
    };
    let locality_overhead_factor = match query_locality {
        P65QueryLocality::Clustered => 0.80,
        P65QueryLocality::Mixed => 1.0,
        P65QueryLocality::Random => 1.22,
    };
    let cache_factor = if cache_enabled { 1.0 } else { 0.72 };

    let adjusted_actor_overhead = ((metrics.total_actor_overhead_bytes as f64)
        * overhead_factor
        * locality_overhead_factor
        * cache_factor)
        .round()
        .max(0.0) as u64;
    let actor_overhead_delta =
        metrics.total_actor_overhead_bytes as i128 - adjusted_actor_overhead as i128;
    let adjusted_total = (metrics.total_persisted_bytes as i128
        - actor_overhead_delta
        - (metrics.journal_bytes as f64 * (1.0 - journal_factor)).round() as i128)
        .max(1) as u64;
    let adjusted_cache_hit_rate = if cache_enabled {
        let base = metrics.cache_hit_rate.unwrap_or(0.0);
        let locality_bonus = match query_locality {
            P65QueryLocality::Clustered => 0.14,
            P65QueryLocality::Mixed => 0.04,
            P65QueryLocality::Random => -0.16,
        };
        let journal_bonus = match journal_policy {
            P65JournalPolicy::Lazy => 0.00,
            P65JournalPolicy::Compact => 0.03,
        };
        (base + locality_bonus + journal_bonus).clamp(0.0, 0.92)
    } else {
        0.0
    };
    let budget_refusal_count = if adjusted_actor_overhead > budget_bytes {
        ((adjusted_actor_overhead - budget_bytes) / budget_bytes.max(1)) as usize + 1
    } else {
        0
    };
    let conflict_count = match query_locality {
        P65QueryLocality::Random if !cache_enabled && radius <= 1 => 1,
        _ => 0,
    };
    let stale_read_count = match journal_policy {
        P65JournalPolicy::Lazy if !cache_enabled && query_locality == P65QueryLocality::Random => 1,
        _ => 0,
    };
    let baseline_gain = effective_gain(
        spec.virtual_effective_units,
        metrics.baseline_no_actor_persisted_bytes,
    );
    let effective_gain_vs_materialized =
        effective_gain(spec.virtual_effective_units, adjusted_total);
    let actor_net_gain = if baseline_gain > 0.0 {
        effective_gain_vs_materialized / baseline_gain
    } else {
        0.0
    };
    let actor_overhead_ratio = ratio(adjusted_actor_overhead as u128, adjusted_total as u128);
    let ratio_effective_per_byte = ratio(spec.virtual_effective_units, adjusted_total as u128);
    let bytes_per_query = adjusted_total as f64 / options.queries.max(1) as f64;
    let budget_refusal_rate = budget_refusal_count as f64 / options.queries.max(1) as f64;
    let generated_units_per_query = local_units_per_query(spec, radius);
    let safety_factor = if conflict_count == 0 && stale_read_count == 0 {
        1.0
    } else {
        0.0
    };
    let budget_factor = if budget_refusal_rate <= 0.10 {
        1.0
    } else {
        0.25
    };
    let cache_adjustment = if cache_enabled {
        0.80 + adjusted_cache_hit_rate
    } else {
        0.72
    };
    let balanced_score = actor_net_gain
        * (1.0 - actor_overhead_ratio).max(0.0)
        * cache_adjustment
        * safety_factor
        * budget_factor;
    let promotion_candidate = actor_net_gain > 1.20
        && actor_overhead_ratio < 0.15
        && adjusted_cache_hit_rate >= 0.45
        && conflict_count == 0
        && stale_read_count == 0
        && budget_refusal_rate <= 0.10;
    let decision = if conflict_count > 0 || stale_read_count > 0 || actor_overhead_ratio > 0.70 {
        P65CalibrationDecision::NoGoP65ActorOverhead
    } else {
        P65CalibrationDecision::RecalibrateP65ActorOverhead
    };

    metrics.total_actor_overhead_bytes = adjusted_actor_overhead;
    metrics.total_persisted_bytes = adjusted_total;

    P65CalibrationConfigMetrics {
        config_id: format!(
            "{}:r{}:b{}:cache{}:journal{}:locality{}",
            spec.kind.as_str(),
            radius,
            budget_bytes,
            if cache_enabled { "on" } else { "off" },
            journal_policy.as_str(),
            query_locality.as_str()
        ),
        workload: spec.kind.as_str().to_string(),
        neighborhood_radius: radius,
        budget_bytes,
        cache_policy: if cache_enabled { "on" } else { "off" }.to_string(),
        journal_policy: journal_policy.as_str().to_string(),
        compaction_policy: "not_available".to_string(),
        query_locality: query_locality.as_str().to_string(),
        update_rate: "inherited_from_workload".to_string(),
        audit_rate: "inherited_from_workload".to_string(),
        ratio_effective_per_byte,
        effective_gain_vs_materialized,
        actor_net_gain,
        actor_overhead_ratio,
        actor_overhead_bytes: adjusted_actor_overhead,
        cache_hit_rate: adjusted_cache_hit_rate,
        conflicts: conflict_count,
        stale_reads: stale_read_count,
        budget_refusal_count,
        budget_refusal_rate,
        generated_units_per_query,
        bytes_per_query,
        balanced_score,
        promotion_candidate,
        decision,
    }
}

fn pareto_front(
    configurations: &[P65CalibrationConfigMetrics],
) -> Vec<P65CalibrationConfigMetrics> {
    let mut front = Vec::new();
    for candidate in configurations {
        if candidate.conflicts > 0 || candidate.stale_reads > 0 {
            continue;
        }
        let dominated = configurations.iter().any(|other| {
            other.workload == candidate.workload
                && other.actor_net_gain >= candidate.actor_net_gain
                && other.ratio_effective_per_byte >= candidate.ratio_effective_per_byte
                && other.actor_overhead_ratio <= candidate.actor_overhead_ratio
                && other.bytes_per_query <= candidate.bytes_per_query
                && (other.actor_net_gain > candidate.actor_net_gain
                    || other.ratio_effective_per_byte > candidate.ratio_effective_per_byte
                    || other.actor_overhead_ratio < candidate.actor_overhead_ratio
                    || other.bytes_per_query < candidate.bytes_per_query)
        });
        if !dominated {
            front.push(candidate.clone());
        }
    }
    front.sort_by(|a, b| b.balanced_score.total_cmp(&a.balanced_score));
    front.truncate(32);
    front
}

fn calibration_decision(
    best_balanced: &Option<P65CalibrationConfigMetrics>,
    configurations: &[P65CalibrationConfigMetrics],
) -> P65CalibrationDecision {
    if configurations
        .iter()
        .all(|config| config.conflicts > 0 || config.stale_reads > 0 || config.actor_net_gain < 1.0)
    {
        return P65CalibrationDecision::NoGoP65ActorOverhead;
    }
    if best_balanced
        .as_ref()
        .map(|config| config.promotion_candidate)
        .unwrap_or(false)
    {
        // P65-2 still requires paired standard+ambitious evidence before promotion.
        return P65CalibrationDecision::RecalibrateP65ActorOverhead;
    }
    P65CalibrationDecision::RecalibrateP65ActorOverhead
}

fn calibration_decision_reasons(
    decision: P65CalibrationDecision,
    best_balanced: &Option<P65CalibrationConfigMetrics>,
    configurations: &[P65CalibrationConfigMetrics],
) -> Vec<String> {
    let promotion_candidates = configurations
        .iter()
        .filter(|config| config.promotion_candidate)
        .count();
    let no_go_configs = configurations
        .iter()
        .filter(|config| {
            config.conflicts > 0
                || config.stale_reads > 0
                || config.budget_refusal_rate > 0.10
                || config.actor_overhead_ratio > 0.60
        })
        .count();
    let best_summary = best_balanced
        .as_ref()
        .map(|config| {
            format!(
                "best_balanced: {} actor_net_gain={:.6} overhead={:.6} cache={:.6}",
                config.config_id,
                config.actor_net_gain,
                config.actor_overhead_ratio,
                config.cache_hit_rate
            )
        })
        .unwrap_or_else(|| "best_balanced: none".to_string());
    vec![
        format!("configurations_tested: {}", configurations.len()),
        format!("promotion_candidate_configs: {}", promotion_candidates),
        format!("no_go_or_rejected_configs: {}", no_go_configs),
        best_summary,
        "PROMOTE requires standard and ambitious confirmation; single report stays conservative"
            .to_string(),
        "no timing golden is used".to_string(),
        format!("decision: {}", decision.as_str()),
    ]
}

pub fn write_p65_actor_calibration_exports(
    report: &P65ActorCalibrationReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir).map_err(|e| {
        Diagnostic::new(
            DiagnosticCode::Io,
            format!("could not create '{}': {}", export_dir.display(), e),
        )
    })?;
    write_file(
        export_dir.join("p65_actor_calibration_report.json"),
        &p65_actor_calibration_json(report),
    )?;
    write_file(
        export_dir.join("p65_actor_calibration_runs.jsonl"),
        &p65_actor_calibration_jsonl(report),
    )?;
    write_file(
        export_dir.join("p65_actor_calibration_summary.md"),
        &p65_actor_calibration_markdown(report),
    )?;
    write_file(
        export_dir.join("p65_actor_calibration_grid.csv"),
        &p65_actor_calibration_csv(report),
    )?;
    Ok(())
}

pub fn p65_actor_calibration_json(report: &P65ActorCalibrationReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("astra_step", &report.astra_step, true, 2));
    out.push_str(&json_string(
        "calibration_version",
        &report.calibration_version,
        true,
        2,
    ));
    out.push_str(&json_string("program_path", &report.program_path, true, 2));
    out.push_str(&json_string("mode", &report.mode, true, 2));
    out.push_str(&json_string("workload", &report.workload_filter, true, 2));
    out.push_str(&json_usize("runs", report.runs, true, 2));
    out.push_str(&json_usize("query_count", report.query_count, true, 2));
    out.push_str(&grid_dimensions_json(report));
    out.push_str(&json_usize(
        "configurations_tested",
        report.configurations_tested,
        true,
        2,
    ));
    optional_config_json(&mut out, "best_by_ratio", &report.best_by_ratio, true, 2);
    optional_config_json(
        &mut out,
        "best_by_overhead",
        &report.best_by_overhead,
        true,
        2,
    );
    optional_config_json(&mut out, "best_balanced", &report.best_balanced, true, 2);
    config_array_json(&mut out, "pareto_front", &report.pareto_front, true, 2);
    config_array_json(&mut out, "no_go_configs", &report.no_go_configs, true, 2);
    out.push_str(&json_usize(
        "rejected_config_count",
        report.rejected_config_count,
        true,
        2,
    ));
    out.push_str(&json_string("decision", report.decision.as_str(), true, 2));
    string_array_json(
        &mut out,
        "decision_reasons",
        &report.decision_reasons,
        true,
        2,
    );
    string_array_json(&mut out, "warnings", &report.warnings, true, 2);
    config_array_json(&mut out, "configurations", &report.configurations, false, 2);
    out.push_str("}\n");
    out
}

fn grid_dimensions_json(report: &P65ActorCalibrationReport) -> String {
    let mut out = String::new();
    out.push_str("  \"grid_dimensions\": {\n");
    out.push_str(&json_usize("radius", report.radius_grid.len(), true, 4));
    out.push_str(&json_usize("budget", report.budget_grid.len(), true, 4));
    out.push_str(&json_usize("cache", report.cache_grid.len(), true, 4));
    out.push_str(&json_usize("journal", report.journal_grid.len(), true, 4));
    out.push_str(&json_usize(
        "query_locality",
        report.query_locality_grid.len(),
        false,
        4,
    ));
    out.push_str("  },\n");
    out
}

fn optional_config_json(
    out: &mut String,
    key: &str,
    value: &Option<P65CalibrationConfigMetrics>,
    trailing: bool,
    indent: usize,
) {
    match value {
        Some(config) => {
            out.push_str(&format!("{}\"{}\": ", " ".repeat(indent), key));
            out.push_str(&indent_json(&calibration_config_json(config), indent));
            out.push_str(&format!("{}\n", if trailing { "," } else { "" }));
        }
        None => out.push_str(&format!(
            "{}\"{}\": null{}\n",
            " ".repeat(indent),
            key,
            if trailing { "," } else { "" }
        )),
    }
}

fn config_array_json(
    out: &mut String,
    key: &str,
    values: &[P65CalibrationConfigMetrics],
    trailing: bool,
    indent: usize,
) {
    out.push_str(&format!("{}\"{}\": [\n", " ".repeat(indent), key));
    for (idx, config) in values.iter().enumerate() {
        out.push_str(&indent_json(&calibration_config_json(config), indent + 2));
        out.push_str(&format!("{}\n", comma(idx, values.len())));
    }
    out.push_str(&format!(
        "{}]{}\n",
        " ".repeat(indent),
        if trailing { "," } else { "" }
    ));
}

fn calibration_config_json(config: &P65CalibrationConfigMetrics) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("config_id", &config.config_id, true, 2));
    out.push_str(&json_string("workload", &config.workload, true, 2));
    out.push_str(&json_usize(
        "neighborhood_radius",
        config.neighborhood_radius,
        true,
        2,
    ));
    out.push_str(&json_u64("budget_bytes", config.budget_bytes, true, 2));
    out.push_str(&json_string("cache_policy", &config.cache_policy, true, 2));
    out.push_str(&json_string(
        "journal_policy",
        &config.journal_policy,
        true,
        2,
    ));
    out.push_str(&json_string(
        "compaction_policy",
        &config.compaction_policy,
        true,
        2,
    ));
    out.push_str(&json_string(
        "query_locality",
        &config.query_locality,
        true,
        2,
    ));
    out.push_str(&json_string("update_rate", &config.update_rate, true, 2));
    out.push_str(&json_string("audit_rate", &config.audit_rate, true, 2));
    out.push_str(&json_f64(
        "ratio_effective_per_byte",
        config.ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "effective_gain_vs_materialized",
        config.effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64("actor_net_gain", config.actor_net_gain, true, 2));
    out.push_str(&json_f64(
        "actor_overhead_ratio",
        config.actor_overhead_ratio,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "actor_overhead_bytes",
        config.actor_overhead_bytes,
        true,
        2,
    ));
    out.push_str(&json_f64("cache_hit_rate", config.cache_hit_rate, true, 2));
    out.push_str(&json_usize("conflicts", config.conflicts, true, 2));
    out.push_str(&json_usize("stale_reads", config.stale_reads, true, 2));
    out.push_str(&json_usize(
        "budget_refusal_count",
        config.budget_refusal_count,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "budget_refusal_rate",
        config.budget_refusal_rate,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "generated_units_per_query",
        config.generated_units_per_query,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "bytes_per_query",
        config.bytes_per_query,
        true,
        2,
    ));
    out.push_str(&json_f64("balanced_score", config.balanced_score, true, 2));
    out.push_str(&json_bool(
        "promotion_candidate",
        config.promotion_candidate,
        true,
        2,
    ));
    out.push_str(&json_string("decision", config.decision.as_str(), false, 2));
    out.push_str("}\n");
    out
}

fn p65_actor_calibration_jsonl(report: &P65ActorCalibrationReport) -> String {
    let mut out = String::new();
    for config in &report.configurations {
        out.push_str(&format!(
            "{{\"astra_step\":\"P65-2\",\"config_id\":\"{}\",\"workload\":\"{}\",\"radius\":{},\"budget_bytes\":{},\"cache_policy\":\"{}\",\"journal_policy\":\"{}\",\"query_locality\":\"{}\",\"ratio_effective_per_byte\":{:.6},\"actor_net_gain\":{:.6},\"actor_overhead_ratio\":{:.6},\"cache_hit_rate\":{:.6},\"conflicts\":{},\"stale_reads\":{},\"budget_refusal_count\":{},\"balanced_score\":{:.6},\"decision\":\"{}\"}}\n",
            escape_json(&config.config_id),
            escape_json(&config.workload),
            config.neighborhood_radius,
            config.budget_bytes,
            escape_json(&config.cache_policy),
            escape_json(&config.journal_policy),
            escape_json(&config.query_locality),
            config.ratio_effective_per_byte,
            config.actor_net_gain,
            config.actor_overhead_ratio,
            config.cache_hit_rate,
            config.conflicts,
            config.stale_reads,
            config.budget_refusal_count,
            config.balanced_score,
            config.decision.as_str()
        ));
    }
    out
}

fn p65_actor_calibration_csv(report: &P65ActorCalibrationReport) -> String {
    let mut out = String::new();
    out.push_str("config_id,workload,radius,budget_bytes,cache_policy,journal_policy,query_locality,ratio_effective_per_byte,effective_gain_vs_materialized,actor_net_gain,actor_overhead_ratio,actor_overhead_bytes,cache_hit_rate,conflicts,stale_reads,budget_refusal_count,generated_units_per_query,bytes_per_query,balanced_score,promotion_candidate,decision\n");
    for config in &report.configurations {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{},{:.6},{},{},{},{},{:.6},{:.6},{},{}\n",
            config.config_id,
            config.workload,
            config.neighborhood_radius,
            config.budget_bytes,
            config.cache_policy,
            config.journal_policy,
            config.query_locality,
            config.ratio_effective_per_byte,
            config.effective_gain_vs_materialized,
            config.actor_net_gain,
            config.actor_overhead_ratio,
            config.actor_overhead_bytes,
            config.cache_hit_rate,
            config.conflicts,
            config.stale_reads,
            config.budget_refusal_count,
            config.generated_units_per_query,
            config.bytes_per_query,
            config.balanced_score,
            config.promotion_candidate,
            config.decision.as_str()
        ));
    }
    out
}

pub fn p65_actor_calibration_markdown(report: &P65ActorCalibrationReport) -> String {
    let mut out = String::new();
    out.push_str("# ASTRA-P65-2 Local Actor Overhead Calibration\n\n");
    out.push_str(&format!("- Mode: `{}`\n", report.mode));
    out.push_str(&format!("- Workload: `{}`\n", report.workload_filter));
    out.push_str(&format!("- Runs: `{}`\n", report.runs));
    out.push_str(&format!("- Query count: `{}`\n", report.query_count));
    out.push_str(&format!(
        "- Configurations tested: `{}`\n",
        report.configurations_tested
    ));
    out.push_str(&format!("- Decision: `{}`\n\n", report.decision.as_str()));
    if let Some(best) = &report.best_balanced {
        out.push_str("## Best balanced configuration\n\n");
        out.push_str(&format!(
            "- Config: `{}`\n- Radius: `{}`\n- Budget bytes: `{}`\n- Cache: `{}`\n- Journal: `{}`\n- Query locality: `{}`\n- Actor net gain: `{:.6}`\n- Actor overhead ratio: `{:.6}`\n- Ratio effective per byte: `{:.6}`\n- Cache hit rate: `{:.6}`\n- Conflicts / stale reads: `{}` / `{}`\n- Budget refusals: `{}`\n\n",
            best.config_id,
            best.neighborhood_radius,
            best.budget_bytes,
            best.cache_policy,
            best.journal_policy,
            best.query_locality,
            best.actor_net_gain,
            best.actor_overhead_ratio,
            best.ratio_effective_per_byte,
            best.cache_hit_rate,
            best.conflicts,
            best.stale_reads,
            best.budget_refusal_count
        ));
    }
    out.push_str("| config | workload | radius | budget | cache | journal | locality | net_gain | overhead | cache_hit | score | decision |\n");
    out.push_str("|---|---|---:|---:|---|---|---|---:|---:|---:|---:|---|\n");
    for config in report.pareto_front.iter().take(16) {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} | `{}` | `{}` | `{}` | {:.6} | {:.6} | {:.6} | {:.6} | `{}` |\n",
            config.config_id,
            config.workload,
            config.neighborhood_radius,
            config.budget_bytes,
            config.cache_policy,
            config.journal_policy,
            config.query_locality,
            config.actor_net_gain,
            config.actor_overhead_ratio,
            config.cache_hit_rate,
            config.balanced_score,
            config.decision.as_str()
        ));
    }
    out
}

fn scale_u64(value: u64, percent: u64) -> u64 {
    ((value as u128 * percent as u128) / 100) as u64
}

fn clamp_u64(value: u128) -> u64 {
    value.min(u64::MAX as u128) as u64
}

fn ratio(numerator: u128, denominator: u128) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
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

fn indent_json(value: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    value
        .lines()
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
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

fn json_bool(key: &str, value: bool, trailing: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        key,
        value,
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

fn json_i128(key: &str, value: i128, trailing: bool, indent: usize) -> String {
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

fn string_array_json(
    out: &mut String,
    key: &str,
    values: &[String],
    trailing: bool,
    indent: usize,
) {
    out.push_str(&format!("{}\"{}\": [\n", " ".repeat(indent), key));
    for (idx, value) in values.iter().enumerate() {
        out.push_str(&format!(
            "{}\"{}\"{}\n",
            " ".repeat(indent + 2),
            escape_json(value),
            comma(idx, values.len())
        ));
    }
    out.push_str(&format!(
        "{}]{}\n",
        " ".repeat(indent),
        if trailing { "," } else { "" }
    ));
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
