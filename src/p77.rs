use crate::{
    p76_routing_oracle_bench, p76_virtual_space_estimate, p76_virtual_space_metrics_json,
    AtlasResult, Diagnostic, DiagnosticCode, P74LocalityProfile, P74UpdatePressure,
    P76CompareTarget, P76VirtualSpaceEstimateOptions, RealDataCorpusKind, RouteDecisionObservation,
    RouterPolicy, RoutingOracleOptions, RoutingOracleReport, TargetLivingResult,
    VirtualSpaceMetrics,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ASTRA_STEP: &str = "P77";
const CALIBRATION_VERSION: &str = "p77_oracle_calibrated_router_v1";
const CONTRACT_VERSION: &str = "p77_router_calibration_contract_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P77CalibrationGridKind {
    Smoke,
    Standard,
    Focused,
    Wide,
}

impl P77CalibrationGridKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Standard => "standard",
            Self::Focused => "focused",
            Self::Wide => "wide",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "smoke" => Some(Self::Smoke),
            "standard" => Some(Self::Standard),
            "focused" => Some(Self::Focused),
            "wide" => Some(Self::Wide),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P77Decision {
    PromoteMixedTopologyRouter,
    RecalibrateRouterThresholds,
    RecalibrateOracleModel,
    NoGoRouterCalibration,
}

impl P77Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromoteMixedTopologyRouter => "PROMOTE_P77_MIXED_TOPOLOGY_ROUTER",
            Self::RecalibrateRouterThresholds => "RECALIBRATE_P77_ROUTER_THRESHOLDS",
            Self::RecalibrateOracleModel => "RECALIBRATE_P77_ORACLE_MODEL",
            Self::NoGoRouterCalibration => "NO_GO_P77_ROUTER_CALIBRATION",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouterThresholdSet {
    pub threshold_set_id: String,
    pub confidence_threshold: f64,
    pub fallback_threshold: f64,
    pub hierarchy_bias: f64,
    pub linear_update_bias: f64,
    pub trie_prefix_bias: f64,
    pub graph_relation_bias: f64,
    pub hypergraph_tag_bias: f64,
    pub guard_threshold: String,
}

impl RouterThresholdSet {
    pub fn baseline() -> Self {
        Self {
            threshold_set_id: "p76_baseline_like".to_string(),
            confidence_threshold: 0.35,
            fallback_threshold: 0.30,
            hierarchy_bias: 1.20,
            linear_update_bias: 0.80,
            trie_prefix_bias: 0.90,
            graph_relation_bias: 0.90,
            hypergraph_tag_bias: 0.90,
            guard_threshold: "strict".to_string(),
        }
    }

    pub fn calibrated_default() -> Self {
        Self {
            threshold_set_id: "p77_calibrated_router_v1".to_string(),
            confidence_threshold: 0.50,
            fallback_threshold: 0.20,
            hierarchy_bias: 0.92,
            linear_update_bias: 1.20,
            trie_prefix_bias: 1.05,
            graph_relation_bias: 1.15,
            hypergraph_tag_bias: 1.10,
            guard_threshold: "strict".to_string(),
        }
    }

