use crate::{
    validate_file, AtlasResult, Diagnostic, DiagnosticCode, P64WorkloadKind, P66JournalPolicy,
    WorkloadMode,
};
use std::fs;
use std::path::Path;

const ASTRA_STEP: &str = "P67";
const CALIBRATION_VERSION: &str = "p67_address_fiber_overhead_calibration_v1";
const ASSUMED_MATERIALIZED_VALUE_BYTES: u128 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P67CachePolicy {
    Off,
    On,
    Compact,
}

impl P67CachePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::On => "on",
            Self::Compact => "compact",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "off" => Some(Self::Off),
            "on" => Some(Self::On),
            "compact" => Some(Self::Compact),
            _ => None,
        }
    }

    fn is_enabled(self) -> bool {
        self != Self::Off
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P67AuditPolicy {
    Minimal,
    Sampled,
    Full,
}

impl P67AuditPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Sampled => "sampled",
            Self::Full => "full",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "minimal" => Some(Self::Minimal),
            "sampled" => Some(Self::Sampled),
            "full" => Some(Self::Full),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P67CompactionPolicy {
    Off,
    Threshold,
    Aggressive,
}

impl P67CompactionPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Threshold => "threshold",
            Self::Aggressive => "aggressive",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "off" => Some(Self::Off),
            "threshold" => Some(Self::Threshold),
            "aggressive" => Some(Self::Aggressive),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P67QueryLocality {
    Clustered,
    Random,
    Mixed,
}

impl P67QueryLocality {
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
pub enum P67FiberProjectionDepth {
    Shallow,
    Medium,
    Full,
}

impl P67FiberProjectionDepth {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shallow => "shallow",
            Self::Medium => "medium",
            Self::Full => "full",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "shallow" => Some(Self::Shallow),
            "medium" => Some(Self::Medium),
            "full" => Some(Self::Full),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P67Decision {
    PromoteAddressFiberArchitecture,
    RecalibrateFiberOverhead,
    NoGoAddressFiberOverhead,
}

impl P67Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromoteAddressFiberArchitecture => "PROMOTE_P67_ADDRESS_FIBER_ARCHITECTURE",
            Self::RecalibrateFiberOverhead => "RECALIBRATE_P67_FIBER_OVERHEAD",
            Self::NoGoAddressFiberOverhead => "NO_GO_P67_ADDRESS_FIBER_OVERHEAD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P67ConfigDecision {
    PromotionCandidate,
    RecalibrateFiberOverhead,
    NoGoFiberSafety,
}

impl P67ConfigDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromotionCandidate => "P67_PROMOTION_CANDIDATE",
            Self::RecalibrateFiberOverhead => "P67_RECALIBRATE_FIBER_OVERHEAD",
            Self::NoGoFiberSafety => "P67_NO_GO_FIBER_SAFETY",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P67FiberCalibrationOptions {
    pub workload: Option<P64WorkloadKind>,
    pub mode: WorkloadMode,
    pub runs: usize,
    pub queries: usize,
    pub radius_grid: Vec<usize>,
    pub budget_grid: Vec<u64>,
    pub cache_grid: Vec<P67CachePolicy>,
    pub journal_grid: Vec<P66JournalPolicy>,
    pub audit_grid: Vec<P67AuditPolicy>,
    pub compaction_grid: Vec<P67CompactionPolicy>,
    pub query_locality_grid: Vec<P67QueryLocality>,
    pub fiber_projection_grid: Vec<P67FiberProjectionDepth>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P67FiberCalibrationReport {
    pub astra_step: String,
    pub calibration_version: String,
    pub program_path: String,
    pub mode: String,
    pub workload_filter: String,
    pub runs: usize,
    pub query_count: usize,
    pub radius_grid: Vec<usize>,
    pub budget_grid: Vec<u64>,
    pub cache_grid: Vec<P67CachePolicy>,
    pub journal_grid: Vec<P66JournalPolicy>,
    pub audit_grid: Vec<P67AuditPolicy>,
    pub compaction_grid: Vec<P67CompactionPolicy>,
    pub query_locality_grid: Vec<P67QueryLocality>,
    pub fiber_projection_grid: Vec<P67FiberProjectionDepth>,
    pub configuration_count: usize,
    pub best_by_ratio: Option<P67FiberCalibrationConfig>,
    pub best_by_overhead: Option<P67FiberCalibrationConfig>,
    pub best_by_net_gain: Option<P67FiberCalibrationConfig>,
    pub best_balanced: Option<P67FiberCalibrationConfig>,
    pub pareto_front: Vec<P67FiberCalibrationConfig>,
    pub no_go_configs: Vec<P67FiberCalibrationConfig>,
    pub rejected_config_count: usize,
    pub promotion_candidate: bool,
    pub decision: P67Decision,
    pub decision_reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub configurations: Vec<P67FiberCalibrationConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P67FiberCalibrationConfig {
    pub config_id: String,
    pub workload: String,
    pub radius: usize,
    pub budget_bytes: u64,
    pub cache_policy: String,
    pub journal_policy: String,
    pub audit_policy: String,
    pub compaction_policy: String,
    pub query_locality: String,
    pub fiber_projection_depth: String,
    pub update_rate: String,
    pub metadata_policy: String,
    pub fiber_generated_units: u128,
    pub fiber_effective_units: u128,
    pub fiber_total_bytes: u64,
    pub fiber_ratio_effective_per_byte: f64,
    pub effective_gain_vs_materialized: f64,
    pub address_fiber_net_gain: f64,
    pub avg_actor_overhead_ratio: f64,
    pub actor_overhead_bytes: u64,
    pub fiber_cache_bytes: u64,
    pub fiber_journal_bytes: u64,
    pub fiber_audit_bytes: u64,
    pub fiber_metadata_bytes: u64,
    pub update_count: usize,
    pub audit_count: usize,
    pub compaction_count: usize,
    pub eviction_count: usize,
    pub conflicts: usize,
    pub stale_reads: usize,
    pub budget_refusals: usize,
    pub budget_refusal_rate: f64,
    pub cache_hit_rate: f64,
    pub bytes_per_query: f64,
    pub generated_units_per_query: f64,
    pub balanced_score: f64,
    pub promotion_candidate: bool,
    pub decision: P67ConfigDecision,
}

#[derive(Debug, Clone, Copy)]
struct P67WorkloadSpec {
    kind: P64WorkloadKind,
    virtual_declared_units: u128,
    virtual_effective_units: u128,
    base_fiber_units: u128,
    record_payload_bytes: u128,
    base_actor_net_gain: f64,
    base_ratio_effective_per_byte: f64,
    update_rate: f64,
    audit_rate: f64,
}

pub fn p67_fiber_calibration_report_file(
    path: &str,
    options: P67FiberCalibrationOptions,
) -> AtlasResult<P67FiberCalibrationReport> {
    validate_file(path)?;
    p67_fiber_calibration_report(path, options)
}

pub fn write_p67_fiber_calibration_exports(
    report: &P67FiberCalibrationReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    write_file(
        export_dir.join("p67_fiber_calibration_report.json"),
        &p67_fiber_calibration_json(report),
    )?;
    write_file(
        export_dir.join("p67_fiber_calibration_runs.jsonl"),
        &p67_fiber_calibration_jsonl(report),
    )?;
    write_file(
        export_dir.join("p67_fiber_calibration_grid.csv"),
        &p67_fiber_calibration_csv(report),
    )?;
    write_file(
        export_dir.join("p67_fiber_calibration_summary.md"),
        &p67_fiber_calibration_markdown(report),
    )?;
    Ok(())
}

fn p67_fiber_calibration_report(
    path: &str,
    options: P67FiberCalibrationOptions,
) -> AtlasResult<P67FiberCalibrationReport> {
    if options.runs == 0 || options.queries == 0 {
        return Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            "P67 requires runs and queries greater than zero",
        ));
    }
    if options.radius_grid.is_empty()
        || options.budget_grid.is_empty()
        || options.cache_grid.is_empty()
        || options.journal_grid.is_empty()
        || options.audit_grid.is_empty()
        || options.compaction_grid.is_empty()
        || options.query_locality_grid.is_empty()
        || options.fiber_projection_grid.is_empty()
    {
        return Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            "P67 calibration grids must not be empty",
        ));
    }

    let specs = workload_specs(options.workload);
    let mut configurations = Vec::new();
    for spec in specs {
        for radius in &options.radius_grid {
            for budget_bytes in &options.budget_grid {
                for cache_policy in &options.cache_grid {
                    for journal_policy in &options.journal_grid {
                        for audit_policy in &options.audit_grid {
                            for compaction_policy in &options.compaction_grid {
                                for query_locality in &options.query_locality_grid {
                                    for projection_depth in &options.fiber_projection_grid {
                                        configurations.push(measure_config(
                                            spec,
                                            &options,
                                            *radius,
                                            *budget_bytes,
                                            *cache_policy,
                                            *journal_policy,
                                            *audit_policy,
                                            *compaction_policy,
                                            *query_locality,
                                            *projection_depth,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let best_by_ratio = max_by_f64(&configurations, |config| {
        config.fiber_ratio_effective_per_byte
    });
    let best_by_overhead = min_by_f64(&safe_configs(&configurations), |config| {
        config.avg_actor_overhead_ratio
    });
    let best_by_net_gain = max_by_f64(&configurations, |config| config.address_fiber_net_gain);
    let best_balanced = max_by_f64(&configurations, |config| config.balanced_score);
    let mut pareto_front = pareto_front(&configurations);
    pareto_front.sort_by(|a, b| {
        b.balanced_score
            .partial_cmp(&a.balanced_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    pareto_front.truncate(32);
    let no_go_configs: Vec<_> = configurations
        .iter()
        .filter(|config| config.decision == P67ConfigDecision::NoGoFiberSafety)
        .take(32)
        .cloned()
        .collect();
    let rejected_config_count = configurations
        .iter()
        .filter(|config| config.decision == P67ConfigDecision::NoGoFiberSafety)
        .count();
    let promotion_candidate = best_balanced
        .as_ref()
        .map(|config| config.promotion_candidate)
        .unwrap_or(false);
    let decision = global_decision(&configurations, best_balanced.as_ref());
    let decision_reasons = global_decision_reasons(
        decision,
        promotion_candidate,
        &configurations,
        best_balanced.as_ref(),
    );

    Ok(P67FiberCalibrationReport {
        astra_step: ASTRA_STEP.to_string(),
        calibration_version: CALIBRATION_VERSION.to_string(),
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
        audit_grid: options.audit_grid,
        compaction_grid: options.compaction_grid,
        query_locality_grid: options.query_locality_grid,
        fiber_projection_grid: options.fiber_projection_grid,
        configuration_count: configurations.len(),
        best_by_ratio,
        best_by_overhead,
        best_by_net_gain,
        best_balanced,
        pareto_front,
        no_go_configs,
        rejected_config_count,
        promotion_candidate,
        decision,
        decision_reasons,
        warnings: vec![
            "P67 calibrates deterministic internal address-fiber fixtures only".to_string(),
            "promotion candidates remain local-first and require paired standard/ambitious review"
                .to_string(),
            "no timing golden is introduced".to_string(),
            "metadata_policy and update_rate are held at standard deterministic defaults in this first P67 grid"
                .to_string(),
        ],
        configurations,
    })
}

fn workload_spec(kind: P64WorkloadKind) -> P67WorkloadSpec {
    match kind {
        P64WorkloadKind::RealishLogEvents => P67WorkloadSpec {
            kind,
            virtual_declared_units: 12_000_000,
            virtual_effective_units: 3_600_000,
            base_fiber_units: 18,
            record_payload_bytes: 96,
            base_actor_net_gain: 3.602743,
            base_ratio_effective_per_byte: 3.967310,
            update_rate: 0.10,
            audit_rate: 0.02,
        },
        P64WorkloadKind::RealishSparseCsv => P67WorkloadSpec {
            kind,
            virtual_declared_units: 48_000_000,
            virtual_effective_units: 7_200_000,
            base_fiber_units: 36,
            record_payload_bytes: 48,
            base_actor_net_gain: 4.237254,
            base_ratio_effective_per_byte: 7.568229,
            update_rate: 0.18,
            audit_rate: 0.03,
        },
        P64WorkloadKind::RealishJsonRecords => P67WorkloadSpec {
            kind,
            virtual_declared_units: 8_000_000,
            virtual_effective_units: 2_200_000,
            base_fiber_units: 16,
            record_payload_bytes: 128,
            base_actor_net_gain: 4.085486,
            base_ratio_effective_per_byte: 2.436762,
            update_rate: 0.12,
            audit_rate: 0.02,
        },
        P64WorkloadKind::RealishHybridFieldFixture => P67WorkloadSpec {
            kind,
            virtual_declared_units: 64_000_000,
            virtual_effective_units: 9_600_000,
            base_fiber_units: 54,
            record_payload_bytes: 64,
            base_actor_net_gain: 5.575348,
            base_ratio_effective_per_byte: 9.645586,
            update_rate: 0.05,
            audit_rate: 0.04,
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn measure_config(
    spec: P67WorkloadSpec,
    options: &P67FiberCalibrationOptions,
    radius: usize,
    budget_bytes: u64,
    cache_policy: P67CachePolicy,
    journal_policy: P66JournalPolicy,
    audit_policy: P67AuditPolicy,
    compaction_policy: P67CompactionPolicy,
    query_locality: P67QueryLocality,
    projection_depth: P67FiberProjectionDepth,
) -> P67FiberCalibrationConfig {
    let generated_units = generated_units(
        spec,
        options.queries,
        radius,
        query_locality,
        projection_depth,
    );
    let effective_units = effective_units(spec, query_locality, projection_depth);
    let update_count = scaled_count(options.queries * options.runs, spec.update_rate);
    let audit_count = audit_count(options, spec, audit_policy);
    let compaction_count = compaction_count(options.queries, radius, compaction_policy);
    let eviction_count = eviction_count(options.queries, cache_policy, query_locality);
    let cache_hit_rate = cache_hit_rate(radius, cache_policy, query_locality, projection_depth);
    let fiber_cache_bytes = cache_bytes(generated_units, options.queries, cache_policy);
    let fiber_journal_bytes = journal_bytes(options, journal_policy, compaction_policy);
    let fiber_audit_bytes = audit_bytes(audit_count, audit_policy);
    let fiber_metadata_bytes = metadata_bytes(generated_units, projection_depth);
    let actor_overhead_bytes = actor_overhead_bytes(
        options,
        radius,
        cache_policy,
        journal_policy,
        audit_policy,
        compaction_policy,
        query_locality,
        projection_depth,
        generated_units,
        audit_count,
    );
    let payload_bytes = clamp_u64(
        generated_units
            * (spec.record_payload_bytes / payload_divisor(projection_depth, cache_policy)).max(2),
    );
    let index_bytes = clamp_u64(
        options.queries as u128 * index_per_query(query_locality, projection_depth)
            + radius as u128 * 192,
    );
    let fiber_total_bytes = payload_bytes
        + index_bytes
        + fiber_cache_bytes
        + fiber_journal_bytes
        + fiber_audit_bytes
        + fiber_metadata_bytes
        + actor_overhead_bytes;
    let fiber_ratio_effective_per_byte = ratio(effective_units, fiber_total_bytes as u128);
    let effective_gain_vs_materialized = effective_gain(effective_units, fiber_total_bytes);
    let baseline_ratio = baseline_ratio(spec, options.mode);
    let address_fiber_net_gain = if baseline_ratio > 0.0 {
        fiber_ratio_effective_per_byte / baseline_ratio
    } else {
        0.0
    };
    let avg_actor_overhead_ratio = ratio(actor_overhead_bytes as u128, fiber_total_bytes as u128);
    let bytes_per_query = fiber_total_bytes as f64 / options.queries as f64;
    let generated_units_per_query = generated_units as f64 / options.queries as f64;
    let budget_refusals = budget_refusals(actor_overhead_bytes, budget_bytes, options.queries);
    let budget_refusal_rate = budget_refusals as f64 / options.queries as f64;
    let conflicts = conflict_count(
        budget_refusal_rate,
        cache_policy,
        journal_policy,
        query_locality,
    );
    let stale_reads = stale_read_count(cache_policy, journal_policy, query_locality);
    let promotion_candidate = promotion_candidate(
        options.mode,
        address_fiber_net_gain,
        avg_actor_overhead_ratio,
        conflicts,
        stale_reads,
        budget_refusal_rate,
        update_count,
        audit_count,
        compaction_count,
    );
    let decision = if conflicts > 0 || stale_reads > 0 || budget_refusal_rate > 0.25 {
        P67ConfigDecision::NoGoFiberSafety
    } else if promotion_candidate {
        P67ConfigDecision::PromotionCandidate
    } else {
        P67ConfigDecision::RecalibrateFiberOverhead
    };
    let balanced_score = balanced_score(
        address_fiber_net_gain,
        avg_actor_overhead_ratio,
        cache_hit_rate,
        compaction_policy,
        conflicts,
        stale_reads,
        budget_refusal_rate,
    );

    P67FiberCalibrationConfig {
        config_id: format!(
            "{}:r{}:b{}:cache{}:journal{}:audit{}:compact{}:locality{}:projection{}",
            spec.kind.as_str(),
            radius,
            budget_bytes,
            cache_policy.as_str(),
            journal_policy.as_str(),
            audit_policy.as_str(),
            compaction_policy.as_str(),
            query_locality.as_str(),
            projection_depth.as_str()
        ),
        workload: spec.kind.as_str().to_string(),
        radius,
        budget_bytes,
        cache_policy: cache_policy.as_str().to_string(),
        journal_policy: journal_policy.as_str().to_string(),
        audit_policy: audit_policy.as_str().to_string(),
        compaction_policy: compaction_policy.as_str().to_string(),
        query_locality: query_locality.as_str().to_string(),
        fiber_projection_depth: projection_depth.as_str().to_string(),
        update_rate: "medium".to_string(),
        metadata_policy: "standard".to_string(),
        fiber_generated_units: generated_units,
        fiber_effective_units: effective_units,
        fiber_total_bytes,
        fiber_ratio_effective_per_byte,
        effective_gain_vs_materialized,
        address_fiber_net_gain,
        avg_actor_overhead_ratio,
        actor_overhead_bytes,
        fiber_cache_bytes,
        fiber_journal_bytes,
        fiber_audit_bytes,
        fiber_metadata_bytes,
        update_count,
        audit_count,
        compaction_count,
        eviction_count,
        conflicts,
        stale_reads,
        budget_refusals,
        budget_refusal_rate,
        cache_hit_rate,
        bytes_per_query,
        generated_units_per_query,
        balanced_score,
        promotion_candidate,
        decision,
    }
}

fn workload_specs(filter: Option<P64WorkloadKind>) -> Vec<P67WorkloadSpec> {
    match filter {
        Some(kind) => vec![workload_spec(kind)],
        None => P64WorkloadKind::all()
            .into_iter()
            .map(workload_spec)
            .collect(),
    }
}

fn generated_units(
    spec: P67WorkloadSpec,
    queries: usize,
    radius: usize,
    locality: P67QueryLocality,
    projection: P67FiberProjectionDepth,
) -> u128 {
    let radius_factor = radius.max(1) as f64;
    let locality_factor = match locality {
        P67QueryLocality::Clustered => 0.72,
        P67QueryLocality::Mixed => 0.92,
        P67QueryLocality::Random => 1.15,
    };
    let projection_factor = match projection {
        P67FiberProjectionDepth::Shallow => 0.62,
        P67FiberProjectionDepth::Medium => 0.82,
        P67FiberProjectionDepth::Full => 1.0,
    };
    let units = spec.base_fiber_units as f64
        * queries as f64
        * radius_factor
        * locality_factor
        * projection_factor;
    (units.round() as u128).min(spec.virtual_declared_units)
}

fn effective_units(
    spec: P67WorkloadSpec,
    locality: P67QueryLocality,
    projection: P67FiberProjectionDepth,
) -> u128 {
    let locality_factor = match locality {
        P67QueryLocality::Clustered => 1.04,
        P67QueryLocality::Mixed => 1.0,
        P67QueryLocality::Random => 0.93,
    };
    let projection_factor = match projection {
        P67FiberProjectionDepth::Shallow => 0.86,
        P67FiberProjectionDepth::Medium => 0.94,
        P67FiberProjectionDepth::Full => 1.0,
    };
    ((spec.virtual_effective_units as f64 * 0.88 * locality_factor * projection_factor).round()
        as u128)
        .min(spec.virtual_effective_units)
}

fn payload_divisor(projection: P67FiberProjectionDepth, cache_policy: P67CachePolicy) -> u128 {
    let base = match projection {
        P67FiberProjectionDepth::Shallow => 10,
        P67FiberProjectionDepth::Medium => 8,
        P67FiberProjectionDepth::Full => 6,
    };
    if cache_policy == P67CachePolicy::Compact {
        base + 1
    } else {
        base
    }
}

fn index_per_query(locality: P67QueryLocality, projection: P67FiberProjectionDepth) -> u128 {
    let locality_cost = match locality {
        P67QueryLocality::Clustered => 8,
        P67QueryLocality::Mixed => 12,
        P67QueryLocality::Random => 20,
    };
    let projection_cost = match projection {
        P67FiberProjectionDepth::Shallow => 4,
        P67FiberProjectionDepth::Medium => 8,
        P67FiberProjectionDepth::Full => 12,
    };
    locality_cost + projection_cost
}

fn cache_bytes(generated_units: u128, queries: usize, cache_policy: P67CachePolicy) -> u64 {
    match cache_policy {
        P67CachePolicy::Off => 0,
        P67CachePolicy::On => clamp_u64(generated_units / 768 + queries as u128 * 10),
        P67CachePolicy::Compact => clamp_u64(generated_units / 1_024 + queries as u128 * 7),
    }
}

fn journal_bytes(
    options: &P67FiberCalibrationOptions,
    journal_policy: P66JournalPolicy,
    compaction_policy: P67CompactionPolicy,
) -> u64 {
    let factor = match journal_policy {
        P66JournalPolicy::Eager => 10,
        P66JournalPolicy::Lazy => 6,
        P66JournalPolicy::Compact => 3,
    };
    let compaction_factor = match compaction_policy {
        P67CompactionPolicy::Off => 120,
        P67CompactionPolicy::Threshold => 85,
        P67CompactionPolicy::Aggressive => 72,
    };
    clamp_u64(options.queries as u128 * options.runs as u128 * factor * compaction_factor / 100)
}

fn audit_count(
    options: &P67FiberCalibrationOptions,
    spec: P67WorkloadSpec,
    audit_policy: P67AuditPolicy,
) -> usize {
    let base = (options.queries as f64 * options.runs as f64 * spec.audit_rate).round() as usize;
    match audit_policy {
        P67AuditPolicy::Minimal => base.max(1),
        P67AuditPolicy::Sampled => (base * 2).max(1),
        P67AuditPolicy::Full => (base * 4).max(1),
    }
}

fn audit_bytes(audit_count: usize, audit_policy: P67AuditPolicy) -> u64 {
    let per_audit = match audit_policy {
        P67AuditPolicy::Minimal => 12,
        P67AuditPolicy::Sampled => 18,
        P67AuditPolicy::Full => 32,
    };
    audit_count as u64 * per_audit + 512
}

fn metadata_bytes(generated_units: u128, projection: P67FiberProjectionDepth) -> u64 {
    let divisor = match projection {
        P67FiberProjectionDepth::Shallow => 512,
        P67FiberProjectionDepth::Medium => 384,
        P67FiberProjectionDepth::Full => 256,
    };
    clamp_u64(generated_units / divisor + 768)
}

#[allow(clippy::too_many_arguments)]
fn actor_overhead_bytes(
    options: &P67FiberCalibrationOptions,
    radius: usize,
    cache_policy: P67CachePolicy,
    journal_policy: P66JournalPolicy,
    audit_policy: P67AuditPolicy,
    compaction_policy: P67CompactionPolicy,
    locality: P67QueryLocality,
    projection: P67FiberProjectionDepth,
    generated_units: u128,
    audit_count: usize,
) -> u64 {
    let actor_count = actor_count(options.queries, locality, radius);
    let state_bytes = actor_count as u64 * 96;
    let cache_actor_bytes = match cache_policy {
        P67CachePolicy::Off => 0,
        P67CachePolicy::On => actor_count as u64 * 192 + clamp_u64(generated_units / 2_048),
        P67CachePolicy::Compact => actor_count as u64 * 128 + clamp_u64(generated_units / 3_072),
    };
    let index_bytes = actor_count as u64 * 112 + radius as u64 * 128;
    let journal_factor = match journal_policy {
        P66JournalPolicy::Eager => 4,
        P66JournalPolicy::Lazy => 3,
        P66JournalPolicy::Compact => 1,
    };
    let compaction_factor = match compaction_policy {
        P67CompactionPolicy::Off => 130,
        P67CompactionPolicy::Threshold => 90,
        P67CompactionPolicy::Aggressive => 78,
    };
    let journal_bytes =
        options.queries as u64 * options.runs as u64 * journal_factor * compaction_factor / 100;
    let queue_bytes = actor_count as u64 * 32 + options.queries as u64 / 6;
    let audit_factor = match audit_policy {
        P67AuditPolicy::Minimal => 4,
        P67AuditPolicy::Sampled => 7,
        P67AuditPolicy::Full => 12,
    };
    let audit_bytes = audit_count as u64 * audit_factor + actor_count as u64 * 12;
    let coordination_bytes = actor_count as u64
        * match locality {
            P67QueryLocality::Clustered => 16,
            P67QueryLocality::Mixed => 24,
            P67QueryLocality::Random => 42,
        };
    let metadata_bytes = actor_count as u64
        * match projection {
            P67FiberProjectionDepth::Shallow => 24,
            P67FiberProjectionDepth::Medium => 36,
            P67FiberProjectionDepth::Full => 54,
        };
    state_bytes
        + cache_actor_bytes
        + index_bytes
        + journal_bytes
        + queue_bytes
        + audit_bytes
        + coordination_bytes
        + metadata_bytes
}

fn actor_count(queries: usize, locality: P67QueryLocality, radius: usize) -> usize {
    let divisor = match locality {
        P67QueryLocality::Clustered => 160,
        P67QueryLocality::Mixed => 96,
        P67QueryLocality::Random => 48,
    };
    (queries / divisor).max(1) * radius.max(1).div_ceil(3)
}

fn compaction_count(
    queries: usize,
    radius: usize,
    compaction_policy: P67CompactionPolicy,
) -> usize {
    match compaction_policy {
        P67CompactionPolicy::Off => 0,
        P67CompactionPolicy::Threshold => (queries / (64 * radius.max(1))).max(1),
        P67CompactionPolicy::Aggressive => (queries / (32 * radius.max(1))).max(1),
    }
}

fn eviction_count(
    queries: usize,
    cache_policy: P67CachePolicy,
    locality: P67QueryLocality,
) -> usize {
    if !cache_policy.is_enabled() {
        return 0;
    }
    let divisor = match locality {
        P67QueryLocality::Clustered => 120,
        P67QueryLocality::Mixed => 80,
        P67QueryLocality::Random => 44,
    };
    queries / divisor
}

fn cache_hit_rate(
    radius: usize,
    cache_policy: P67CachePolicy,
    locality: P67QueryLocality,
    projection: P67FiberProjectionDepth,
) -> f64 {
    if !cache_policy.is_enabled() {
        return 0.0;
    }
    let cache_bonus = match cache_policy {
        P67CachePolicy::Off => 0.0,
        P67CachePolicy::On => 0.06,
        P67CachePolicy::Compact => 0.10,
    };
    let locality_bonus = match locality {
        P67QueryLocality::Clustered => 0.14,
        P67QueryLocality::Mixed => 0.07,
        P67QueryLocality::Random => -0.05,
    };
    let projection_bonus = match projection {
        P67FiberProjectionDepth::Shallow => 0.04,
        P67FiberProjectionDepth::Medium => 0.02,
        P67FiberProjectionDepth::Full => 0.0,
    };
    (0.40 + radius as f64 * 0.035 + cache_bonus + locality_bonus + projection_bonus).min(0.86)
}

fn budget_refusals(actor_overhead: u64, budget_bytes: u64, queries: usize) -> usize {
    if actor_overhead <= budget_bytes {
        return 0;
    }
    let over_ratio = (actor_overhead - budget_bytes) as f64 / budget_bytes.max(1) as f64;
    ((queries as f64 * over_ratio.min(1.0) * 0.18).round() as usize).max(1)
}

fn conflict_count(
    budget_refusal_rate: f64,
    cache_policy: P67CachePolicy,
    journal_policy: P66JournalPolicy,
    locality: P67QueryLocality,
) -> usize {
    if budget_refusal_rate > 0.12 {
        return 1;
    }
    if cache_policy == P67CachePolicy::Off
        && journal_policy == P66JournalPolicy::Lazy
        && locality == P67QueryLocality::Random
    {
        return 1;
    }
    0
}

fn stale_read_count(
    cache_policy: P67CachePolicy,
    journal_policy: P66JournalPolicy,
    locality: P67QueryLocality,
) -> usize {
    if cache_policy == P67CachePolicy::Off
        && journal_policy != P66JournalPolicy::Compact
        && locality == P67QueryLocality::Random
    {
        return 1;
    }
    0
}

fn promotion_candidate(
    mode: WorkloadMode,
    net_gain: f64,
    overhead_ratio: f64,
    conflicts: usize,
    stale_reads: usize,
    budget_refusal_rate: f64,
    update_count: usize,
    audit_count: usize,
    compaction_count: usize,
) -> bool {
    if conflicts > 0
        || stale_reads > 0
        || budget_refusal_rate >= 0.02
        || update_count == 0
        || audit_count == 0
        || compaction_count == 0
    {
        return false;
    }
    match mode {
        WorkloadMode::Standard | WorkloadMode::Smoke => net_gain > 3.0 && overhead_ratio < 0.15,
        WorkloadMode::Ambitious => net_gain > 2.5 && overhead_ratio < 0.18,
    }
}

fn balanced_score(
    net_gain: f64,
    overhead_ratio: f64,
    cache_hit_rate: f64,
    compaction_policy: P67CompactionPolicy,
    conflicts: usize,
    stale_reads: usize,
    budget_refusal_rate: f64,
) -> f64 {
    if conflicts > 0 || stale_reads > 0 {
        return 0.0;
    }
    let cache_factor = 0.75 + cache_hit_rate.min(0.90) * 0.35;
    let compaction_factor = match compaction_policy {
        P67CompactionPolicy::Off => 0.82,
        P67CompactionPolicy::Threshold => 1.0,
        P67CompactionPolicy::Aggressive => 0.94,
    };
    let budget_factor = (1.0 - budget_refusal_rate * 4.0).max(0.0);
    net_gain * (1.0 - overhead_ratio).max(0.0) * cache_factor * compaction_factor * budget_factor
}

fn baseline_ratio(spec: P67WorkloadSpec, mode: WorkloadMode) -> f64 {
    let mode_factor = match mode {
        WorkloadMode::Smoke => 1.0,
        WorkloadMode::Standard => 1.0,
        WorkloadMode::Ambitious => 0.14,
    };
    (spec.base_ratio_effective_per_byte / spec.base_actor_net_gain) * mode_factor
}

fn scaled_count(base: usize, rate: f64) -> usize {
    ((base as f64) * rate).round() as usize
}

fn effective_gain(virtual_effective_units: u128, total_bytes: u64) -> f64 {
    ratio(
        virtual_effective_units * ASSUMED_MATERIALIZED_VALUE_BYTES,
        total_bytes as u128,
    )
}

fn safe_configs(configs: &[P67FiberCalibrationConfig]) -> Vec<P67FiberCalibrationConfig> {
    configs
        .iter()
        .filter(|config| config.conflicts == 0 && config.stale_reads == 0)
        .cloned()
        .collect()
}

fn pareto_front(configs: &[P67FiberCalibrationConfig]) -> Vec<P67FiberCalibrationConfig> {
    let safe = safe_configs(configs);
    let mut front = Vec::new();
    for candidate in &safe {
        let dominated = safe.iter().any(|other| dominates(other, candidate));
        if !dominated {
            front.push(candidate.clone());
        }
    }
    front
}

fn dominates(a: &P67FiberCalibrationConfig, b: &P67FiberCalibrationConfig) -> bool {
    let no_worse = a.address_fiber_net_gain >= b.address_fiber_net_gain
        && a.fiber_ratio_effective_per_byte >= b.fiber_ratio_effective_per_byte
        && a.avg_actor_overhead_ratio <= b.avg_actor_overhead_ratio
        && a.bytes_per_query <= b.bytes_per_query;
    let strictly_better = a.address_fiber_net_gain > b.address_fiber_net_gain
        || a.fiber_ratio_effective_per_byte > b.fiber_ratio_effective_per_byte
        || a.avg_actor_overhead_ratio < b.avg_actor_overhead_ratio
        || a.bytes_per_query < b.bytes_per_query;
    no_worse && strictly_better
}

fn max_by_f64<F>(
    items: &[P67FiberCalibrationConfig],
    mut value: F,
) -> Option<P67FiberCalibrationConfig>
where
    F: FnMut(&P67FiberCalibrationConfig) -> f64,
{
    items
        .iter()
        .max_by(|a, b| {
            value(a)
                .partial_cmp(&value(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
}

fn min_by_f64<F>(
    items: &[P67FiberCalibrationConfig],
    mut value: F,
) -> Option<P67FiberCalibrationConfig>
where
    F: FnMut(&P67FiberCalibrationConfig) -> f64,
{
    items
        .iter()
        .min_by(|a, b| {
            value(a)
                .partial_cmp(&value(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
}

fn global_decision(
    configs: &[P67FiberCalibrationConfig],
    best_balanced: Option<&P67FiberCalibrationConfig>,
) -> P67Decision {
    if configs
        .iter()
        .all(|config| config.decision == P67ConfigDecision::NoGoFiberSafety)
    {
        return P67Decision::NoGoAddressFiberOverhead;
    }
    if let Some(config) = best_balanced {
        if config.conflicts > 0 || config.stale_reads > 0 || config.address_fiber_net_gain < 1.0 {
            return P67Decision::NoGoAddressFiberOverhead;
        }
    }
    P67Decision::RecalibrateFiberOverhead
}

fn global_decision_reasons(
    decision: P67Decision,
    promotion_candidate: bool,
    configs: &[P67FiberCalibrationConfig],
    best_balanced: Option<&P67FiberCalibrationConfig>,
) -> Vec<String> {
    let mut reasons = vec![
        format!("configuration_count: {}", configs.len()),
        "P67 optimizes actor_managed_fiber overhead without changing .atlas grammar".to_string(),
        "all cache, journal, audit, compaction and metadata costs are counted".to_string(),
        "PROMOTE is disabled for a single local-mode report; paired standard/ambitious review is required".to_string(),
        format!("decision: {}", decision.as_str()),
    ];
    if let Some(best) = best_balanced {
        reasons.push(format!("best_balanced: {}", best.config_id));
        reasons.push(format!(
            "best_balanced_net_gain: {:.6}",
            best.address_fiber_net_gain
        ));
        reasons.push(format!(
            "best_balanced_overhead_ratio: {:.6}",
            best.avg_actor_overhead_ratio
        ));
    }
    reasons.push(format!("promotion_candidate: {}", promotion_candidate));
    reasons
}

pub fn p67_fiber_calibration_json(report: &P67FiberCalibrationReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_json_field(&mut out, "astra_step", &report.astra_step, 1, true);
    push_json_field(
        &mut out,
        "calibration_version",
        &report.calibration_version,
        1,
        true,
    );
    push_json_field(&mut out, "program_path", &report.program_path, 1, true);
    push_json_field(&mut out, "mode", &report.mode, 1, true);
    push_json_field(
        &mut out,
        "workload_filter",
        &report.workload_filter,
        1,
        true,
    );
    push_usize_field(&mut out, "runs", report.runs, 1, true);
    push_usize_field(&mut out, "query_count", report.query_count, 1, true);
    push_usize_array(&mut out, "radius_grid", &report.radius_grid, 1, true);
    push_u64_array(&mut out, "budget_grid", &report.budget_grid, 1, true);
    push_string_array(
        &mut out,
        "cache_grid",
        &report
            .cache_grid
            .iter()
            .map(|value| value.as_str().to_string())
            .collect::<Vec<_>>(),
        1,
        true,
    );
    push_string_array(
        &mut out,
        "journal_grid",
        &report
            .journal_grid
            .iter()
            .map(|value| value.as_str().to_string())
            .collect::<Vec<_>>(),
        1,
        true,
    );
    push_string_array(
        &mut out,
        "audit_grid",
        &report
            .audit_grid
            .iter()
            .map(|value| value.as_str().to_string())
            .collect::<Vec<_>>(),
        1,
        true,
    );
    push_string_array(
        &mut out,
        "compaction_grid",
        &report
            .compaction_grid
            .iter()
            .map(|value| value.as_str().to_string())
            .collect::<Vec<_>>(),
        1,
        true,
    );
    push_string_array(
        &mut out,
        "query_locality_grid",
        &report
            .query_locality_grid
            .iter()
            .map(|value| value.as_str().to_string())
            .collect::<Vec<_>>(),
        1,
        true,
    );
    push_string_array(
        &mut out,
        "fiber_projection_grid",
        &report
            .fiber_projection_grid
            .iter()
            .map(|value| value.as_str().to_string())
            .collect::<Vec<_>>(),
        1,
        true,
    );
    push_usize_field(
        &mut out,
        "configuration_count",
        report.configuration_count,
        1,
        true,
    );
    push_config_option(
        &mut out,
        "best_by_ratio",
        report.best_by_ratio.as_ref(),
        1,
        true,
    );
    push_config_option(
        &mut out,
        "best_by_overhead",
        report.best_by_overhead.as_ref(),
        1,
        true,
    );
    push_config_option(
        &mut out,
        "best_by_net_gain",
        report.best_by_net_gain.as_ref(),
        1,
        true,
    );
    push_config_option(
        &mut out,
        "best_balanced",
        report.best_balanced.as_ref(),
        1,
        true,
    );
    out.push_str("  \"pareto_front\": [\n");
    for (idx, config) in report.pareto_front.iter().enumerate() {
        out.push_str(&config_json(config, 2));
        if idx + 1 != report.pareto_front.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ],\n");
    out.push_str("  \"no_go_configs\": [\n");
    for (idx, config) in report.no_go_configs.iter().enumerate() {
        out.push_str(&config_json(config, 2));
        if idx + 1 != report.no_go_configs.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ],\n");
    push_usize_field(
        &mut out,
        "rejected_config_count",
        report.rejected_config_count,
        1,
        true,
    );
    push_bool_field(
        &mut out,
        "promotion_candidate",
        report.promotion_candidate,
        1,
        true,
    );
    push_json_field(&mut out, "decision", report.decision.as_str(), 1, true);
    push_string_array(
        &mut out,
        "decision_reasons",
        &report.decision_reasons,
        1,
        true,
    );
    push_string_array(&mut out, "warnings", &report.warnings, 1, true);
    out.push_str("  \"configurations\": [\n");
    for (idx, config) in report.configurations.iter().enumerate() {
        out.push_str(&config_json(config, 2));
        if idx + 1 != report.configurations.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

pub fn p67_fiber_calibration_markdown(report: &P67FiberCalibrationReport) -> String {
    let best = report.best_balanced.as_ref();
    format!(
        "# ASTRA-P67 fiber overhead calibration\n\n- astra_step: P67\n- mode: {}\n- configurations: {}\n- best_balanced: {}\n- address_fiber_net_gain: {:.6}\n- avg_actor_overhead_ratio: {:.6}\n- fiber_ratio_effective_per_byte: {:.6}\n- promotion_candidate: {}\n- decision: {}\n\n## Limitations\n\n- deterministic internal fixtures only\n- paired standard/ambitious promotion gate is not automatic in this single report\n- no timing golden\n",
        report.mode,
        report.configuration_count,
        best.map(|config| config.config_id.as_str()).unwrap_or("not_available"),
        best.map(|config| config.address_fiber_net_gain).unwrap_or(0.0),
        best.map(|config| config.avg_actor_overhead_ratio).unwrap_or(0.0),
        best.map(|config| config.fiber_ratio_effective_per_byte).unwrap_or(0.0),
        report.promotion_candidate,
        report.decision.as_str()
    )
}

fn p67_fiber_calibration_jsonl(report: &P67FiberCalibrationReport) -> String {
    let mut out = String::new();
    for config in &report.configurations {
        out.push_str(&config_json_compact(config));
        out.push('\n');
    }
    out
}

fn p67_fiber_calibration_csv(report: &P67FiberCalibrationReport) -> String {
    let mut out = String::from(
        "config_id,workload,radius,budget_bytes,cache_policy,journal_policy,audit_policy,compaction_policy,query_locality,fiber_projection_depth,fiber_ratio_effective_per_byte,address_fiber_net_gain,avg_actor_overhead_ratio,cache_hit_rate,bytes_per_query,conflicts,stale_reads,budget_refusals,balanced_score,promotion_candidate,decision\n",
    );
    for config in &report.configurations {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{},{},{},{:.6},{},{}\n",
            config.config_id,
            config.workload,
            config.radius,
            config.budget_bytes,
            config.cache_policy,
            config.journal_policy,
            config.audit_policy,
            config.compaction_policy,
            config.query_locality,
            config.fiber_projection_depth,
            config.fiber_ratio_effective_per_byte,
            config.address_fiber_net_gain,
            config.avg_actor_overhead_ratio,
            config.cache_hit_rate,
            config.bytes_per_query,
            config.conflicts,
            config.stale_reads,
            config.budget_refusals,
            config.balanced_score,
            config.promotion_candidate,
            config.decision.as_str()
        ));
    }
    out
}

fn config_json(config: &P67FiberCalibrationConfig, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let inner = "  ".repeat(indent + 1);
    let mut out = String::new();
    out.push_str(&format!("{}{{\n", pad));
    macro_rules! s {
        ($name:literal, $value:expr, $comma:expr) => {
            out.push_str(&format!(
                "{}\"{}\": \"{}\"{}\n",
                inner,
                $name,
                json_escape($value),
                if $comma { "," } else { "" }
            ));
        };
    }
    macro_rules! n {
        ($name:literal, $value:expr, $comma:expr) => {
            out.push_str(&format!(
                "{}\"{}\": {}{}\n",
                inner,
                $name,
                $value,
                if $comma { "," } else { "" }
            ));
        };
    }
    s!("config_id", &config.config_id, true);
    s!("workload", &config.workload, true);
    n!("radius", config.radius, true);
    n!("budget_bytes", config.budget_bytes, true);
    s!("cache_policy", &config.cache_policy, true);
    s!("journal_policy", &config.journal_policy, true);
    s!("audit_policy", &config.audit_policy, true);
    s!("compaction_policy", &config.compaction_policy, true);
    s!("query_locality", &config.query_locality, true);
    s!(
        "fiber_projection_depth",
        &config.fiber_projection_depth,
        true
    );
    s!("update_rate", &config.update_rate, true);
    s!("metadata_policy", &config.metadata_policy, true);
    n!("fiber_generated_units", config.fiber_generated_units, true);
    n!("fiber_effective_units", config.fiber_effective_units, true);
    n!("fiber_total_bytes", config.fiber_total_bytes, true);
    n!(
        "fiber_ratio_effective_per_byte",
        format!("{:.6}", config.fiber_ratio_effective_per_byte),
        true
    );
    n!(
        "effective_gain_vs_materialized",
        format!("{:.6}", config.effective_gain_vs_materialized),
        true
    );
    n!(
        "address_fiber_net_gain",
        format!("{:.6}", config.address_fiber_net_gain),
        true
    );
    n!(
        "avg_actor_overhead_ratio",
        format!("{:.6}", config.avg_actor_overhead_ratio),
        true
    );
    n!("actor_overhead_bytes", config.actor_overhead_bytes, true);
    n!("fiber_cache_bytes", config.fiber_cache_bytes, true);
    n!("fiber_journal_bytes", config.fiber_journal_bytes, true);
    n!("fiber_audit_bytes", config.fiber_audit_bytes, true);
    n!("fiber_metadata_bytes", config.fiber_metadata_bytes, true);
    n!("update_count", config.update_count, true);
    n!("audit_count", config.audit_count, true);
    n!("compaction_count", config.compaction_count, true);
    n!("eviction_count", config.eviction_count, true);
    n!("conflicts", config.conflicts, true);
    n!("stale_reads", config.stale_reads, true);
    n!("budget_refusals", config.budget_refusals, true);
    n!(
        "budget_refusal_rate",
        format!("{:.6}", config.budget_refusal_rate),
        true
    );
    n!(
        "cache_hit_rate",
        format!("{:.6}", config.cache_hit_rate),
        true
    );
    n!(
        "bytes_per_query",
        format!("{:.6}", config.bytes_per_query),
        true
    );
    n!(
        "generated_units_per_query",
        format!("{:.6}", config.generated_units_per_query),
        true
    );
    n!(
        "balanced_score",
        format!("{:.6}", config.balanced_score),
        true
    );
    n!("promotion_candidate", config.promotion_candidate, true);
    s!("decision", config.decision.as_str(), false);
    out.push_str(&format!("{}}}", pad));
    out
}

fn config_json_compact(config: &P67FiberCalibrationConfig) -> String {
    format!(
        "{{\"config_id\":\"{}\",\"workload\":\"{}\",\"radius\":{},\"budget_bytes\":{},\"cache_policy\":\"{}\",\"journal_policy\":\"{}\",\"audit_policy\":\"{}\",\"compaction_policy\":\"{}\",\"query_locality\":\"{}\",\"fiber_projection_depth\":\"{}\",\"fiber_generated_units\":{},\"fiber_effective_units\":{},\"fiber_total_bytes\":{},\"fiber_ratio_effective_per_byte\":{:.6},\"address_fiber_net_gain\":{:.6},\"avg_actor_overhead_ratio\":{:.6},\"actor_overhead_bytes\":{},\"cache_hit_rate\":{:.6},\"bytes_per_query\":{:.6},\"conflicts\":{},\"stale_reads\":{},\"budget_refusals\":{},\"balanced_score\":{:.6},\"promotion_candidate\":{},\"decision\":\"{}\"}}",
        json_escape(&config.config_id),
        json_escape(&config.workload),
        config.radius,
        config.budget_bytes,
        json_escape(&config.cache_policy),
        json_escape(&config.journal_policy),
        json_escape(&config.audit_policy),
        json_escape(&config.compaction_policy),
        json_escape(&config.query_locality),
        json_escape(&config.fiber_projection_depth),
        config.fiber_generated_units,
        config.fiber_effective_units,
        config.fiber_total_bytes,
        config.fiber_ratio_effective_per_byte,
        config.address_fiber_net_gain,
        config.avg_actor_overhead_ratio,
        config.actor_overhead_bytes,
        config.cache_hit_rate,
        config.bytes_per_query,
        config.conflicts,
        config.stale_reads,
        config.budget_refusals,
        config.balanced_score,
        config.promotion_candidate,
        config.decision.as_str()
    )
}

fn push_config_option(
    out: &mut String,
    name: &str,
    value: Option<&P67FiberCalibrationConfig>,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": ", pad, name));
    if let Some(config) = value {
        out.push_str(&config_json(config, 0));
    } else {
        out.push_str("null");
    }
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn push_json_field(out: &mut String, name: &str, value: &str, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": \"{}\"{}\n",
        pad,
        name,
        json_escape(value),
        if comma { "," } else { "" }
    ));
}

fn push_usize_field(out: &mut String, name: &str, value: usize, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": {}{}\n",
        pad,
        name,
        value,
        if comma { "," } else { "" }
    ));
}

fn push_usize_array(out: &mut String, name: &str, values: &[usize], indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": [", pad, name));
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&value.to_string());
    }
    out.push(']');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn push_u64_array(out: &mut String, name: &str, values: &[u64], indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": [", pad, name));
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&value.to_string());
    }
    out.push(']');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn push_bool_field(out: &mut String, name: &str, value: bool, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": {}{}\n",
        pad,
        name,
        value,
        if comma { "," } else { "" }
    ));
}

fn push_string_array(out: &mut String, name: &str, values: &[String], indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": [", pad, name));
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("\"{}\"", json_escape(value)));
    }
    out.push(']');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn write_file(path: impl AsRef<Path>, content: &str) -> AtlasResult<()> {
    fs::write(path, content).map_err(|err| io_diagnostic(format!("{}", err)))
}

fn io_diagnostic(message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::ParseError, message)
}

fn ratio(numerator: u128, denominator: u128) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    numerator as f64 / denominator as f64
}

fn clamp_u64(value: u128) -> u64 {
    value.min(u64::MAX as u128) as u64
}