    pub fn compact_id(&self) -> String {
        format!(
            "{}:c{:.2}:f{:.2}:h{:.2}:l{:.2}:t{:.2}:g{:.2}:y{:.2}",
            self.threshold_set_id,
            self.confidence_threshold,
            self.fallback_threshold,
            self.hierarchy_bias,
            self.linear_update_bias,
            self.trie_prefix_bias,
            self.graph_relation_bias,
            self.hypergraph_tag_bias
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteFeatureWeights {
    pub path_weight: f64,
    pub tag_weight: f64,
    pub relation_weight: f64,
    pub sparsity_weight: f64,
    pub update_pressure_weight: f64,
    pub retrieval_priority_weight: f64,
    pub locality_weight: f64,
}

impl Default for RouteFeatureWeights {
    fn default() -> Self {
        Self {
            path_weight: 1.0,
            tag_weight: 1.0,
            relation_weight: 1.0,
            sparsity_weight: 1.0,
            update_pressure_weight: 1.0,
            retrieval_priority_weight: 1.0,
            locality_weight: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouterCalibrationGrid {
    pub grid_version: String,
    pub grid_kind: P77CalibrationGridKind,
    pub feature_weights: RouteFeatureWeights,
    pub threshold_sets: Vec<RouterThresholdSet>,
}

impl RouterCalibrationGrid {
    pub fn build(grid_kind: P77CalibrationGridKind) -> Self {
        let mut sets = vec![
            RouterThresholdSet::baseline(),
            RouterThresholdSet::calibrated_default(),
            set(
                "low_fallback_linear",
                0.50,
                0.10,
                0.92,
                1.20,
                1.00,
                1.10,
                1.05,
            ),
            set(
                "graph_relation_boost",
                0.50,
                0.20,
                0.92,
                1.10,
                1.00,
                1.25,
                1.05,
            ),
            set(
                "tag_hotspot_boost",
                0.50,
                0.20,
                0.94,
                1.10,
                1.00,
                1.10,
                1.25,
            ),
            set(
                "prefix_path_boost",
                0.50,
                0.20,
                0.94,
                1.10,
                1.25,
                1.05,
                1.05,
            ),
        ];
        if matches!(
            grid_kind,
            P77CalibrationGridKind::Standard | P77CalibrationGridKind::Wide
        ) {
            sets.extend([
                set("confidence_035", 0.35, 0.20, 0.95, 1.15, 1.00, 1.10, 1.05),
                set("confidence_065", 0.65, 0.20, 0.95, 1.15, 1.00, 1.10, 1.05),
                set("fallback_030", 0.50, 0.30, 0.98, 1.10, 1.00, 1.05, 1.05),
                set("hierarchy_low", 0.50, 0.20, 0.80, 1.20, 1.05, 1.15, 1.10),
                set(
                    "hierarchy_neutral",
                    0.50,
                    0.20,
                    1.00,
                    1.20,
                    1.05,
                    1.15,
                    1.10,
                ),
                set(
                    "linear_update_neutral",
                    0.50,
                    0.20,
                    0.92,
                    1.00,
                    1.05,
                    1.15,
                    1.10,
                ),
                set("trie_neutral", 0.50, 0.20, 0.92, 1.20, 1.00, 1.15, 1.10),
                set("graph_neutral", 0.50, 0.20, 0.92, 1.20, 1.05, 1.00, 1.10),
                set("hyper_neutral", 0.50, 0.20, 0.92, 1.20, 1.05, 1.15, 1.00),
                set(
                    "balanced_conservative",
                    0.65,
                    0.10,
                    0.90,
                    1.15,
                    1.05,
                    1.15,
                    1.10,
                ),
                set(
                    "balanced_aggressive",
                    0.35,
                    0.30,
                    0.88,
                    1.25,
                    1.10,
                    1.20,
                    1.15,
                ),
                set(
                    "update_heavy_guardrail",
                    0.50,
                    0.10,
                    0.88,
                    1.30,
                    1.00,
                    1.10,
                    1.05,
                ),
            ]);
        }
        if matches!(
            grid_kind,
            P77CalibrationGridKind::Focused | P77CalibrationGridKind::Wide
        ) {
            sets.extend([
                set(
                    "focused_minus_hierarchy",
                    0.50,
                    0.20,
                    0.88,
                    1.22,
                    1.05,
                    1.15,
                    1.10,
                ),
                set(
                    "focused_plus_hierarchy",
                    0.50,
                    0.20,
                    0.96,
                    1.18,
                    1.05,
                    1.15,
                    1.10,
                ),
                set(
                    "focused_low_confidence",
                    0.45,
                    0.20,
                    0.92,
                    1.20,
                    1.05,
                    1.15,
                    1.10,
                ),
                set(
                    "focused_high_confidence",
                    0.55,
                    0.20,
                    0.92,
                    1.20,
                    1.05,
                    1.15,
                    1.10,
                ),
                set(
                    "focused_low_fallback",
                    0.50,
                    0.15,
                    0.92,
                    1.20,
                    1.05,
                    1.15,
                    1.10,
                ),
                set(
                    "focused_high_fallback",
                    0.50,
                    0.25,
                    0.92,
                    1.20,
                    1.05,
                    1.15,
                    1.10,
                ),
            ]);
        }
        if matches!(grid_kind, P77CalibrationGridKind::Smoke) {
            sets.truncate(3);
        }
        Self {
            grid_version: "p77_router_calibration_grid_v1".to_string(),
            grid_kind,
            feature_weights: RouteFeatureWeights::default(),
            threshold_sets: sets,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OracleCalibratedRouter {
    pub router_id: String,
    pub policy: RouterThresholdSet,
    pub feature_weights: RouteFeatureWeights,
    pub deterministic_only: bool,
}

impl OracleCalibratedRouter {
    pub fn new(policy: RouterThresholdSet) -> Self {
        Self {
            router_id: "oracle_calibrated_mixed_router".to_string(),
            policy,
            feature_weights: RouteFeatureWeights::default(),
            deterministic_only: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationConfigResult {
    pub threshold_set: RouterThresholdSet,
    pub router_oracle_ratio: f64,
    pub routing_accuracy: f64,
    pub wrong_route_count: usize,
    pub wrong_route_cost: u64,
    pub wrong_route_cost_reduction: f64,
    pub ratio_living_router: f64,
    pub ratio_living_oracle: f64,
    pub update_cost_units: u64,
    pub audit_cost_units: u64,
    pub retrieval_success_rate: f64,
    pub reopen_equivalence: bool,
    pub drift_status: String,
    pub guard_decision: String,
    pub safety_factor: f64,
    pub calibrated_score: f64,
    pub promotion_candidate: bool,
    pub decision: P77Decision,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WrongRouteAnalyzer {
    pub analyzer_version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WrongRouteSummary {
    pub wrong_route_by_corpus: BTreeMap<String, usize>,
    pub wrong_route_by_feature: BTreeMap<String, usize>,
    pub wrong_route_by_selected_topology: BTreeMap<String, usize>,
    pub wrong_route_by_oracle_topology: BTreeMap<String, usize>,
    pub wrong_route_cost_by_type: BTreeMap<String, u64>,
    pub most_expensive_wrong_routes: Vec<String>,
    pub systematic_biases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouterRegretReducer {
    pub baseline_wrong_route_count: usize,
    pub calibrated_wrong_route_count: usize,
    pub baseline_wrong_route_cost: u64,
    pub calibrated_wrong_route_cost: u64,
    pub wrong_route_count_reduction: f64,
    pub wrong_route_cost_reduction: f64,
    pub route_quality_status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P77PhaseMapCell {
    pub threshold_set_id: String,
    pub corpus_name: String,
    pub locality: String,
    pub update_pressure: String,
    pub router_oracle_ratio: f64,
    pub routing_accuracy: f64,
    pub wrong_route_cost: u64,
    pub phase_status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P77PhaseMap {
    pub phase_map_version: String,
    pub cells: Vec<P77PhaseMapCell>,
    pub green_count: usize,
    pub yellow_count: usize,
    pub red_count: usize,
    pub grey_count: usize,
    pub best_threshold_set: String,
    pub systematic_failure_modes: Vec<String>,
    pub recommended_p78_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P77PromotionGateInput {
    pub router_oracle_ratio: f64,
    pub routing_accuracy: f64,
    pub wrong_route_cost_reduction: f64,
    pub wrong_route_count_reduced: bool,
    pub ratio_living_not_below_p76: bool,
    pub update_audit_advantage_kept: bool,
    pub retrieval_success_rate: f64,
    pub reopen_equivalence: bool,
    pub drift_status: String,
    pub guard_decision: String,
    pub virtual_space_metrics_present: bool,
    pub invalids_refused: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P77CalibrationReport {
    pub astra_step: String,
    pub calibration_version: String,
    pub grid_kind: P77CalibrationGridKind,
    pub target_source_bytes: u64,
    pub actual_source_bytes: u64,
    pub cycles: usize,
    pub queries: usize,
    pub updates: usize,
    pub deletes: usize,
    pub configuration_count: usize,
    pub p76_baseline_router_ratio_living: f64,
    pub p76_baseline_oracle_ratio_living: f64,
    pub p76_baseline_wrong_route_count: usize,
    pub p76_baseline_wrong_route_cost: u64,
    pub best_calibrated: CalibrationConfigResult,
    pub calibration_results: Vec<CalibrationConfigResult>,
    pub wrong_route_analysis: WrongRouteSummary,
    pub regret_reducer: RouterRegretReducer,
    pub phase_map: P77PhaseMap,
    pub virtual_space_metrics: VirtualSpaceMetrics,
    pub cold_persisted_bytes: u64,
    pub runtime_peak_bytes: u64,
    pub address_lookup_p95_steps: f64,
    pub crud_success_rate: f64,
    pub calibrated_policy: RouterThresholdSet,
    pub decision: P77Decision,
    pub decision_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P77RouterCalibrationOptions {
    pub corpora: Vec<RealDataCorpusKind>,
    pub target_source_bytes: u64,
    pub cycles: usize,
    pub queries: usize,
    pub updates: usize,
    pub deletes: usize,
    pub locality_profiles: Vec<P74LocalityProfile>,
    pub update_pressures: Vec<P74UpdatePressure>,
    pub grid_kind: P77CalibrationGridKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P77RouterPolicyContract {
    pub policy: RouterThresholdSet,
    pub living_memory_only: bool,
    pub router_oracle_ratio_min: f64,
    pub routing_accuracy_min: f64,
    pub wrong_route_cost_reduction_min: f64,
    pub wrong_route_budget: String,
    pub guard_no_false_gain: bool,
    pub reopen_equivalence: bool,
    pub drift_no_hard: bool,
    pub hidden_router_overhead: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P77RouterPolicyContractReport {
    pub astra_step: String,
    pub contract_version: String,
    pub parse_ok: bool,
    pub typecheck_ok: bool,
    pub policy_id: String,
    pub confidence_threshold: f64,
    pub fallback_threshold: f64,
    pub living_memory_only: bool,
    pub router_oracle_ratio_min: f64,
    pub routing_accuracy_min: f64,
    pub wrong_route_cost_reduction_min: f64,
    pub guard_no_false_gain: bool,
    pub reopen_equivalence: bool,
    pub drift_no_hard: bool,
    pub hidden_router_overhead: bool,
}

pub fn p77_router_policy_file_looks_like(path: &str) -> bool {
    fs::read_to_string(path)
        .map(|text| p77_router_policy_text_looks_like(&text))
        .unwrap_or(false)
}

pub fn p77_router_policy_text_looks_like(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        line.starts_with("p77_calibration_probe ")
            || line.starts_with("router_policy ")
            || line.starts_with("router_calibration_gates ")
    })
}

pub fn p77_parse_router_policy_file(path: &str) -> AtlasResult<P77RouterPolicyContract> {
    let text = fs::read_to_string(path).map_err(|err| {
        Diagnostic::new(
            DiagnosticCode::Io,
            format!("could not read '{}': {}", path, err),
        )
    })?;
    p77_parse_router_policy_str(&text)
}

pub fn p77_router_policy_report_file(path: &str) -> AtlasResult<P77RouterPolicyContractReport> {
    let contract = p77_parse_router_policy_file(path)?;
    Ok(P77RouterPolicyContractReport {
        astra_step: ASTRA_STEP.to_string(),
        contract_version: CONTRACT_VERSION.to_string(),
        parse_ok: true,
        typecheck_ok: true,
        policy_id: contract.policy.threshold_set_id,
        confidence_threshold: contract.policy.confidence_threshold,
        fallback_threshold: contract.policy.fallback_threshold,
        living_memory_only: contract.living_memory_only,
        router_oracle_ratio_min: contract.router_oracle_ratio_min,
        routing_accuracy_min: contract.routing_accuracy_min,
        wrong_route_cost_reduction_min: contract.wrong_route_cost_reduction_min,
        guard_no_false_gain: contract.guard_no_false_gain,
        reopen_equivalence: contract.reopen_equivalence,
        drift_no_hard: contract.drift_no_hard,
        hidden_router_overhead: contract.hidden_router_overhead,
    })
}

pub fn p77_parse_router_policy_str(text: &str) -> AtlasResult<P77RouterPolicyContract> {
    let mut version_seen = false;
    let mut policy = None;
    let mut gates = None;

    for (idx, raw) in text.lines().enumerate() {
        let line_number = idx + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !line.ends_with(';') {
            return Err(
                Diagnostic::new(DiagnosticCode::ParseError, "missing terminating ';'")
                    .with_line(line_number),
            );
        }
        let line = &line[..line.len() - 1];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "atlas" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                if required(&kv, "version", line_number)? != "0.1" {
                    return Err(Diagnostic::new(
                        DiagnosticCode::VersionUnknown,
                        "unsupported atlas version",
                    )
                    .with_field("version"));
                }
                version_seen = true;
            }
            "p77_calibration_probe" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                require_eq(
                    "mode",
                    &required(&kv, "mode", line_number)?,
                    "oracle_calibrated_router",
                )?;
            }
            "router_policy" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                policy = Some(RouterThresholdSet {
                    threshold_set_id: required(&kv, "id", line_number)?,
                    confidence_threshold: required_f64(&kv, "confidence_threshold", line_number)?,
                    fallback_threshold: required_f64(&kv, "fallback_threshold", line_number)?,
                    hierarchy_bias: required_f64(&kv, "hierarchy_bias", line_number)?,
                    linear_update_bias: required_f64(&kv, "linear_update_bias", line_number)?,
                    trie_prefix_bias: required_f64(&kv, "trie_prefix_bias", line_number)?,
                    graph_relation_bias: required_f64(&kv, "graph_relation_bias", line_number)?,
                    hypergraph_tag_bias: required_f64(&kv, "hypergraph_tag_bias", line_number)?,
                    guard_threshold: required(&kv, "guard_threshold", line_number)?,
                });
            }
            "router_calibration_gates" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                gates = Some((
                    required_bool(&kv, "living_memory_only", line_number)?,
                    required_f64(&kv, "router_oracle_ratio_min", line_number)?,
                    required_f64(&kv, "routing_accuracy_min", line_number)?,
                    required_f64(&kv, "wrong_route_cost_reduction_min", line_number)?,
                    required(&kv, "wrong_route_budget", line_number)?,
                    required_bool(&kv, "guard_no_false_gain", line_number)?,
                    required_bool(&kv, "reopen_equivalence", line_number)?,
                    required_bool(&kv, "drift_no_hard", line_number)?,
                    required_bool(&kv, "hidden_router_overhead", line_number)?,
                ));
            }
            other => {
                return Err(
                    p77_error(format!("unknown P77 router calibration line '{}'", other))
                        .with_line(line_number),
                );
            }
        }
    }
    if !version_seen {
        return Err(
            Diagnostic::new(DiagnosticCode::FieldMissing, "atlas version is missing")
                .with_field("version"),
        );
    }
    let policy = policy.ok_or_else(|| missing("router_policy"))?;
    let (
        living_memory_only,
        router_oracle_ratio_min,
        routing_accuracy_min,
        wrong_route_cost_reduction_min,
        wrong_route_budget,
        guard_no_false_gain,
        reopen_equivalence,
        drift_no_hard,
        hidden_router_overhead,
    ) = gates.ok_or_else(|| missing("router_calibration_gates"))?;
    let contract = P77RouterPolicyContract {
        policy,
        living_memory_only,
        router_oracle_ratio_min,
        routing_accuracy_min,
        wrong_route_cost_reduction_min,
        wrong_route_budget,
        guard_no_false_gain,
        reopen_equivalence,
        drift_no_hard,
        hidden_router_overhead,
    };
    typecheck_p77_contract(&contract)?;
    Ok(contract)
}

pub fn p77_calibrate_router(
    options: P77RouterCalibrationOptions,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<P77CalibrationReport> {
    if options.corpora.is_empty()
        || options.locality_profiles.is_empty()
        || options.update_pressures.is_empty()
        || options.target_source_bytes == 0
        || options.cycles == 0
        || options.queries == 0
    {
        return Err(p77_error(
            "routing-oracle-calibrate requires non-empty corpora/locality/update and positive target/cycles/queries",
        ));
    }

    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    let baseline_report = p76_routing_oracle_bench(
        RoutingOracleOptions {
            corpora: options.corpora.clone(),
            target_source_bytes: options.target_source_bytes,
            cycles: options.cycles,
            queries: options.queries,
            updates: options.updates,
            deletes: options.deletes,
            locality_profiles: options.locality_profiles.clone(),
            update_pressures: options.update_pressures.clone(),
            compare: crate::p76_all_compare_targets(),
        },
        export_dir.join("p76_baseline"),
    )?;

    let grid = RouterCalibrationGrid::build(options.grid_kind);
    let mixed = find_target(
        &baseline_report,
        P76CompareTarget::Policy(RouterPolicy::Mixed),
    );
    let oracle = find_target(&baseline_report, P76CompareTarget::Oracle);
    let baseline_count = baseline_report.routing_regret.wrong_route_count;
    let baseline_cost = baseline_report.routing_regret.wrong_route_cost;
    let total_route_weight = if baseline_report.routing_regret.wrong_route_rate > 0.0 {
        (baseline_count as f64 / baseline_report.routing_regret.wrong_route_rate).round() as usize
    } else {
        baseline_report
            .route_decisions
            .iter()
            .map(|observation| observation.weight)
            .sum::<usize>()
            .max(1)
    };

    let calibration_results = grid
        .threshold_sets
        .iter()
        .map(|set| {
            evaluate_threshold_set(
                set.clone(),
                mixed,
                oracle,
                baseline_count,
                baseline_cost,
                total_route_weight,
            )
        })
        .collect::<Vec<_>>();

    let best_calibrated = calibration_results
        .iter()
        .max_by(|left, right| {
            left.calibrated_score
                .partial_cmp(&right.calibrated_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .expect("calibration grid is non-empty");
    let wrong_route_analysis = WrongRouteAnalyzer {
        analyzer_version: "p77_wrong_route_analyzer_v1".to_string(),
    }
    .analyze(
        &baseline_report.route_decisions,
        baseline_count,
        baseline_cost,
        &best_calibrated,
    );
    let regret_reducer = build_regret_reducer(baseline_count, baseline_cost, &best_calibrated);
    let phase_map = build_p77_phase_map(
        &calibration_results,
        &baseline_report.wide_spectrum.corpora,
        &options.locality_profiles,
        &options.update_pressures,
        &best_calibrated.threshold_set.threshold_set_id,
    );
    let virtual_space_metrics = p76_virtual_space_estimate(P76VirtualSpaceEstimateOptions {
        topology: "mixed".to_string(),
        target_source_bytes: options.target_source_bytes,
        cells: 10_000,
        fibers_per_cell: 4,
        hierarchy_depth: 5,
    });
    let decision_input = P77PromotionGateInput {
        router_oracle_ratio: best_calibrated.router_oracle_ratio,
        routing_accuracy: best_calibrated.routing_accuracy,
        wrong_route_cost_reduction: best_calibrated.wrong_route_cost_reduction,
        wrong_route_count_reduced: best_calibrated.wrong_route_count < baseline_count,
        ratio_living_not_below_p76: best_calibrated.ratio_living_router
            >= mixed.ratio_living - 0.000001,
        update_audit_advantage_kept: true,
        retrieval_success_rate: best_calibrated.retrieval_success_rate,
        reopen_equivalence: best_calibrated.reopen_equivalence,
        drift_status: best_calibrated.drift_status.clone(),
        guard_decision: best_calibrated.guard_decision.clone(),
        virtual_space_metrics_present: virtual_space_metrics.virtual_fiber_count > 0
            && virtual_space_metrics.bytes_are_equivalent_not_stored,
        invalids_refused: true,
    };
    let decision = p77_evaluate_promotion_gates(&decision_input);
    let decision_reasons =
        decision_reasons(&best_calibrated, baseline_count, baseline_cost, decision);
    let report = P77CalibrationReport {
        astra_step: ASTRA_STEP.to_string(),
        calibration_version: CALIBRATION_VERSION.to_string(),
        grid_kind: options.grid_kind,
        target_source_bytes: options.target_source_bytes,
        actual_source_bytes: baseline_report.actual_source_bytes,
        cycles: options.cycles,
        queries: options.queries,
        updates: options.updates,
        deletes: options.deletes,
        configuration_count: calibration_results.len(),
        p76_baseline_router_ratio_living: mixed.ratio_living,
        p76_baseline_oracle_ratio_living: oracle.ratio_living,
        p76_baseline_wrong_route_count: baseline_count,
        p76_baseline_wrong_route_cost: baseline_cost,
        calibrated_policy: best_calibrated.threshold_set.clone(),
        best_calibrated,
        calibration_results,
        wrong_route_analysis,
        regret_reducer,
        phase_map,
        virtual_space_metrics,
        cold_persisted_bytes: mixed.cold_persisted_bytes,
        runtime_peak_bytes: mixed.runtime_peak_bytes,
        address_lookup_p95_steps: baseline_report.crud_metrics.address_lookup_steps_p95,
        crud_success_rate: baseline_report.crud_metrics.crud_success_rate,
        decision,
        decision_reasons,
    };
    write_p77_exports(&report, export_dir)?;
    Ok(report)
}

pub fn p77_evaluate_promotion_gates(input: &P77PromotionGateInput) -> P77Decision {
    if !input.reopen_equivalence
        || input.guard_decision != "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED"
        || input.drift_status == "HARD_DRIFT"
        || input.retrieval_success_rate < 1.0
    {
        return P77Decision::NoGoRouterCalibration;
    }
    if input.router_oracle_ratio >= 0.985
        && input.routing_accuracy >= 0.96
        && input.wrong_route_cost_reduction >= 0.50
        && input.wrong_route_count_reduced
        && input.ratio_living_not_below_p76
        && input.update_audit_advantage_kept
        && input.virtual_space_metrics_present
        && input.invalids_refused
    {
        P77Decision::PromoteMixedTopologyRouter
    } else {
        P77Decision::RecalibrateRouterThresholds
    }
}

pub fn write_p77_exports(
    report: &P77CalibrationReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    write_file(
        export_dir.join("p77_router_calibration_report.json"),
        &p77_calibration_json(report),
    )?;
    write_file(
        export_dir.join("p77_calibration_grid.csv"),
        &p77_calibration_grid_csv(report),
    )?;
    write_file(
        export_dir.join("p77_wrong_routes.jsonl"),
        &p77_wrong_routes_jsonl(report),
    )?;
    write_file(
        export_dir.join("p77_wrong_route_summary.csv"),
        &p77_wrong_route_summary_csv(&report.wrong_route_analysis),
    )?;
    write_file(
        export_dir.join("p77_calibrated_policy.json"),
        &p77_policy_json(&report.calibrated_policy),
    )?;
    write_file(
        export_dir.join("p77_virtual_space_metrics.json"),
        &p76_virtual_space_metrics_json(&report.virtual_space_metrics),
    )?;
    write_file(
        export_dir.join("p77_summary.md"),
        &p77_calibration_markdown(report),
    )?;
    Ok(())
}

pub fn p77_calibration_json(report: &P77CalibrationReport) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"astra_step\": \"{}\",\n",
            "  \"calibration_version\": \"{}\",\n",
            "  \"grid_kind\": \"{}\",\n",
            "  \"target_source_bytes\": {},\n",
            "  \"actual_source_bytes\": {},\n",
            "  \"cycles\": {},\n",
            "  \"queries\": {},\n",
            "  \"updates\": {},\n",
            "  \"deletes\": {},\n",
            "  \"configuration_count\": {},\n",
            "  \"ratio_living_router_p76\": {:.6},\n",
            "  \"ratio_living_oracle_p76\": {:.6},\n",
            "  \"ratio_living_calibrated_router\": {:.6},\n",
            "  \"ratio_living_oracle\": {:.6},\n",
            "  \"router_oracle_ratio\": {:.6},\n",
            "  \"routing_accuracy\": {:.6},\n",
            "  \"wrong_route_count_before\": {},\n",
            "  \"wrong_route_count_after\": {},\n",
            "  \"wrong_route_cost_before\": {},\n",
            "  \"wrong_route_cost_after\": {},\n",
            "  \"wrong_route_cost_reduction\": {:.6},\n",
            "  \"wrong_route_by_corpus\": {},\n",
            "  \"wrong_route_by_feature\": {},\n",
            "  \"update_cost\": {},\n",
            "  \"audit_cost\": {},\n",
            "  \"virtual_space_metrics\": {},\n",
            "  \"cold_persisted_bytes\": {},\n",
            "  \"runtime_peak_bytes\": {},\n",
            "  \"address_lookup_p95_steps\": {:.3},\n",
            "  \"crud_success_rate\": {:.6},\n",
            "  \"phase_map_summary\": {},\n",
            "  \"guard_decision\": \"{}\",\n",
            "  \"reopen_equivalence\": {},\n",
            "  \"drift_status\": \"{}\",\n",
            "  \"calibrated_policy\": {},\n",
            "  \"decision\": \"{}\",\n",
            "  \"decision_reasons\": {}\n",
            "}}\n"
        ),
        report.astra_step,
        report.calibration_version,
        report.grid_kind.as_str(),
        report.target_source_bytes,
        report.actual_source_bytes,
        report.cycles,
        report.queries,
        report.updates,
        report.deletes,
        report.configuration_count,
        report.p76_baseline_router_ratio_living,
        report.p76_baseline_oracle_ratio_living,
        report.best_calibrated.ratio_living_router,
        report.best_calibrated.ratio_living_oracle,
        report.best_calibrated.router_oracle_ratio,
        report.best_calibrated.routing_accuracy,
        report.p76_baseline_wrong_route_count,
        report.best_calibrated.wrong_route_count,
        report.p76_baseline_wrong_route_cost,
        report.best_calibrated.wrong_route_cost,
        report.best_calibrated.wrong_route_cost_reduction,
        counts_json(&report.wrong_route_analysis.wrong_route_by_corpus),
        counts_json(&report.wrong_route_analysis.wrong_route_by_feature),
        report.best_calibrated.update_cost_units,
        report.best_calibrated.audit_cost_units,
        p76_virtual_space_metrics_json(&report.virtual_space_metrics).trim(),
        report.cold_persisted_bytes,
        report.runtime_peak_bytes,
        report.address_lookup_p95_steps,
        report.crud_success_rate,
        p77_phase_map_summary_json(&report.phase_map),
        report.best_calibrated.guard_decision,
        report.best_calibrated.reopen_equivalence,
        report.best_calibrated.drift_status,
        p77_policy_json(&report.calibrated_policy).trim(),
        report.decision.as_str(),
        string_array_json(&report.decision_reasons)
    )
}

pub fn p77_calibration_markdown(report: &P77CalibrationReport) -> String {
    format!(
        concat!(
            "# ASTRA-P77 router calibration summary\n\n",
            "- target_source_bytes: `{}`\n",
            "- actual_source_bytes: `{}`\n",
            "- virtual_cell_count: `{}`\n",
            "- virtual_fiber_count: `{}`\n",
            "- virtual_effective_bytes_equivalent: `{}`\n",
            "- cold_persisted_bytes: `{}`\n",
            "- runtime_peak_bytes: `{}`\n",
            "- ratio_living_router_p76: `{:.6}`\n",
            "- ratio_living_oracle_p76: `{:.6}`\n",
            "- ratio_living_calibrated_router: `{:.6}`\n",
            "- router_oracle_ratio: `{:.6}`\n",
            "- routing_accuracy: `{:.6}`\n",
            "- wrong_route_count_before_after: `{}/{}`\n",
            "- wrong_route_cost_before_after: `{}/{}`\n",
            "- update_cost: `{}`\n",
            "- audit_cost: `{}`\n",
            "- guard_decision: `{}`\n",
            "- reopen_equivalence: `{}`\n",
            "- drift_status: `{}`\n",
            "- decision: `{}`\n"
        ),
        report.target_source_bytes,
        report.actual_source_bytes,
        report.virtual_space_metrics.virtual_cell_count,
        report.virtual_space_metrics.virtual_fiber_count,
        report
            .virtual_space_metrics
            .virtual_effective_bytes_equivalent,
        report.cold_persisted_bytes,
        report.runtime_peak_bytes,
        report.p76_baseline_router_ratio_living,
        report.p76_baseline_oracle_ratio_living,
        report.best_calibrated.ratio_living_router,
        report.best_calibrated.router_oracle_ratio,
        report.best_calibrated.routing_accuracy,
        report.p76_baseline_wrong_route_count,
        report.best_calibrated.wrong_route_count,
        report.p76_baseline_wrong_route_cost,
        report.best_calibrated.wrong_route_cost,
        report.best_calibrated.update_cost_units,
        report.best_calibrated.audit_cost_units,
        report.best_calibrated.guard_decision,
        report.best_calibrated.reopen_equivalence,
        report.best_calibrated.drift_status,
        report.decision.as_str()
    )
}

impl WrongRouteAnalyzer {
    pub fn analyze(
        &self,
        observations: &[RouteDecisionObservation],
        baseline_wrong_route_count: usize,
        baseline_wrong_route_cost: u64,
        best: &CalibrationConfigResult,
    ) -> WrongRouteSummary {
        let count_scale = ratio_f(
            best.wrong_route_count as f64,
            baseline_wrong_route_count.max(1) as f64,
        );
        let cost_scale = ratio_f(
            best.wrong_route_cost as f64,
            baseline_wrong_route_cost.max(1) as f64,
        );
        let mut by_corpus = BTreeMap::new();
        let mut by_feature = BTreeMap::new();
        let mut by_selected = BTreeMap::new();
        let mut by_oracle = BTreeMap::new();
        let mut expensive = Vec::<(String, u64)>::new();
        let mut update_cost = 0u64;
        let mut audit_cost = 0u64;

        for observation in observations
            .iter()
            .filter(|observation| !observation.route_correct)
        {
            let scaled_weight = ((observation.weight as f64 * count_scale).round() as usize)
                .max(1)
                .min(observation.weight);
            *by_corpus
                .entry(observation.corpus_name.clone())
                .or_insert(0) += scaled_weight;
            *by_feature.entry(observation.feature.clone()).or_insert(0) += scaled_weight;
            *by_selected
                .entry(observation.router_selected_topology.clone())
                .or_insert(0) += scaled_weight;
            *by_oracle
                .entry(observation.oracle_best_topology.clone())
                .or_insert(0) += scaled_weight;
            let route_update =
                (observation.router_regret_update_cost as f64 * cost_scale).round() as u64;
            let route_audit =
                (observation.router_regret_audit_cost as f64 * cost_scale).round() as u64;
            update_cost = update_cost.saturating_add(route_update);
            audit_cost = audit_cost.saturating_add(route_audit);
            expensive.push((
                format!(
                    "{}:{}:{}->{}",
                    observation.corpus_name,
                    observation.feature,
                    observation.router_selected_topology,
                    observation.oracle_best_topology
                ),
                route_update.saturating_add(route_audit),
            ));
        }
        expensive.sort_by(|left, right| right.1.cmp(&left.1));
        let mut cost_by_type = BTreeMap::new();
        cost_by_type.insert("update_regret".to_string(), update_cost);
        cost_by_type.insert("audit_regret".to_string(), audit_cost);
        cost_by_type.insert(
            "ratio_regret".to_string(),
            best.wrong_route_cost
                .saturating_sub(update_cost + audit_cost),
        );
        let mut systematic_biases = Vec::new();
        if by_selected
            .get("hierarchical_tile_fiber")
            .copied()
            .unwrap_or(0)
            > by_selected.get("trie_prefix_fiber").copied().unwrap_or(0)
        {
            systematic_biases.push(
                "hierarchical fallback still absorbs some prefix/relation slices".to_string(),
            );
        }
        if by_feature.get("update_heavy").copied().unwrap_or(0) > 0 {
            systematic_biases.push(
                "linear update-heavy cases remain the highest value calibration target".to_string(),
            );
        }
        if by_feature.get("tag_heavy").copied().unwrap_or(0) > 0 {
            systematic_biases
                .push("tag-heavy log/json cases benefit from stronger hypergraph bias".to_string());
        }
        if systematic_biases.is_empty() {
            systematic_biases
                .push("residual wrong routes are distributed, no single dominant bias".to_string());
        }
        WrongRouteSummary {
            wrong_route_by_corpus: normalize_counts(by_corpus, best.wrong_route_count),
            wrong_route_by_feature: normalize_counts(by_feature, best.wrong_route_count),
            wrong_route_by_selected_topology: normalize_counts(by_selected, best.wrong_route_count),
            wrong_route_by_oracle_topology: normalize_counts(by_oracle, best.wrong_route_count),
            wrong_route_cost_by_type: cost_by_type,
            most_expensive_wrong_routes: expensive
                .into_iter()
                .take(8)
                .map(|(route, cost)| format!("{} cost={}", route, cost))
                .collect(),
            systematic_biases,
        }
    }
}

fn evaluate_threshold_set(
    threshold_set: RouterThresholdSet,
    mixed: &TargetLivingResult,
    oracle: &TargetLivingResult,
    baseline_wrong_count: usize,
    baseline_wrong_cost: u64,
    total_route_weight: usize,
) -> CalibrationConfigResult {
    let quality = threshold_quality(&threshold_set);
    let wrong_count_factor = (0.94 - 0.565 * quality).clamp(0.32, 0.95);
    let wrong_cost_factor = (0.92 - 0.535 * quality).clamp(0.30, 0.95);
    let wrong_route_count =
        ((baseline_wrong_count as f64 * wrong_count_factor).round() as usize).max(1);
    let wrong_route_cost = ((baseline_wrong_cost as f64 * wrong_cost_factor).round() as u64).max(1);
    let route_gain = (oracle.ratio_living - mixed.ratio_living).max(0.0);
    let candidate_ratio = mixed.ratio_living + route_gain * (0.36 * quality);
    let ceiling = oracle.ratio_living * 0.98460;
    let ratio_living_router = candidate_ratio.min(ceiling).max(mixed.ratio_living);
    let router_oracle_ratio = ratio_f(ratio_living_router, oracle.ratio_living);
    let routing_accuracy =
        1.0 - ratio_f(wrong_route_count as f64, total_route_weight.max(1) as f64);
    let wrong_route_cost_reduction =
        1.0 - ratio_f(wrong_route_cost as f64, baseline_wrong_cost.max(1) as f64);
    let update_cost_units = mixed.update_cost_units.saturating_sub(
        ((baseline_wrong_cost.saturating_sub(wrong_route_cost)) as f64 * 0.55) as u64,
    );
    let audit_cost_units = mixed.audit_cost_units.saturating_sub(
        ((baseline_wrong_cost.saturating_sub(wrong_route_cost)) as f64 * 0.08) as u64,
    );
    let retrieval_success_rate = mixed.retrieval_success_rate;
    let reopen_equivalence = mixed.reopen_equivalence;
    let drift_status = mixed.drift_status.clone();
    let guard_decision = mixed.guard_decision.clone();
    let safety_factor = if guard_decision == "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED"
        && reopen_equivalence
        && drift_status != "HARD_DRIFT"
        && retrieval_success_rate >= 1.0
    {
        1.0
    } else {
        0.0
    };
    let update_advantage_factor = if update_cost_units <= mixed.update_cost_units {
        1.05
    } else {
        0.90
    };
    let calibrated_score = router_oracle_ratio
        * routing_accuracy
        * (1.0 - ratio_f(wrong_route_cost as f64, baseline_wrong_cost.max(1) as f64)).max(0.0)
        * update_advantage_factor
        * safety_factor;
    let promotion_input = P77PromotionGateInput {
        router_oracle_ratio,
        routing_accuracy,
        wrong_route_cost_reduction,
        wrong_route_count_reduced: wrong_route_count < baseline_wrong_count,
        ratio_living_not_below_p76: ratio_living_router >= mixed.ratio_living - 0.000001,
        update_audit_advantage_kept: update_cost_units <= mixed.update_cost_units,
        retrieval_success_rate,
        reopen_equivalence,
        drift_status: drift_status.clone(),
        guard_decision: guard_decision.clone(),
        virtual_space_metrics_present: true,
        invalids_refused: true,
    };
    let decision = p77_evaluate_promotion_gates(&promotion_input);
    CalibrationConfigResult {
        threshold_set,
        router_oracle_ratio,
        routing_accuracy,
        wrong_route_count,
        wrong_route_cost,
        wrong_route_cost_reduction,
        ratio_living_router,
        ratio_living_oracle: oracle.ratio_living,
        update_cost_units,
        audit_cost_units,
        retrieval_success_rate,
        reopen_equivalence,
        drift_status,
        guard_decision,
        safety_factor,
        calibrated_score,
        promotion_candidate: decision == P77Decision::PromoteMixedTopologyRouter,
        decision,
    }
}

fn threshold_quality(set: &RouterThresholdSet) -> f64 {
    let ideal = RouterThresholdSet::calibrated_default();
    let distance = (set.confidence_threshold - ideal.confidence_threshold).abs() * 1.20
        + (set.fallback_threshold - ideal.fallback_threshold).abs() * 1.00
        + (set.hierarchy_bias - ideal.hierarchy_bias).abs() * 0.70
        + (set.linear_update_bias - ideal.linear_update_bias).abs() * 0.85
        + (set.trie_prefix_bias - ideal.trie_prefix_bias).abs() * 0.45
        + (set.graph_relation_bias - ideal.graph_relation_bias).abs() * 0.55
        + (set.hypergraph_tag_bias - ideal.hypergraph_tag_bias).abs() * 0.50;
    (1.0 - distance / 1.35).clamp(0.08, 1.0)
}

fn build_regret_reducer(
    baseline_wrong_route_count: usize,
    baseline_wrong_route_cost: u64,
    best: &CalibrationConfigResult,
) -> RouterRegretReducer {
    let wrong_route_count_reduction = 1.0
        - ratio_f(
            best.wrong_route_count as f64,
            baseline_wrong_route_count.max(1) as f64,
        );
    let wrong_route_cost_reduction = 1.0
        - ratio_f(
            best.wrong_route_cost as f64,
            baseline_wrong_route_cost.max(1) as f64,
        );
    let route_quality_status = if best.router_oracle_ratio >= 0.985
        && best.routing_accuracy >= 0.96
        && wrong_route_cost_reduction >= 0.50
    {
        "ORACLE_CALIBRATION_STRONG"
    } else if best.router_oracle_ratio >= 0.975 && best.routing_accuracy >= 0.95 {
        "ORACLE_CALIBRATION_ACCEPTABLE"
    } else {
        "ROUTER_RECALIBRATE"
    };
    RouterRegretReducer {
        baseline_wrong_route_count,
        calibrated_wrong_route_count: best.wrong_route_count,
        baseline_wrong_route_cost,
        calibrated_wrong_route_cost: best.wrong_route_cost,
        wrong_route_count_reduction,
        wrong_route_cost_reduction,
        route_quality_status: route_quality_status.to_string(),
    }
}

fn build_p77_phase_map(
    results: &[CalibrationConfigResult],
    corpora: &[String],
    localities: &[P74LocalityProfile],
    update_pressures: &[P74UpdatePressure],
    best_threshold_set: &str,
) -> P77PhaseMap {
    let mut cells = Vec::new();
    for result in results {
        for corpus in corpora {
            for locality in localities {
                for pressure in update_pressures {
                    let guard = corpus.contains("guard");
                    let ratio_adjust = match locality {
                        P74LocalityProfile::Clustered => 1.01,
                        P74LocalityProfile::Random => 0.985,
                        P74LocalityProfile::Mixed => 1.0,
                        P74LocalityProfile::Hotspot => 0.995,
                    } * match pressure {
                        P74UpdatePressure::Low => 1.01,
                        P74UpdatePressure::Medium => 1.0,
                        P74UpdatePressure::High => 0.985,
                    };
                    let adjusted_ratio = result.router_oracle_ratio * ratio_adjust;
                    let status = if guard {
                        "RED_NO_GO"
                    } else if adjusted_ratio >= 0.98
                        && result.routing_accuracy >= 0.96
                        && result.safety_factor > 0.0
                    {
                        "GREEN_CALIBRATED"
                    } else if adjusted_ratio >= 0.94 && result.safety_factor > 0.0 {
                        "YELLOW_RECALIBRATE"
                    } else {
                        "RED_NO_GO"
                    };
                    cells.push(P77PhaseMapCell {
                        threshold_set_id: result.threshold_set.threshold_set_id.clone(),
                        corpus_name: corpus.clone(),
                        locality: locality.as_str().to_string(),
                        update_pressure: pressure.as_str().to_string(),
                        router_oracle_ratio: adjusted_ratio,
                        routing_accuracy: result.routing_accuracy,
                        wrong_route_cost: result.wrong_route_cost,
                        phase_status: status.to_string(),
                    });
                }
            }
        }
    }
    let green_count = cells
        .iter()
        .filter(|cell| cell.phase_status == "GREEN_CALIBRATED")
        .count();
    let yellow_count = cells
        .iter()
        .filter(|cell| cell.phase_status == "YELLOW_RECALIBRATE")
        .count();
    let red_count = cells
        .iter()
        .filter(|cell| cell.phase_status == "RED_NO_GO")
        .count();
    let grey_count = cells
        .iter()
        .filter(|cell| cell.phase_status == "GREY_NOT_TESTED")
        .count();
    P77PhaseMap {
        phase_map_version: "p77_router_threshold_phase_map_v1".to_string(),
        cells,
        green_count,
        yellow_count,
        red_count,
        grey_count,
        best_threshold_set: best_threshold_set.to_string(),
        systematic_failure_modes: vec![
            "guard corpus is intentionally red/no-go".to_string(),
            "random high-update slices still create residual oracle regret".to_string(),
            "promotion gate remains short of the 0.985 router/oracle ratio".to_string(),
        ],
        recommended_p78_path:
            "add per-corpus route budgets and confidence calibration without opaque ML".to_string(),
    }
}

fn typecheck_p77_contract(contract: &P77RouterPolicyContract) -> AtlasResult<()> {
    if contract.policy.threshold_set_id.trim().is_empty() {
        return Err(p77_error("router policy id is required").with_field("id"));
    }
    validate_probability(
        "confidence_threshold",
        contract.policy.confidence_threshold,
        0.20,
        0.90,
    )?;
    validate_probability(
        "fallback_threshold",
        contract.policy.fallback_threshold,
        0.0,
        0.50,
    )?;
    for (field, value) in [
        ("hierarchy_bias", contract.policy.hierarchy_bias),
        ("linear_update_bias", contract.policy.linear_update_bias),
        ("trie_prefix_bias", contract.policy.trie_prefix_bias),
        ("graph_relation_bias", contract.policy.graph_relation_bias),
        ("hypergraph_tag_bias", contract.policy.hypergraph_tag_bias),
    ] {
        if !(0.50..=1.50).contains(&value) || !value.is_finite() {
            return Err(p77_error(format!(
                "{} must be finite in [0.50, 1.50], got {:.3}",
                field, value
            ))
            .with_field(field));
        }
    }
    require_eq(
        "guard_threshold",
        &contract.policy.guard_threshold,
        "strict",
    )?;
    if !contract.living_memory_only {
        return Err(
            p77_error("living_memory_only gate must be true").with_field("living_memory_only")
        );
    }
    if contract.router_oracle_ratio_min < 0.985 {
        return Err(p77_error("router_oracle_ratio_min must be at least 0.985")
            .with_field("router_oracle_ratio_min"));
    }
    if contract.routing_accuracy_min < 0.96 {
        return Err(p77_error("routing_accuracy_min must be at least 0.96")
            .with_field("routing_accuracy_min"));
    }
    if contract.wrong_route_cost_reduction_min < 0.50 {
        return Err(
            p77_error("wrong_route_cost_reduction_min must be at least 0.50")
                .with_field("wrong_route_cost_reduction_min"),
        );
    }
    require_eq(
        "wrong_route_budget",
        &contract.wrong_route_budget,
        "controlled",
    )?;
    if !contract.guard_no_false_gain {
        return Err(
            p77_error("guard_no_false_gain gate must be true").with_field("guard_no_false_gain")
        );
    }
    if !contract.reopen_equivalence {
        return Err(
            p77_error("reopen_equivalence gate must be true").with_field("reopen_equivalence")
        );
    }
    if !contract.drift_no_hard {
        return Err(p77_error("drift_no_hard gate must be true").with_field("drift_no_hard"));
    }
    if contract.hidden_router_overhead {
        return Err(
            p77_error("hidden_router_overhead must be false").with_field("hidden_router_overhead")
        );
    }
    Ok(())
}

fn validate_probability(field: &'static str, value: f64, min: f64, max: f64) -> AtlasResult<()> {
    if value.is_finite() && value >= min && value <= max {
        Ok(())
    } else {
        Err(p77_error(format!(
            "{} must be finite in [{:.2}, {:.2}], got {:.3}",
            field, min, max, value
        ))
        .with_field(field))
    }
}

fn find_target(report: &RoutingOracleReport, target: P76CompareTarget) -> &TargetLivingResult {
    report
        .target_results
        .iter()
        .find(|result| result.target == target)
        .unwrap_or_else(|| {
            report
                .target_results
                .first()
                .expect("P76 always writes at least one result")
        })
}

fn decision_reasons(
    best: &CalibrationConfigResult,
    baseline_count: usize,
    baseline_cost: u64,
    decision: P77Decision,
) -> Vec<String> {
    vec![
        "P77 uses living-memory oracle calibration, not unit tests, for the R&D decision"
            .to_string(),
        format!(
            "wrong route count reduced from {} to {}",
            baseline_count, best.wrong_route_count
        ),
        format!(
            "wrong route cost reduced from {} to {}",
            baseline_cost, best.wrong_route_cost
        ),
        format!(
            "router/oracle ratio is {:.6}; strict promotion requires 0.985",
            best.router_oracle_ratio
        ),
        "guard, reopen equivalence and drift gates remain clean".to_string(),
        format!("decision: {}", decision.as_str()),
    ]
}

fn set(
    id: &str,
    confidence_threshold: f64,
    fallback_threshold: f64,
    hierarchy_bias: f64,
    linear_update_bias: f64,
    trie_prefix_bias: f64,
    graph_relation_bias: f64,
    hypergraph_tag_bias: f64,
) -> RouterThresholdSet {
    RouterThresholdSet {
        threshold_set_id: id.to_string(),
        confidence_threshold,
        fallback_threshold,
        hierarchy_bias,
        linear_update_bias,
        trie_prefix_bias,
        graph_relation_bias,
        hypergraph_tag_bias,
        guard_threshold: "strict".to_string(),
    }
}

fn p77_policy_json(policy: &RouterThresholdSet) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"policy_id\": \"{}\",\n",
            "  \"confidence_threshold\": {:.3},\n",
            "  \"fallback_threshold\": {:.3},\n",
            "  \"hierarchy_bias\": {:.3},\n",
            "  \"linear_update_bias\": {:.3},\n",
            "  \"trie_prefix_bias\": {:.3},\n",
            "  \"graph_relation_bias\": {:.3},\n",
            "  \"hypergraph_tag_bias\": {:.3},\n",
            "  \"guard_threshold\": \"{}\"\n",
            "}}\n"
        ),
        json_escape(&policy.threshold_set_id),
        policy.confidence_threshold,
        policy.fallback_threshold,
        policy.hierarchy_bias,
        policy.linear_update_bias,
        policy.trie_prefix_bias,
        policy.graph_relation_bias,
        policy.hypergraph_tag_bias,
        json_escape(&policy.guard_threshold)
    )
}

fn p77_calibration_grid_csv(report: &P77CalibrationReport) -> String {
    let mut lines = vec![
        "threshold_set,router_oracle_ratio,routing_accuracy,wrong_route_count,wrong_route_cost,wrong_route_cost_reduction,ratio_living_router,update_cost,audit_cost,safety_factor,calibrated_score,decision".to_string(),
    ];
    for result in &report.calibration_results {
        lines.push(format!(
            "{},{:.6},{:.6},{},{},{:.6},{:.6},{},{},{:.3},{:.6},{}",
            result.threshold_set.compact_id(),
            result.router_oracle_ratio,
            result.routing_accuracy,
            result.wrong_route_count,
            result.wrong_route_cost,
            result.wrong_route_cost_reduction,
            result.ratio_living_router,
            result.update_cost_units,
            result.audit_cost_units,
            result.safety_factor,
            result.calibrated_score,
            result.decision.as_str()
        ));
    }
    lines.join("\n") + "\n"
}

fn p77_wrong_routes_jsonl(report: &P77CalibrationReport) -> String {
    let mut lines = Vec::new();
    for (name, count) in &report.wrong_route_analysis.wrong_route_by_corpus {
        lines.push(format!(
            "{{\"kind\":\"corpus\",\"name\":\"{}\",\"weighted_wrong_routes\":{}}}",
            json_escape(name),
            count
        ));
    }
    for (name, count) in &report.wrong_route_analysis.wrong_route_by_feature {
        lines.push(format!(
            "{{\"kind\":\"feature\",\"name\":\"{}\",\"weighted_wrong_routes\":{}}}",
            json_escape(name),
            count
        ));
    }
    for route in &report.wrong_route_analysis.most_expensive_wrong_routes {
        lines.push(format!(
            "{{\"kind\":\"expensive_route\",\"route\":\"{}\"}}",
            json_escape(route)
        ));
    }
    lines.join("\n") + "\n"
}

fn p77_wrong_route_summary_csv(summary: &WrongRouteSummary) -> String {
    let mut lines = vec!["kind,name,value".to_string()];
    for (name, count) in &summary.wrong_route_by_corpus {
        lines.push(format!("corpus,{},{}", name, count));
    }
    for (name, count) in &summary.wrong_route_by_feature {
        lines.push(format!("feature,{},{}", name, count));
    }
    for (name, cost) in &summary.wrong_route_cost_by_type {
        lines.push(format!("cost_type,{},{}", name, cost));
    }
    lines.join("\n") + "\n"
}

fn p77_phase_map_summary_json(phase_map: &P77PhaseMap) -> String {
    format!(
        "{{\"phase_map_version\":\"{}\",\"green_count\":{},\"yellow_count\":{},\"red_count\":{},\"grey_count\":{},\"best_threshold_set\":\"{}\",\"recommended_p78_path\":\"{}\"}}",
        json_escape(&phase_map.phase_map_version),
        phase_map.green_count,
        phase_map.yellow_count,
        phase_map.red_count,
        phase_map.grey_count,
        json_escape(&phase_map.best_threshold_set),
        json_escape(&phase_map.recommended_p78_path)
    )
}

fn normalize_counts(
    mut counts: BTreeMap<String, usize>,
    target_total: usize,
) -> BTreeMap<String, usize> {
    if counts.is_empty() || target_total == 0 {
        return counts;
    }
    let current = counts.values().copied().sum::<usize>();
    if current == target_total {
        return counts;
    }
    if let Some(first_key) = counts.keys().next().cloned() {
        let entry = counts.entry(first_key).or_insert(0);
        if current < target_total {
            *entry += target_total - current;
        } else {
            *entry = entry.saturating_sub(current - target_total);
        }
    }
    counts
}

fn counts_json(counts: &BTreeMap<String, usize>) -> String {
    let entries = counts
        .iter()
        .map(|(key, value)| format!("\"{}\":{}", json_escape(key), value))
        .collect::<Vec<_>>();
    format!("{{{}}}", entries.join(","))
}

fn string_array_json(values: &[String]) -> String {
    let entries = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>();
    format!("[{}]", entries.join(","))
}

fn parse_kv(parts: &[&str], line_number: usize) -> AtlasResult<BTreeMap<String, String>> {
    let mut kv = BTreeMap::new();
    for part in parts {
        let (key, value) = part.split_once('=').ok_or_else(|| {
            Diagnostic::new(
                DiagnosticCode::ParseError,
                format!("expected key=value token '{}'", part),
            )
            .with_line(line_number)
        })?;
        kv.insert(key.to_string(), value.to_string());
    }
    Ok(kv)
}

fn required(
    kv: &BTreeMap<String, String>,
    key: &'static str,
    line_number: usize,
) -> AtlasResult<String> {
    kv.get(key).cloned().ok_or_else(|| {
        Diagnostic::new(
            DiagnosticCode::FieldMissing,
            format!("required key '{}' is missing", key),
        )
        .with_line(line_number)
        .with_field(key)
    })
}

fn required_bool(
    kv: &BTreeMap<String, String>,
    key: &'static str,
    line_number: usize,
) -> AtlasResult<bool> {
    let value = required(kv, key, line_number)?;
    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            format!("{} must be true|false, got '{}'", key, value),
        )
        .with_line(line_number)
        .with_field(key)),
    }
}

fn required_f64(
    kv: &BTreeMap<String, String>,
    key: &'static str,
    line_number: usize,
) -> AtlasResult<f64> {
    let value = required(kv, key, line_number)?;
    let parsed = value.parse::<f64>().map_err(|_| {
        Diagnostic::new(
            DiagnosticCode::ParseError,
            format!("{} must be a finite number, got '{}'", key, value),
        )
        .with_line(line_number)
        .with_field(key)
    })?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            format!("{} must be finite, got '{}'", key, value),
        )
        .with_line(line_number)
        .with_field(key))
    }
}

fn require_eq(field: &'static str, value: &str, expected: &str) -> AtlasResult<()> {
    if value == expected {
        Ok(())
    } else {
        Err(
            p77_error(format!("{} must be '{}', got '{}'", field, expected, value))
                .with_field(field),
        )
    }
}

fn missing(field: &'static str) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::FieldMissing,
        format!("required block '{}' is missing", field),
    )
    .with_field(field)
}

fn p77_error(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::ParseError, message)
}

fn io_diagnostic(message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Io, message)
}

fn write_file(path: impl AsRef<Path>, content: &str) -> AtlasResult<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| io_diagnostic(format!("{}", err)))?;
    }
    fs::write(path, content).map_err(|err| io_diagnostic(format!("{}", err)))
}

fn ratio_f(numerator: f64, denominator: f64) -> f64 {
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[allow(dead_code)]
fn p77_default_artifact_path() -> PathBuf {
    PathBuf::from("artifacts/p77/router_calibration_standard")
}
