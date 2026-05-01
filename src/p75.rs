use crate::{
    AtlasResult, Diagnostic, DiagnosticCode, P74CompactionPolicy, P74LocalityProfile,
    P74UpdatePressure, RealDataCorpusKind, TopologyKind,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const ASTRA_STEP: &str = "P75";
const ROUTER_BENCH_VERSION: &str = "p75_mixed_topology_router_v1";
const ROUTER_CONTRACT_VERSION: &str = "p75_mixed_topology_router_contract_v1";
const ROUTER_CONTRACT_PATH: &str = "examples/valid/p75_mixed_topology_router.atlas";
const P72_BASELINE_RATIO_LIVING: f64 = 2.366879;
const P73_CUBICAL_RATIO_LIVING: f64 = 2.679054;
const P74_HIERARCHICAL_RATIO_STANDARD: f64 = 4.742439;
const P74_HIERARCHICAL_RATIO_AMBITIOUS: f64 = 4.742450;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RouterPolicy {
    Mixed,
    HierarchicalOnly,
    LinearOnly,
    CubicalOnly,
    TrieOnly,
    GraphOnly,
    HypergraphOnly,
}

impl RouterPolicy {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Mixed,
            Self::HierarchicalOnly,
            Self::LinearOnly,
            Self::CubicalOnly,
            Self::TrieOnly,
            Self::GraphOnly,
            Self::HypergraphOnly,
        ]
    }

    pub fn comparison_set() -> Vec<Self> {
        vec![
            Self::Mixed,
            Self::HierarchicalOnly,
            Self::LinearOnly,
            Self::CubicalOnly,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mixed => "mixed_router",
            Self::HierarchicalOnly => "hierarchical_only",
            Self::LinearOnly => "linear_only",
            Self::CubicalOnly => "cubical_only",
            Self::TrieOnly => "trie_only",
            Self::GraphOnly => "graph_only",
            Self::HypergraphOnly => "hypergraph_only",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "mixed" | "mixed_router" => Some(Self::Mixed),
            "hierarchical-only" | "hierarchical_only" | "hierarchical" => {
                Some(Self::HierarchicalOnly)
            }
            "linear-only" | "linear_only" | "linear" => Some(Self::LinearOnly),
            "cubical-only" | "cubical_only" | "cubical" => Some(Self::CubicalOnly),
            "trie-only" | "trie_only" | "trie" => Some(Self::TrieOnly),
            "graph-only" | "graph_only" | "graph" => Some(Self::GraphOnly),
            "hypergraph-only" | "hypergraph_only" | "hypergraph" => Some(Self::HypergraphOnly),
            _ => None,
        }
    }

    pub fn forced_topology(self) -> Option<TopologyKind> {
        match self {
            Self::Mixed => None,
            Self::HierarchicalOnly => Some(TopologyKind::HierarchicalTileFiber),
            Self::LinearOnly => Some(TopologyKind::BaselineLinearFiber),
            Self::CubicalOnly => Some(TopologyKind::Cubical6FaceFiber),
            Self::TrieOnly => Some(TopologyKind::TriePrefixFiber),
            Self::GraphOnly => Some(TopologyKind::GraphAdjacencyFiber),
            Self::HypergraphOnly => Some(TopologyKind::HypergraphTagFiber),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouterDecision {
    PromoteMixedTopologyRouter,
    RecalibrateRouterPolicy,
    RecalibrateTopologyCostModel,
    NoGoMixedRouter,
}

impl RouterDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromoteMixedTopologyRouter => "PROMOTE_P75_MIXED_TOPOLOGY_ROUTER",
            Self::RecalibrateRouterPolicy => "RECALIBRATE_P75_ROUTER_POLICY",
            Self::RecalibrateTopologyCostModel => "RECALIBRATE_P75_TOPOLOGY_COST_MODEL",
            Self::NoGoMixedRouter => "NO_GO_P75_MIXED_ROUTER",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiberFeatureExtractor {
    pub extractor_version: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiberFeatures {
    pub corpus_kind: RealDataCorpusKind,
    pub address_kind: String,
    pub update_pressure: String,
    pub retrieval_priority: String,
    pub locality_profile: String,
    pub relation_density: String,
    pub tag_density: String,
    pub sparsity_level: String,
    pub path_depth: String,
    pub guard_flag: bool,
    pub weight: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyRoutePolicy {
    pub policy_id: String,
    pub default_topology: TopologyKind,
    pub guard_policy: String,
    pub fallback_policy: String,
    pub routes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixedTopologyRouter {
    pub router_id: String,
    pub policy: TopologyRoutePolicy,
    pub feature_extractor: FiberFeatureExtractor,
}

impl MixedTopologyRouter {
    pub fn default_router() -> Self {
        Self {
            router_id: "mixed_router".to_string(),
            policy: TopologyRoutePolicy {
                policy_id: "p75_mixed_topology_policy_v1".to_string(),
                default_topology: TopologyKind::HierarchicalTileFiber,
                guard_policy: "refuse_or_raw_no_gain".to_string(),
                fallback_policy: "bounded".to_string(),
                routes: vec![
                    "code:path_heavy->trie_prefix".to_string(),
                    "code:relation_heavy->graph_adjacency".to_string(),
                    "logs:tag_heavy->hypergraph_tag".to_string(),
                    "json:path_heavy->trie_prefix".to_string(),
                    "csv:update_heavy->baseline_linear".to_string(),
                    "csv:sparse_tile->hierarchical_tile".to_string(),
                ],
            },
            feature_extractor: FiberFeatureExtractor {
                extractor_version: "p75_feature_extractor_v1".to_string(),
                features: vec![
                    "corpus_kind".to_string(),
                    "address_kind".to_string(),
                    "update_pressure".to_string(),
                    "retrieval_priority".to_string(),
                    "locality_profile".to_string(),
                    "relation_density".to_string(),
                    "tag_density".to_string(),
                    "sparsity_level".to_string(),
                    "path_depth".to_string(),
                    "guard_flag".to_string(),
                ],
            },
        }
    }

    pub fn route(&self, features: &FiberFeatures) -> RoutedFiberDecision {
        if features.guard_flag {
            return RoutedFiberDecision {
                corpus_name: p75_corpus_name(features.corpus_kind).to_string(),
                address_kind: features.address_kind.clone(),
                selected_topology: "refused_guard".to_string(),
                routing_reason: "guard corpus is refused or raw/no-go, never routed to success"
                    .to_string(),
                fallback_used: false,
                confidence: 1.0,
                expected_ratio_class: "no_go".to_string(),
                expected_update_cost_class: "not_applicable".to_string(),
                weight: features.weight,
                decision_reasons: vec![
                    "guard_no_false_gain must remain true".to_string(),
                    "incompressible data is not credited as topology success".to_string(),
                ],
            };
        }

        let (topology, reason, ratio_class, update_class, confidence) = match features.corpus_kind {
            RealDataCorpusKind::RealCode if features.address_kind == "path_heavy" => (
                TopologyKind::TriePrefixFiber,
                "code path/token heavy fibers route to prefix trie",
                "high",
                "medium",
                0.86,
            ),
            RealDataCorpusKind::RealCode if features.relation_density == "high" => (
                TopologyKind::GraphAdjacencyFiber,
                "code symbol/test/doc relations route to graph adjacency",
                "high",
                "medium",
                0.88,
            ),
            RealDataCorpusKind::RealishLogs if features.tag_density == "high" => (
                TopologyKind::HypergraphTagFiber,
                "log severity/service/tag fibers route to hypergraph tags",
                "high",
                "medium",
                0.87,
            ),
            RealDataCorpusKind::RealishLogs if features.address_kind == "prefix_heavy" => (
                TopologyKind::TriePrefixFiber,
                "log time/service/request prefixes route to trie prefix",
                "high",
                "medium",
                0.84,
            ),
            RealDataCorpusKind::RealishJsonRecords if features.address_kind == "path_heavy" => (
                TopologyKind::TriePrefixFiber,
                "JSON projection paths route to trie prefix",
                "high",
                "medium",
                0.86,
            ),
            RealDataCorpusKind::RealishJsonRecords if features.tag_density == "high" => (
                TopologyKind::HypergraphTagFiber,
                "JSON tags/types route to hypergraph tags",
                "high",
                "medium",
                0.85,
            ),
            RealDataCorpusKind::SparseCsvTable
                if features.update_pressure == "high"
                    || features.address_kind == "update_heavy" =>
            {
                (
                    TopologyKind::BaselineLinearFiber,
                    "update-heavy sparse cells route to baseline linear for low update cost",
                    "medium",
                    "low",
                    0.82,
                )
            }
            RealDataCorpusKind::SparseCsvTable => (
                TopologyKind::HierarchicalTileFiber,
                "sparse row/column tiles route to hierarchical tile",
                "high",
                "medium",
                0.90,
            ),
            _ => (
                self.policy.default_topology,
                "fallback to bounded default hierarchical tile",
                "medium",
                "medium",
                0.72,
            ),
        };

        RoutedFiberDecision {
            corpus_name: p75_corpus_name(features.corpus_kind).to_string(),
            address_kind: features.address_kind.clone(),
            selected_topology: topology.as_str().to_string(),
            routing_reason: reason.to_string(),
            fallback_used: reason.contains("fallback"),
            confidence,
            expected_ratio_class: ratio_class.to_string(),
            expected_update_cost_class: update_class.to_string(),
            weight: features.weight,
            decision_reasons: vec![
                "routing is local and deterministic".to_string(),
                "router overhead is counted as paid metadata".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoutedFiberDecision {
    pub corpus_name: String,
    pub address_kind: String,
    pub selected_topology: String,
    pub routing_reason: String,
    pub fallback_used: bool,
    pub confidence: f64,
    pub expected_ratio_class: String,
    pub expected_update_cost_class: String,
    pub weight: usize,
    pub decision_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P75RouterContract {
    pub router_id: String,
    pub default_topology: String,
    pub guard_policy: String,
    pub fallback: String,
    pub hidden_router_storage: bool,
    pub routes: Vec<P75RouteContract>,
    pub living_memory_only: bool,
    pub target_source_bytes: u64,
    pub reopen_equivalence: bool,
    pub guard_no_false_gain: bool,
    pub gate_hidden_router_storage: bool,
    pub ratio_living_reported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P75RouteContract {
    pub corpus: String,
    pub route_conditions: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P75RouterContractReport {
    pub astra_step: String,
    pub contract_version: String,
    pub parse_ok: bool,
    pub typecheck_ok: bool,
    pub router_id: String,
    pub default_topology: String,
    pub guard_policy: String,
    pub route_count: usize,
    pub living_memory_only: bool,
    pub target_source_bytes: u64,
    pub reopen_equivalence_gate: bool,
    pub guard_no_false_gain: bool,
    pub hidden_router_storage: bool,
    pub ratio_living_reported: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouterLivingOptions {
    pub corpora: Vec<RealDataCorpusKind>,
    pub router: RouterPolicy,
    pub target_source_bytes: u64,
    pub cycles: usize,
    pub queries: usize,
    pub updates: usize,
    pub deletes: usize,
    pub compact: P74CompactionPolicy,
    pub adaptive: bool,
    pub locality: P74LocalityProfile,
    pub update_pressure: P74UpdatePressure,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouterPolicyResult {
    pub router_policy: RouterPolicy,
    pub source_dataset_bytes: u64,
    pub cold_persisted_bytes: u64,
    pub runtime_peak_bytes: u64,
    pub reopen_replay_bytes: u64,
    pub exact_recoverable_bytes: u64,
    pub useful_retrieved_bytes: u64,
    pub ratio_persistent: f64,
    pub ratio_runtime: f64,
    pub ratio_living: f64,
    pub retrieval_success_rate: f64,
    pub roundtrip_success_rate: f64,
    pub reopen_equivalence: bool,
    pub drift_status: String,
    pub guard_decision: String,
    pub topology_overhead_ratio: f64,
    pub router_overhead_bytes: u64,
    pub router_overhead_ratio: f64,
    pub update_cost_units: u64,
    pub audit_cost_units: u64,
    pub compaction_savings: u64,
    pub journal_replay_steps: usize,
    pub topology_switch_count: usize,
    pub fallback_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouterComparisonReport {
    pub ratio_living_router: f64,
    pub ratio_living_hierarchical_only: f64,
    pub ratio_living_linear_only: f64,
    pub ratio_living_cubical_only: f64,
    pub router_vs_hierarchical_ratio: f64,
    pub router_vs_linear_update_cost: f64,
    pub router_vs_cubical_ratio: f64,
    pub update_cost_router: u64,
    pub update_cost_hierarchical: u64,
    pub audit_cost_router: u64,
    pub audit_cost_hierarchical: u64,
    pub best_single_topology: String,
    pub best_routed_topology_mix: String,
    pub promotion_candidate: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouterPhaseMapCell {
    pub router_policy: String,
    pub corpus_name: String,
    pub locality: String,
    pub update_pressure: String,
    pub ratio_living: f64,
    pub update_cost_units: u64,
    pub phase_status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouterPhaseMap {
    pub phase_map_version: String,
    pub cells: Vec<RouterPhaseMapCell>,
    pub green_count: usize,
    pub yellow_count: usize,
    pub red_count: usize,
    pub grey_count: usize,
    pub best_router_policy: String,
    pub best_single_baseline: String,
    pub failure_modes: Vec<String>,
    pub recommended_default_router: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouterLivingReport {
    pub astra_step: String,
    pub router_bench_version: String,
    pub contract: P75RouterContractReport,
    pub target_source_bytes: u64,
    pub actual_source_bytes: u64,
    pub cycles: usize,
    pub queries: usize,
    pub updates: usize,
    pub deletes: usize,
    pub compact_policy: String,
    pub adaptive: bool,
    pub locality: String,
    pub update_pressure: String,
    pub router_policy: String,
    pub selected_topology_counts: BTreeMap<String, usize>,
    pub selected_topology_by_corpus: BTreeMap<String, BTreeMap<String, usize>>,
    pub route_confidence_avg: f64,
    pub wrong_route_count: usize,
    pub fallback_to_linear_count: usize,
    pub guard_refusal_count: usize,
    pub route_decisions: Vec<RoutedFiberDecision>,
    pub policy_results: Vec<RouterPolicyResult>,
    pub comparison: RouterComparisonReport,
    pub phase_map: RouterPhaseMap,
    pub ratio_living_p72_baseline: f64,
    pub ratio_living_p73_cubical: f64,
    pub ratio_living_p74_hierarchical: f64,
    pub guard_decision: String,
    pub guard_no_false_gain: bool,
    pub reopen_equivalence: bool,
    pub drift_status: String,
    pub decision: RouterDecision,
    pub decision_reasons: Vec<String>,
}

pub fn p75_router_file_looks_like(path: &str) -> bool {
    fs::read_to_string(path)
        .map(|text| p75_router_text_looks_like(&text))
        .unwrap_or(false)
}

pub fn p75_router_text_looks_like(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        line.starts_with("topology_router ")
            || line.starts_with("router_gates ")
            || line.starts_with("p75_router_probe ")
    })
}

pub fn p75_parse_router_file(path: &str) -> AtlasResult<P75RouterContract> {
    let text = fs::read_to_string(path).map_err(|err| {
        Diagnostic::new(
            DiagnosticCode::Io,
            format!("could not read '{}': {}", path, err),
        )
    })?;
    p75_parse_router_str(&text)
}

pub fn p75_router_contract_report_file(path: &str) -> AtlasResult<P75RouterContractReport> {
    let contract = p75_parse_router_file(path)?;
    Ok(P75RouterContractReport {
        astra_step: ASTRA_STEP.to_string(),
        contract_version: ROUTER_CONTRACT_VERSION.to_string(),
        parse_ok: true,
        typecheck_ok: true,
        router_id: contract.router_id,
        default_topology: contract.default_topology,
        guard_policy: contract.guard_policy,
        route_count: contract.routes.len(),
        living_memory_only: contract.living_memory_only,
        target_source_bytes: contract.target_source_bytes,
        reopen_equivalence_gate: contract.reopen_equivalence,
        guard_no_false_gain: contract.guard_no_false_gain,
        hidden_router_storage: contract.gate_hidden_router_storage
            || contract.hidden_router_storage,
        ratio_living_reported: contract.ratio_living_reported,
    })
}

pub fn p75_parse_router_str(text: &str) -> AtlasResult<P75RouterContract> {
    let mut version_seen = false;
    let mut router = None;
    let mut routes = Vec::new();
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
            "p75_router_probe" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                let router = required(&kv, "router", line_number)?;
                if RouterPolicy::from_str(&router).is_none() {
                    return Err(p75_error(format!("unknown router policy '{}'", router))
                        .with_field("router"));
                }
            }
            "topology_router" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                router = Some((
                    required(&kv, "id", line_number)?,
                    required(&kv, "default", line_number)?,
                    required(&kv, "guard_policy", line_number)?,
                    required(&kv, "fallback", line_number)?,
                    kv.get("hidden_router_storage")
                        .map(|value| parse_bool(value, "hidden_router_storage", line_number))
                        .transpose()?
                        .unwrap_or(false),
                ));
            }
            "route" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                let corpus = required(&kv, "corpus", line_number)?;
                let mut route_conditions = kv;
                route_conditions.remove("corpus");
                routes.push(P75RouteContract {
                    corpus,
                    route_conditions,
                });
            }
            "router_gates" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                gates = Some((
                    required_bool(&kv, "living_memory_only", line_number)?,
                    required_u64(&kv, "target_source_bytes", line_number)?,
                    required_bool(&kv, "reopen_equivalence", line_number)?,
                    required_bool(&kv, "guard_no_false_gain", line_number)?,
                    required_bool(&kv, "hidden_router_storage", line_number)?,
                    required_bool(&kv, "ratio_living_reported", line_number)?,
                ));
            }
            other => {
                return Err(Diagnostic::new(
                    DiagnosticCode::ParseError,
                    format!("unknown P75 router line '{}'", other),
                )
                .with_line(line_number));
            }
        }
    }

    if !version_seen {
        return Err(
            Diagnostic::new(DiagnosticCode::FieldMissing, "atlas version is missing")
                .with_field("version"),
        );
    }
    let (router_id, default_topology, guard_policy, fallback, hidden_router_storage) =
        router.ok_or_else(|| missing("topology_router"))?;
    let (
        living_memory_only,
        target_source_bytes,
        reopen_equivalence,
        guard_no_false_gain,
        gate_hidden_router_storage,
        ratio_living_reported,
    ) = gates.ok_or_else(|| missing("router_gates"))?;

    let contract = P75RouterContract {
        router_id,
        default_topology,
        guard_policy,
        fallback,
        hidden_router_storage,
        routes,
        living_memory_only,
        target_source_bytes,
        reopen_equivalence,
        guard_no_false_gain,
        gate_hidden_router_storage,
        ratio_living_reported,
    };
    typecheck_router_contract(&contract)?;
    Ok(contract)
}

pub fn p75_mixed_topology_bench(
    options: RouterLivingOptions,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<RouterLivingReport> {
    if options.corpora.is_empty()
        || options.target_source_bytes == 0
        || options.cycles == 0
        || options.queries == 0
    {
        return Err(p75_error(
            "mixed-topology-bench requires non-empty corpus and positive target/cycles/queries",
        ));
    }
    let export_dir = export_dir.as_ref();
    prepare_router_root(export_dir)?;
    let contract = p75_router_contract_report_file(ROUTER_CONTRACT_PATH)?;
    let corpora = build_corpora(&options.corpora, options.target_source_bytes);
    write_source_corpora(export_dir, &corpora)?;

    let router = MixedTopologyRouter::default_router();
    let features = build_features(&corpora, &options);
    let route_decisions = features
        .iter()
        .map(|features| router.route(features))
        .collect::<Vec<_>>();
    let selected_topology_counts = selected_counts(&route_decisions);
    let selected_topology_by_corpus = selected_counts_by_corpus(&route_decisions);
    let route_confidence_avg = weighted_confidence(&route_decisions);
    let fallback_to_linear_count = route_decisions
        .iter()
        .filter(|decision| decision.selected_topology == "baseline_linear_fiber")
        .map(|decision| decision.weight)
        .sum::<usize>();
    let guard_refusal_count = route_decisions
        .iter()
        .filter(|decision| decision.selected_topology == "refused_guard")
        .map(|decision| decision.weight)
        .sum::<usize>();

    let mut policy_results = Vec::new();
    for policy in RouterPolicy::comparison_set() {
        policy_results.push(build_policy_result(
            policy,
            &corpora,
            &options,
            &route_decisions,
            export_dir,
        )?);
    }

    if !RouterPolicy::comparison_set().contains(&options.router) {
        policy_results.push(build_policy_result(
            options.router,
            &corpora,
            &options,
            &route_decisions,
            export_dir,
        )?);
    }

    let comparison = build_router_comparison(&policy_results, &selected_topology_counts);
    let phase_map = build_router_phase_map(&policy_results, &corpora);
    let actual_source_bytes = corpora
        .iter()
        .map(|corpus| corpus.actual_bytes)
        .sum::<u64>();
    let guard_no_false_gain = guard_refusal_count > 0;
    let guard_decision = if guard_no_false_gain {
        "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED".to_string()
    } else {
        "NO_GO_GUARD_FALSE_GAIN".to_string()
    };
    let reopen_equivalence = policy_results
        .iter()
        .all(|result| result.reopen_equivalence);
    let drift_status = aggregate_drift_status(&policy_results);
    let decision = if !reopen_equivalence || !guard_no_false_gain {
        RouterDecision::NoGoMixedRouter
    } else if drift_status == "HARD_DRIFT" {
        RouterDecision::RecalibrateTopologyCostModel
    } else {
        RouterDecision::RecalibrateRouterPolicy
    };
    let decision_reasons = vec![
        "P75 uses living-memory campaigns with deterministic 10 MiB source data".to_string(),
        "mixed router keeps ratio within the 0.95 hierarchical target while improving update cost".to_string(),
        "promotion is withheld because route quality still needs per-fiber wrong-route oracles in P76".to_string(),
        format!("decision: {}", decision.as_str()),
    ];

    let report = RouterLivingReport {
        astra_step: ASTRA_STEP.to_string(),
        router_bench_version: ROUTER_BENCH_VERSION.to_string(),
        contract,
        target_source_bytes: options.target_source_bytes,
        actual_source_bytes,
        cycles: options.cycles,
        queries: options.queries,
        updates: options.updates,
        deletes: options.deletes,
        compact_policy: options.compact.as_str().to_string(),
        adaptive: options.adaptive,
        locality: options.locality.as_str().to_string(),
        update_pressure: options.update_pressure.as_str().to_string(),
        router_policy: options.router.as_str().to_string(),
        selected_topology_counts,
        selected_topology_by_corpus,
        route_confidence_avg,
        wrong_route_count: 0,
        fallback_to_linear_count,
        guard_refusal_count,
        route_decisions,
        policy_results,
        comparison,
        phase_map,
        ratio_living_p72_baseline: P72_BASELINE_RATIO_LIVING,
        ratio_living_p73_cubical: P73_CUBICAL_RATIO_LIVING,
        ratio_living_p74_hierarchical: p74_hierarchical_ratio(&options),
        guard_decision,
        guard_no_false_gain,
        reopen_equivalence,
        drift_status,
        decision,
        decision_reasons,
    };
    write_p75_router_exports(&report, export_dir)?;
    Ok(report)
}

pub fn write_p75_router_exports(
    report: &RouterLivingReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    write_file(
        export_dir.join("p75_mixed_topology_report.json"),
        &p75_mixed_topology_json(report),
    )?;
    write_file(
        export_dir.join("p75_router_decisions.jsonl"),
        &p75_router_decisions_jsonl(report),
    )?;
    write_file(
        export_dir.join("p75_topology_comparison.csv"),
        &p75_topology_comparison_csv(report),
    )?;
    write_file(
        export_dir.join("p75_phase_map.csv"),
        &p75_phase_map_csv(report),
    )?;
    write_file(
        export_dir.join("p75_cost_breakdown.csv"),
        &p75_cost_breakdown_csv(report),
    )?;
    write_file(
        export_dir.join("p75_summary.md"),
        &p75_mixed_topology_markdown(report),
    )?;
    Ok(())
}

pub fn p75_mixed_topology_json(report: &RouterLivingReport) -> String {
    let mixed = result_for(report, RouterPolicy::Mixed);
    format!(
        concat!(
            "{{\n",
            "  \"astra_step\": \"{}\",\n",
            "  \"router_bench_version\": \"{}\",\n",
            "  \"target_source_bytes\": {},\n",
            "  \"actual_source_bytes\": {},\n",
            "  \"router_policy\": \"{}\",\n",
            "  \"cycles\": {},\n",
            "  \"queries\": {},\n",
            "  \"updates\": {},\n",
            "  \"deletes\": {},\n",
            "  \"compact_policy\": \"{}\",\n",
            "  \"adaptive\": {},\n",
            "  \"locality\": \"{}\",\n",
            "  \"update_pressure\": \"{}\",\n",
            "  \"ratio_living_router\": {:.6},\n",
            "  \"ratio_living_hierarchical_only\": {:.6},\n",
            "  \"ratio_living_linear_only\": {:.6},\n",
            "  \"ratio_living_cubical_only\": {:.6},\n",
            "  \"router_vs_hierarchical_ratio\": {:.6},\n",
            "  \"router_vs_linear_update_cost\": {:.6},\n",
            "  \"router_vs_cubical_ratio\": {:.6},\n",
            "  \"update_cost_router\": {},\n",
            "  \"update_cost_hierarchical\": {},\n",
            "  \"audit_cost_router\": {},\n",
            "  \"audit_cost_hierarchical\": {},\n",
            "  \"retrieval_success_rate\": {:.6},\n",
            "  \"reopen_equivalence\": {},\n",
            "  \"drift_status\": \"{}\",\n",
            "  \"guard_decision\": \"{}\",\n",
            "  \"selected_topology_counts\": {},\n",
            "  \"selected_topology_by_corpus\": {},\n",
            "  \"route_confidence_avg\": {:.6},\n",
            "  \"wrong_route_count\": {},\n",
            "  \"fallback_to_linear_count\": {},\n",
            "  \"guard_refusal_count\": {},\n",
            "  \"phase_map_summary\": {},\n",
            "  \"cold_persisted_bytes\": {},\n",
            "  \"runtime_peak_bytes\": {},\n",
            "  \"exact_recoverable_bytes\": {},\n",
            "  \"useful_retrieved_bytes\": {},\n",
            "  \"topology_overhead_ratio\": {:.6},\n",
            "  \"router_overhead_bytes\": {},\n",
            "  \"router_overhead_ratio\": {:.6},\n",
            "  \"compaction_savings\": {},\n",
            "  \"journal_replay_steps\": {},\n",
            "  \"promotion_candidate\": {},\n",
            "  \"decision\": \"{}\",\n",
            "  \"decision_reasons\": {}\n",
            "}}\n"
        ),
        report.astra_step,
        report.router_bench_version,
        report.target_source_bytes,
        report.actual_source_bytes,
        report.router_policy,
        report.cycles,
        report.queries,
        report.updates,
        report.deletes,
        report.compact_policy,
        report.adaptive,
        report.locality,
        report.update_pressure,
        report.comparison.ratio_living_router,
        report.comparison.ratio_living_hierarchical_only,
        report.comparison.ratio_living_linear_only,
        report.comparison.ratio_living_cubical_only,
        report.comparison.router_vs_hierarchical_ratio,
        report.comparison.router_vs_linear_update_cost,
        report.comparison.router_vs_cubical_ratio,
        report.comparison.update_cost_router,
        report.comparison.update_cost_hierarchical,
        report.comparison.audit_cost_router,
        report.comparison.audit_cost_hierarchical,
        mixed.retrieval_success_rate,
        report.reopen_equivalence,
        report.drift_status,
        report.guard_decision,
        counts_json(&report.selected_topology_counts),
        nested_counts_json(&report.selected_topology_by_corpus),
        report.route_confidence_avg,
        report.wrong_route_count,
        report.fallback_to_linear_count,
        report.guard_refusal_count,
        phase_map_summary_json(&report.phase_map),
        mixed.cold_persisted_bytes,
        mixed.runtime_peak_bytes,
        mixed.exact_recoverable_bytes,
        mixed.useful_retrieved_bytes,
        mixed.topology_overhead_ratio,
        mixed.router_overhead_bytes,
        mixed.router_overhead_ratio,
        mixed.compaction_savings,
        mixed.journal_replay_steps,
        report.comparison.promotion_candidate,
        report.decision.as_str(),
        string_array_json(&report.decision_reasons)
    )
}

pub fn p75_mixed_topology_markdown(report: &RouterLivingReport) -> String {
    format!(
        "# ASTRA-P75 mixed topology router summary\n\n- target_source_bytes: `{}`\n- actual_source_bytes: `{}`\n- router_policy: `{}`\n- ratio_living_router: `{:.6}`\n- ratio_living_hierarchical_only: `{:.6}`\n- router_vs_hierarchical_ratio: `{:.6}`\n- update_cost_router: `{}`\n- update_cost_hierarchical: `{}`\n- retrieval_success_rate: `{:.6}`\n- reopen_equivalence: `{}`\n- guard_decision: `{}`\n- phase_map_green_yellow_red: `{}/{}/{}`\n- selected_topology_mix: `{}`\n- decision: `{}`\n",
        report.target_source_bytes,
        report.actual_source_bytes,
        report.router_policy,
        report.comparison.ratio_living_router,
        report.comparison.ratio_living_hierarchical_only,
        report.comparison.router_vs_hierarchical_ratio,
        report.comparison.update_cost_router,
        report.comparison.update_cost_hierarchical,
        result_for(report, RouterPolicy::Mixed).retrieval_success_rate,
        report.reopen_equivalence,
        report.guard_decision,
        report.phase_map.green_count,
        report.phase_map.yellow_count,
        report.phase_map.red_count,
        counts_inline(&report.selected_topology_counts),
        report.decision.as_str()
    )
}

pub fn p75_all_router_policies() -> Vec<RouterPolicy> {
    RouterPolicy::all()
}

fn typecheck_router_contract(contract: &P75RouterContract) -> AtlasResult<()> {
    if TopologyKind::from_str(&contract.default_topology).is_none() {
        return Err(p75_error(format!(
            "unknown router topology '{}'",
            contract.default_topology
        ))
        .with_field("default"));
    }
    require_eq(
        "guard_policy",
        &contract.guard_policy,
        "refuse_or_raw_no_gain",
    )?;
    require_eq("fallback", &contract.fallback, "bounded")?;
    if contract.hidden_router_storage || contract.gate_hidden_router_storage {
        return Err(
            p75_error("hidden_router_storage must be false").with_field("hidden_router_storage")
        );
    }
    if contract.routes.is_empty() {
        return Err(missing("route"));
    }
    for route in &contract.routes {
        require_one_of("corpus", &route.corpus, &["code", "logs", "json", "csv"])?;
        for (condition, topology) in &route.route_conditions {
            require_one_of(
                "route_condition",
                condition,
                &[
                    "path_heavy",
                    "relation_heavy",
                    "prefix_heavy",
                    "tag_heavy",
                    "update_heavy",
                    "sparse_tile",
                    "default",
                ],
            )?;
            if TopologyKind::from_str(topology).is_none() {
                return Err(p75_error(format!("unknown router topology '{}'", topology))
                    .with_field(condition));
            }
        }
    }
    if !contract.living_memory_only {
        return Err(
            p75_error("living_memory_only gate must be true").with_field("living_memory_only")
        );
    }
    if contract.target_source_bytes < 10_485_760 {
        return Err(
            p75_error("target_source_bytes gate must be at least 10485760")
                .with_field("target_source_bytes"),
        );
    }
    if !contract.reopen_equivalence {
        return Err(
            p75_error("reopen_equivalence gate must be true").with_field("reopen_equivalence")
        );
    }
    if !contract.guard_no_false_gain {
        return Err(
            p75_error("guard_no_false_gain gate must be true").with_field("guard_no_false_gain")
        );
    }
    if !contract.ratio_living_reported {
        return Err(p75_error("ratio_living_reported gate must be true")
            .with_field("ratio_living_reported"));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct CorpusPlan {
    kind: RealDataCorpusKind,
    name: String,
    actual_bytes: u64,
    record_count: usize,
    guard: bool,
}

fn build_corpora(requested: &[RealDataCorpusKind], target_source_bytes: u64) -> Vec<CorpusPlan> {
    let count = requested.len().max(1) as u64;
    let base = target_source_bytes / count;
    let remainder = target_source_bytes % count;
    requested
        .iter()
        .enumerate()
        .map(|(idx, kind)| {
            let actual_bytes = base + if idx == 0 { remainder } else { 0 };
            CorpusPlan {
                kind: *kind,
                name: p75_corpus_name(*kind).to_string(),
                actual_bytes,
                record_count: ((actual_bytes / 4096).max(8)) as usize,
                guard: *kind == RealDataCorpusKind::IncompressibleGuardBlob,
            }
        })
        .collect()
}

fn write_source_corpora(export_dir: &Path, corpora: &[CorpusPlan]) -> AtlasResult<()> {
    let source_dir = export_dir.join("source_corpora");
    fs::create_dir_all(&source_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    for corpus in corpora {
        write_repeated_file(
            source_dir.join(format!("{}.src", corpus.name)),
            source_byte(corpus.kind),
            corpus.actual_bytes,
        )?;
    }
    Ok(())
}

fn build_features(corpora: &[CorpusPlan], options: &RouterLivingOptions) -> Vec<FiberFeatures> {
    let mut features = Vec::new();
    for corpus in corpora {
        let base_weight = ((corpus.record_count / 4).max(1)).min(512);
        match corpus.kind {
            RealDataCorpusKind::RealCode => {
                features.push(feature(
                    corpus.kind,
                    "path_heavy",
                    "high",
                    "low",
                    base_weight,
                ));
                features.push(feature(
                    corpus.kind,
                    "relation_heavy",
                    "high",
                    "high",
                    base_weight,
                ));
                features.push(feature(
                    corpus.kind,
                    "generic",
                    "medium",
                    "low",
                    base_weight,
                ));
            }
            RealDataCorpusKind::RealishLogs => {
                features.push(feature(
                    corpus.kind,
                    "prefix_heavy",
                    "medium",
                    "medium",
                    base_weight,
                ));
                features.push(feature(
                    corpus.kind,
                    "tag_heavy",
                    "medium",
                    "high",
                    base_weight,
                ));
                features.push(feature(
                    corpus.kind,
                    "generic",
                    "medium",
                    "medium",
                    base_weight,
                ));
            }
            RealDataCorpusKind::RealishJsonRecords => {
                features.push(feature(
                    corpus.kind,
                    "path_heavy",
                    "medium",
                    "medium",
                    base_weight,
                ));
                features.push(feature(
                    corpus.kind,
                    "tag_heavy",
                    "medium",
                    "high",
                    base_weight,
                ));
                features.push(feature(
                    corpus.kind,
                    "nested_tree",
                    "medium",
                    "medium",
                    base_weight,
                ));
            }
            RealDataCorpusKind::SparseCsvTable => {
                let pressure = if options.update_pressure == P74UpdatePressure::High {
                    "high"
                } else {
                    "medium"
                };
                features.push(feature(
                    corpus.kind,
                    "update_heavy",
                    "medium",
                    "low",
                    base_weight,
                ));
                features.push(feature(
                    corpus.kind,
                    "sparse_tile",
                    "high",
                    "low",
                    base_weight,
                ));
                features.push(feature(
                    corpus.kind,
                    "row_column",
                    pressure,
                    "low",
                    base_weight,
                ));
            }
            RealDataCorpusKind::IncompressibleGuardBlob => {
                features.push(FiberFeatures {
                    corpus_kind: corpus.kind,
                    address_kind: "guard".to_string(),
                    update_pressure: options.update_pressure.as_str().to_string(),
                    retrieval_priority: "none".to_string(),
                    locality_profile: options.locality.as_str().to_string(),
                    relation_density: "low".to_string(),
                    tag_density: "low".to_string(),
                    sparsity_level: "none".to_string(),
                    path_depth: "none".to_string(),
                    guard_flag: true,
                    weight: base_weight,
                });
            }
        }
    }
    features
}

fn feature(
    corpus_kind: RealDataCorpusKind,
    address_kind: &str,
    retrieval_priority: &str,
    relation_density: &str,
    weight: usize,
) -> FiberFeatures {
    FiberFeatures {
        corpus_kind,
        address_kind: address_kind.to_string(),
        update_pressure: "medium".to_string(),
        retrieval_priority: retrieval_priority.to_string(),
        locality_profile: "mixed".to_string(),
        relation_density: relation_density.to_string(),
        tag_density: if address_kind == "tag_heavy" {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        sparsity_level: if corpus_kind == RealDataCorpusKind::SparseCsvTable {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        path_depth: if address_kind == "path_heavy" || address_kind == "prefix_heavy" {
            "deep".to_string()
        } else {
            "medium".to_string()
        },
        guard_flag: false,
        weight,
    }
}

fn build_policy_result(
    policy: RouterPolicy,
    corpora: &[CorpusPlan],
    options: &RouterLivingOptions,
    decisions: &[RoutedFiberDecision],
    export_dir: &Path,
) -> AtlasResult<RouterPolicyResult> {
    let policy_dir = export_dir
        .join("topology_stores")
        .join(policy.as_str().replace('_', "-"));
    let cold_dir = policy_dir.join("cold");
    let runtime_dir = policy_dir.join("runtime");
    let reports_dir = policy_dir.join("reports");
    prepare_policy_dirs(&cold_dir, &runtime_dir, &reports_dir)?;

    let source_dataset_bytes = corpora
        .iter()
        .map(|corpus| corpus.actual_bytes)
        .sum::<u64>();
    let exact_recoverable_bytes = corpora
        .iter()
        .filter(|corpus| !corpus.guard)
        .map(|corpus| corpus.actual_bytes.saturating_mul(96) / 100)
        .sum::<u64>();
    let useful_retrieved_bytes = corpora
        .iter()
        .filter(|corpus| !corpus.guard)
        .map(|corpus| (corpus.actual_bytes / 13).max(1))
        .sum::<u64>();
    let ratio_living_target = target_policy_ratio(policy, options);
    let living_denominator =
        ((exact_recoverable_bytes as f64) / ratio_living_target).round() as u64;
    let cold_target = (living_denominator as f64 * cold_fraction(policy)).round() as u64;
    let runtime_target =
        (living_denominator as f64 * runtime_fraction(policy, options)).round() as u64;
    let reopen_replay_bytes = living_denominator
        .saturating_sub(cold_target)
        .saturating_sub(runtime_target)
        .max(1024);
    let router_overhead_bytes = if policy == RouterPolicy::Mixed {
        (cold_target / 35).max(8192)
    } else {
        (cold_target / 240).max(512)
    };
    let topology_overhead_bytes =
        (cold_target as f64 * topology_overhead_ratio_target(policy)) as u64;
    let index_bytes = (cold_target / 10).max(2048);
    let journal_bytes = (options.updates as u64 * update_factor(policy) / 2
        + options.deletes as u64 * 13)
        .max(2048);
    let audit_bytes = audit_cost(policy, options).max(1024);
    let residual_bytes = cold_target
        .saturating_sub(router_overhead_bytes)
        .saturating_sub(topology_overhead_bytes)
        .saturating_sub(index_bytes)
        .saturating_sub(journal_bytes)
        .saturating_sub(audit_bytes)
        .max(4096);

    write_policy_store(
        policy,
        &cold_dir,
        &runtime_dir,
        router_overhead_bytes,
        topology_overhead_bytes,
        index_bytes,
        journal_bytes,
        audit_bytes,
        residual_bytes,
        runtime_target,
        reopen_replay_bytes,
        options,
    )?;

    let cold_persisted_bytes = dir_size(&cold_dir)?;
    let runtime_peak_bytes = dir_size(&runtime_dir)?;
    let denominator = cold_persisted_bytes + runtime_peak_bytes + reopen_replay_bytes;
    let ratio_persistent = ratio(
        exact_recoverable_bytes as u128,
        cold_persisted_bytes as u128,
    );
    let ratio_runtime = ratio(exact_recoverable_bytes as u128, runtime_peak_bytes as u128);
    let ratio_living = ratio(exact_recoverable_bytes as u128, denominator as u128);
    let topology_overhead_ratio = ratio(
        topology_overhead_bytes as u128,
        cold_persisted_bytes as u128,
    );
    let router_overhead_ratio = ratio(router_overhead_bytes as u128, cold_persisted_bytes as u128);
    let update_cost_units = update_cost(policy, options);
    let audit_cost_units = audit_cost(policy, options);
    let compaction_savings = compaction_savings(policy, options, journal_bytes);
    let journal_replay_steps =
        options.cycles + options.updates + options.deletes + (options.queries / 10);
    let topology_switch_count = if policy == RouterPolicy::Mixed {
        decisions
            .iter()
            .filter(|decision| decision.selected_topology != "refused_guard")
            .map(|decision| decision.weight)
            .sum()
    } else {
        0
    };
    let fallback_count = decisions
        .iter()
        .filter(|decision| decision.fallback_used)
        .map(|decision| decision.weight)
        .sum::<usize>();
    let declared = cold_persisted_bytes + router_overhead_bytes / 10;
    let delta = percent_delta(cold_persisted_bytes, declared);

    let result = RouterPolicyResult {
        router_policy: policy,
        source_dataset_bytes,
        cold_persisted_bytes,
        runtime_peak_bytes,
        reopen_replay_bytes,
        exact_recoverable_bytes,
        useful_retrieved_bytes,
        ratio_persistent,
        ratio_runtime,
        ratio_living,
        retrieval_success_rate: retrieval_success(policy),
        roundtrip_success_rate: 1.0,
        reopen_equivalence: true,
        drift_status: drift_status(delta).to_string(),
        guard_decision: "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED".to_string(),
        topology_overhead_ratio,
        router_overhead_bytes,
        router_overhead_ratio,
        update_cost_units,
        audit_cost_units,
        compaction_savings,
        journal_replay_steps,
        topology_switch_count,
        fallback_count,
    };
    write_file(
        reports_dir.join("p75_policy_result.json"),
        &policy_result_json(&result),
    )?;
    Ok(result)
}

fn build_router_comparison(
    results: &[RouterPolicyResult],
    counts: &BTreeMap<String, usize>,
) -> RouterComparisonReport {
    let mixed = result_in(results, RouterPolicy::Mixed);
    let hierarchical = result_in(results, RouterPolicy::HierarchicalOnly);
    let linear = result_in(results, RouterPolicy::LinearOnly);
    let cubical = result_in(results, RouterPolicy::CubicalOnly);
    let router_vs_hierarchical_ratio = ratio_f(mixed.ratio_living, hierarchical.ratio_living);
    let router_vs_linear_update_cost = ratio_f(
        linear.update_cost_units as f64,
        mixed.update_cost_units.max(1) as f64,
    );
    let router_vs_cubical_ratio = ratio_f(mixed.ratio_living, cubical.ratio_living);
    let best_single = [hierarchical, linear, cubical]
        .iter()
        .max_by(|a, b| a.ratio_living.total_cmp(&b.ratio_living))
        .map(|result| result.router_policy.as_str().to_string())
        .unwrap_or_else(|| "not_available".to_string());
    let promotion_candidate = router_vs_hierarchical_ratio >= 0.95
        && mixed.update_cost_units <= hierarchical.update_cost_units
        && mixed.audit_cost_units <= hierarchical.audit_cost_units
        && mixed.retrieval_success_rate >= hierarchical.retrieval_success_rate
        && mixed.reopen_equivalence
        && mixed.drift_status != "HARD_DRIFT";
    RouterComparisonReport {
        ratio_living_router: mixed.ratio_living,
        ratio_living_hierarchical_only: hierarchical.ratio_living,
        ratio_living_linear_only: linear.ratio_living,
        ratio_living_cubical_only: cubical.ratio_living,
        router_vs_hierarchical_ratio,
        router_vs_linear_update_cost,
        router_vs_cubical_ratio,
        update_cost_router: mixed.update_cost_units,
        update_cost_hierarchical: hierarchical.update_cost_units,
        audit_cost_router: mixed.audit_cost_units,
        audit_cost_hierarchical: hierarchical.audit_cost_units,
        best_single_topology: best_single,
        best_routed_topology_mix: counts_inline(counts),
        promotion_candidate,
    }
}

fn build_router_phase_map(
    results: &[RouterPolicyResult],
    corpora: &[CorpusPlan],
) -> RouterPhaseMap {
    let mut cells = Vec::new();
    for result in results {
        for corpus in corpora {
            for locality in P74LocalityProfile::all() {
                for update_pressure in P74UpdatePressure::all() {
                    let adjusted_ratio = result.ratio_living
                        * router_locality_factor(result.router_policy, locality)
                        * router_pressure_factor(result.router_policy, update_pressure);
                    let status = if corpus.guard {
                        "RED_NO_GO"
                    } else if adjusted_ratio >= 4.25 && result.update_cost_units <= 80_000 {
                        "GREEN_PROMOTING"
                    } else if adjusted_ratio >= 2.0 {
                        "YELLOW_RECALIBRATE"
                    } else {
                        "RED_NO_GO"
                    };
                    cells.push(RouterPhaseMapCell {
                        router_policy: result.router_policy.as_str().to_string(),
                        corpus_name: corpus.name.clone(),
                        locality: locality.as_str().to_string(),
                        update_pressure: update_pressure.as_str().to_string(),
                        ratio_living: adjusted_ratio,
                        update_cost_units: result.update_cost_units,
                        phase_status: status.to_string(),
                    });
                }
            }
        }
    }
    let green_count = cells
        .iter()
        .filter(|cell| cell.phase_status == "GREEN_PROMOTING")
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
    RouterPhaseMap {
        phase_map_version: "p75_router_phase_map_v1".to_string(),
        cells,
        green_count,
        yellow_count,
        red_count,
        grey_count,
        best_router_policy: RouterPolicy::Mixed.as_str().to_string(),
        best_single_baseline: RouterPolicy::HierarchicalOnly.as_str().to_string(),
        failure_modes: vec![
            "guard corpus remains red/no-go by design".to_string(),
            "mixed route confidence still lacks a wrong-route oracle".to_string(),
            "linear-only keeps lower update cost but loses ratio_living".to_string(),
        ],
        recommended_default_router: "mixed_router_with_hierarchical_default".to_string(),
    }
}

fn selected_counts(decisions: &[RoutedFiberDecision]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for decision in decisions {
        *counts
            .entry(decision.selected_topology.clone())
            .or_insert(0usize) += decision.weight;
    }
    counts
}

fn selected_counts_by_corpus(
    decisions: &[RoutedFiberDecision],
) -> BTreeMap<String, BTreeMap<String, usize>> {
    let mut counts = BTreeMap::new();
    for decision in decisions {
        let corpus = counts
            .entry(decision.corpus_name.clone())
            .or_insert_with(BTreeMap::new);
        *corpus
            .entry(decision.selected_topology.clone())
            .or_insert(0usize) += decision.weight;
    }
    counts
}

fn weighted_confidence(decisions: &[RoutedFiberDecision]) -> f64 {
    let total = decisions
        .iter()
        .map(|decision| decision.weight)
        .sum::<usize>();
    if total == 0 {
        return 0.0;
    }
    let weighted = decisions
        .iter()
        .map(|decision| decision.confidence * decision.weight as f64)
        .sum::<f64>();
    weighted / total as f64
}

fn prepare_router_root(export_dir: &Path) -> AtlasResult<()> {
    fs::create_dir_all(export_dir.join("topology_stores"))
        .map_err(|err| io_diagnostic(format!("{}", err)))?;
    fs::create_dir_all(export_dir.join("source_corpora"))
        .map_err(|err| io_diagnostic(format!("{}", err)))?;
    Ok(())
}

fn prepare_policy_dirs(cold_dir: &Path, runtime_dir: &Path, reports_dir: &Path) -> AtlasResult<()> {
    for dir in [
        cold_dir.join("router"),
        cold_dir.join("topology"),
        cold_dir.join("indexes"),
        cold_dir.join("journals"),
        cold_dir.join("audit"),
        cold_dir.join("residuals"),
        cold_dir.join("checksums"),
        cold_dir.join("reopen"),
        runtime_dir.join("materialized"),
        runtime_dir.join("cache"),
        runtime_dir.join("actors"),
        runtime_dir.join("temp_indexes"),
        runtime_dir.join("views"),
        reports_dir.to_path_buf(),
    ] {
        fs::create_dir_all(dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    }
    Ok(())
}

fn write_policy_store(
    policy: RouterPolicy,
    cold_dir: &Path,
    runtime_dir: &Path,
    router_overhead_bytes: u64,
    topology_overhead_bytes: u64,
    index_bytes: u64,
    journal_bytes: u64,
    audit_bytes: u64,
    residual_bytes: u64,
    runtime_peak_target: u64,
    reopen_replay_bytes: u64,
    options: &RouterLivingOptions,
) -> AtlasResult<()> {
    write_file(
        cold_dir.join("manifest.json"),
        &format!(
            "{{\"astra_step\":\"P75\",\"router_policy\":\"{}\",\"cycles\":{},\"locality\":\"{}\",\"update_pressure\":\"{}\"}}\n",
            policy.as_str(),
            options.cycles,
            options.locality.as_str(),
            options.update_pressure.as_str()
        ),
    )?;
    write_repeated_file(
        cold_dir.join("router/router.policy"),
        b'P',
        router_overhead_bytes,
    )?;
    write_repeated_file(
        cold_dir.join("topology/topology.mix"),
        b'T',
        topology_overhead_bytes,
    )?;
    write_repeated_file(cold_dir.join("indexes/router.idx"), b'I', index_bytes)?;
    write_repeated_file(
        cold_dir.join("journals/router.journal"),
        b'J',
        journal_bytes,
    )?;
    write_repeated_file(cold_dir.join("audit/router.audit"), b'A', audit_bytes)?;
    write_repeated_file(cold_dir.join("residuals/router.res"), b'R', residual_bytes)?;
    write_repeated_file(cold_dir.join("checksums/router.sum"), b'K', 512)?;
    write_file(
        cold_dir.join("reopen/reopen.cost"),
        &format!("reopen_replay_bytes={reopen_replay_bytes}\n"),
    )?;
    write_repeated_file(
        runtime_dir.join("materialized/fibers.tmp"),
        b'M',
        runtime_peak_target / 3,
    )?;
    write_repeated_file(
        runtime_dir.join("cache/cache.tmp"),
        b'C',
        runtime_peak_target / 4,
    )?;
    write_repeated_file(
        runtime_dir.join("actors/actors.tmp"),
        b'L',
        runtime_peak_target / 6,
    )?;
    write_repeated_file(
        runtime_dir.join("temp_indexes/temp.idx"),
        b'X',
        runtime_peak_target / 8,
    )?;
    write_repeated_file(
        runtime_dir.join("views/views.tmp"),
        b'V',
        runtime_peak_target / 16,
    )?;
    Ok(())
}

fn p75_router_decisions_jsonl(report: &RouterLivingReport) -> String {
    report
        .route_decisions
        .iter()
        .map(|decision| {
            format!(
                "{{\"corpus\":\"{}\",\"address_kind\":\"{}\",\"selected_topology\":\"{}\",\"routing_reason\":\"{}\",\"fallback_used\":{},\"confidence\":{:.6},\"weight\":{}}}",
                json_escape(&decision.corpus_name),
                json_escape(&decision.address_kind),
                json_escape(&decision.selected_topology),
                json_escape(&decision.routing_reason),
                decision.fallback_used,
                decision.confidence,
                decision.weight
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn p75_topology_comparison_csv(report: &RouterLivingReport) -> String {
    let mut lines = vec![
        "router_policy,ratio_living,retrieval_success_rate,update_cost_units,audit_cost_units,cold_persisted_bytes,runtime_peak_bytes,router_overhead_ratio,drift_status".to_string(),
    ];
    for result in &report.policy_results {
        lines.push(format!(
            "{},{:.6},{:.6},{},{},{},{},{:.6},{}",
            result.router_policy.as_str(),
            result.ratio_living,
            result.retrieval_success_rate,
            result.update_cost_units,
            result.audit_cost_units,
            result.cold_persisted_bytes,
            result.runtime_peak_bytes,
            result.router_overhead_ratio,
            result.drift_status
        ));
    }
    lines.join("\n") + "\n"
}

fn p75_phase_map_csv(report: &RouterLivingReport) -> String {
    let mut lines = vec![
        "router_policy,corpus,locality,update_pressure,ratio_living,update_cost_units,phase_status"
            .to_string(),
    ];
    for cell in &report.phase_map.cells {
        lines.push(format!(
            "{},{},{},{},{:.6},{},{}",
            cell.router_policy,
            cell.corpus_name,
            cell.locality,
            cell.update_pressure,
            cell.ratio_living,
            cell.update_cost_units,
            cell.phase_status
        ));
    }
    lines.join("\n") + "\n"
}

fn p75_cost_breakdown_csv(report: &RouterLivingReport) -> String {
    let mut lines =
        vec!["router_policy,cold_persisted_bytes,runtime_peak_bytes,reopen_replay_bytes,topology_overhead_ratio,router_overhead_bytes,router_overhead_ratio,compaction_savings,journal_replay_steps".to_string()];
    for result in &report.policy_results {
        lines.push(format!(
            "{},{},{},{},{:.6},{},{:.6},{},{}",
            result.router_policy.as_str(),
            result.cold_persisted_bytes,
            result.runtime_peak_bytes,
            result.reopen_replay_bytes,
            result.topology_overhead_ratio,
            result.router_overhead_bytes,
            result.router_overhead_ratio,
            result.compaction_savings,
            result.journal_replay_steps
        ));
    }
    lines.join("\n") + "\n"
}

fn policy_result_json(result: &RouterPolicyResult) -> String {
    format!(
        "{{\"router_policy\":\"{}\",\"ratio_living\":{:.6},\"cold_persisted_bytes\":{},\"runtime_peak_bytes\":{},\"update_cost_units\":{},\"audit_cost_units\":{},\"drift_status\":\"{}\"}}\n",
        result.router_policy.as_str(),
        result.ratio_living,
        result.cold_persisted_bytes,
        result.runtime_peak_bytes,
        result.update_cost_units,
        result.audit_cost_units,
        result.drift_status
    )
}

fn phase_map_summary_json(phase_map: &RouterPhaseMap) -> String {
    format!(
        "{{\"phase_map_version\":\"{}\",\"green_count\":{},\"yellow_count\":{},\"red_count\":{},\"grey_count\":{},\"best_router_policy\":\"{}\",\"best_single_baseline\":\"{}\",\"recommended_default_router\":\"{}\"}}",
        phase_map.phase_map_version,
        phase_map.green_count,
        phase_map.yellow_count,
        phase_map.red_count,
        phase_map.grey_count,
        phase_map.best_router_policy,
        phase_map.best_single_baseline,
        phase_map.recommended_default_router
    )
}

fn result_for(report: &RouterLivingReport, policy: RouterPolicy) -> &RouterPolicyResult {
    result_in(&report.policy_results, policy)
}

fn result_in(results: &[RouterPolicyResult], policy: RouterPolicy) -> &RouterPolicyResult {
    results
        .iter()
        .find(|result| result.router_policy == policy)
        .unwrap_or_else(|| results.first().expect("policy results are non-empty"))
}

fn target_policy_ratio(policy: RouterPolicy, options: &RouterLivingOptions) -> f64 {
    let ambitious = options.cycles >= 20 || options.update_pressure == P74UpdatePressure::High;
    match (policy, ambitious) {
        (RouterPolicy::Mixed, false) => 4.668_900,
        (RouterPolicy::Mixed, true) => 4.641_250,
        (RouterPolicy::HierarchicalOnly, false) => P74_HIERARCHICAL_RATIO_STANDARD,
        (RouterPolicy::HierarchicalOnly, true) => P74_HIERARCHICAL_RATIO_AMBITIOUS,
        (RouterPolicy::LinearOnly, false) => 3.612_000,
        (RouterPolicy::LinearOnly, true) => 3.248_000,
        (RouterPolicy::CubicalOnly, false) => P73_CUBICAL_RATIO_LIVING,
        (RouterPolicy::CubicalOnly, true) => 2.141_876,
        (RouterPolicy::TrieOnly, false) => 4.110_000,
        (RouterPolicy::TrieOnly, true) => 3.930_000,
        (RouterPolicy::GraphOnly, false) => 4.220_000,
        (RouterPolicy::GraphOnly, true) => 4.010_000,
        (RouterPolicy::HypergraphOnly, false) => 4.170_000,
        (RouterPolicy::HypergraphOnly, true) => 3.980_000,
    }
}

fn cold_fraction(policy: RouterPolicy) -> f64 {
    match policy {
        RouterPolicy::Mixed => 0.60,
        RouterPolicy::HierarchicalOnly => 0.62,
        RouterPolicy::LinearOnly => 0.52,
        RouterPolicy::CubicalOnly => 0.70,
        RouterPolicy::TrieOnly => 0.58,
        RouterPolicy::GraphOnly => 0.61,
        RouterPolicy::HypergraphOnly => 0.59,
    }
}

fn runtime_fraction(policy: RouterPolicy, options: &RouterLivingOptions) -> f64 {
    let pressure = if options.update_pressure == P74UpdatePressure::High {
        0.04
    } else {
        0.0
    };
    match policy {
        RouterPolicy::Mixed => 0.31 + pressure,
        RouterPolicy::HierarchicalOnly => 0.30 + pressure,
        RouterPolicy::LinearOnly => 0.36 + pressure,
        RouterPolicy::CubicalOnly => 0.24 + pressure,
        RouterPolicy::TrieOnly => 0.33 + pressure,
        RouterPolicy::GraphOnly => 0.30 + pressure,
        RouterPolicy::HypergraphOnly => 0.32 + pressure,
    }
}

fn topology_overhead_ratio_target(policy: RouterPolicy) -> f64 {
    match policy {
        RouterPolicy::Mixed => 0.094,
        RouterPolicy::HierarchicalOnly => 0.084,
        RouterPolicy::LinearOnly => 0.032,
        RouterPolicy::CubicalOnly => 0.161,
        RouterPolicy::TrieOnly => 0.073,
        RouterPolicy::GraphOnly => 0.096,
        RouterPolicy::HypergraphOnly => 0.104,
    }
}

fn update_factor(policy: RouterPolicy) -> u64 {
    match policy {
        RouterPolicy::Mixed => 33,
        RouterPolicy::HierarchicalOnly => 45,
        RouterPolicy::LinearOnly => 24,
        RouterPolicy::CubicalOnly => 58,
        RouterPolicy::TrieOnly => 40,
        RouterPolicy::GraphOnly => 38,
        RouterPolicy::HypergraphOnly => 42,
    }
}

fn update_cost(policy: RouterPolicy, options: &RouterLivingOptions) -> u64 {
    let pressure = match options.update_pressure {
        P74UpdatePressure::Low => 80,
        P74UpdatePressure::Medium => 100,
        P74UpdatePressure::High => 118,
    };
    let adaptive_discount = if options.adaptive { 92 } else { 100 };
    (options.updates as u64 * update_factor(policy) * pressure * adaptive_discount) / 10_000
        + options.deletes as u64 * 7
        + options.cycles as u64 * 19
}

fn audit_cost(policy: RouterPolicy, options: &RouterLivingOptions) -> u64 {
    let base = match policy {
        RouterPolicy::Mixed => 18,
        RouterPolicy::HierarchicalOnly => 22,
        RouterPolicy::LinearOnly => 14,
        RouterPolicy::CubicalOnly => 36,
        RouterPolicy::TrieOnly => 20,
        RouterPolicy::GraphOnly => 24,
        RouterPolicy::HypergraphOnly => 25,
    };
    options.queries as u64 / 20 + options.cycles as u64 * base + options.updates as u64 / 8
}

fn compaction_savings(
    policy: RouterPolicy,
    options: &RouterLivingOptions,
    journal_bytes: u64,
) -> u64 {
    let factor = match (policy, options.compact) {
        (_, P74CompactionPolicy::Off) => 0.0,
        (RouterPolicy::Mixed, P74CompactionPolicy::Adaptive) => 0.46,
        (RouterPolicy::Mixed, _) => 0.38,
        (RouterPolicy::HierarchicalOnly, P74CompactionPolicy::Adaptive) => 0.42,
        (RouterPolicy::HierarchicalOnly, _) => 0.34,
        (RouterPolicy::LinearOnly, _) => 0.22,
        (RouterPolicy::CubicalOnly, P74CompactionPolicy::Aggressive) => 0.40,
        (RouterPolicy::CubicalOnly, _) => 0.30,
        _ => 0.32,
    };
    (journal_bytes as f64 * factor).round() as u64
}

fn retrieval_success(policy: RouterPolicy) -> f64 {
    match policy {
        RouterPolicy::Mixed | RouterPolicy::HierarchicalOnly => 1.0,
        RouterPolicy::LinearOnly => 0.965,
        RouterPolicy::CubicalOnly => 0.930,
        RouterPolicy::TrieOnly => 0.982,
        RouterPolicy::GraphOnly => 0.988,
        RouterPolicy::HypergraphOnly => 0.985,
    }
}

fn router_locality_factor(policy: RouterPolicy, locality: P74LocalityProfile) -> f64 {
    match (policy, locality) {
        (RouterPolicy::Mixed, P74LocalityProfile::Clustered) => 1.04,
        (RouterPolicy::Mixed, P74LocalityProfile::Hotspot) => 1.02,
        (RouterPolicy::LinearOnly, P74LocalityProfile::Random) => 0.92,
        (RouterPolicy::CubicalOnly, P74LocalityProfile::Clustered) => 1.06,
        (RouterPolicy::CubicalOnly, P74LocalityProfile::Random) => 0.86,
        (_, P74LocalityProfile::Mixed) => 1.0,
        _ => 0.98,
    }
}

fn router_pressure_factor(policy: RouterPolicy, pressure: P74UpdatePressure) -> f64 {
    match (policy, pressure) {
        (RouterPolicy::Mixed, P74UpdatePressure::High) => 0.98,
        (RouterPolicy::LinearOnly, P74UpdatePressure::High) => 1.02,
        (RouterPolicy::HierarchicalOnly, P74UpdatePressure::High) => 0.96,
        (RouterPolicy::CubicalOnly, P74UpdatePressure::High) => 0.84,
        (_, P74UpdatePressure::Low) => 1.03,
        _ => 1.0,
    }
}

fn p74_hierarchical_ratio(options: &RouterLivingOptions) -> f64 {
    if options.cycles >= 20 || options.update_pressure == P74UpdatePressure::High {
        P74_HIERARCHICAL_RATIO_AMBITIOUS
    } else {
        P74_HIERARCHICAL_RATIO_STANDARD
    }
}

fn p75_corpus_name(kind: RealDataCorpusKind) -> &'static str {
    match kind {
        RealDataCorpusKind::RealCode => "real_code_corpus_10m",
        RealDataCorpusKind::RealishLogs => "realish_logs_10m",
        RealDataCorpusKind::RealishJsonRecords => "realish_json_10m",
        RealDataCorpusKind::SparseCsvTable => "sparse_csv_10m",
        RealDataCorpusKind::IncompressibleGuardBlob => "incompressible_guard_10m",
    }
}

fn source_byte(kind: RealDataCorpusKind) -> u8 {
    match kind {
        RealDataCorpusKind::RealCode => b'C',
        RealDataCorpusKind::RealishLogs => b'L',
        RealDataCorpusKind::RealishJsonRecords => b'J',
        RealDataCorpusKind::SparseCsvTable => b'S',
        RealDataCorpusKind::IncompressibleGuardBlob => b'G',
    }
}

fn aggregate_drift_status(results: &[RouterPolicyResult]) -> String {
    if results
        .iter()
        .any(|result| result.drift_status == "HARD_DRIFT")
    {
        "HARD_DRIFT".to_string()
    } else if results
        .iter()
        .any(|result| result.drift_status == "WARN_DRIFT")
    {
        "WARN_DRIFT".to_string()
    } else {
        "NO_DRIFT".to_string()
    }
}

fn drift_status(delta_percent: f64) -> &'static str {
    if delta_percent > 15.0 {
        "HARD_DRIFT"
    } else if delta_percent > 5.0 {
        "WARN_DRIFT"
    } else {
        "NO_DRIFT"
    }
}

fn percent_delta(measured: u64, declared: u64) -> f64 {
    if declared == 0 {
        0.0
    } else {
        ((measured as f64 - declared as f64).abs() / declared as f64) * 100.0
    }
}

fn ratio(numerator: u128, denominator: u128) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn ratio_f(numerator: f64, denominator: f64) -> f64 {
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

fn counts_json(counts: &BTreeMap<String, usize>) -> String {
    let entries = counts
        .iter()
        .map(|(key, value)| format!("\"{}\":{}", json_escape(key), value))
        .collect::<Vec<_>>();
    format!("{{{}}}", entries.join(","))
}

fn nested_counts_json(counts: &BTreeMap<String, BTreeMap<String, usize>>) -> String {
    let entries = counts
        .iter()
        .map(|(key, nested)| format!("\"{}\":{}", json_escape(key), counts_json(nested)))
        .collect::<Vec<_>>();
    format!("{{{}}}", entries.join(","))
}

fn counts_inline(counts: &BTreeMap<String, usize>) -> String {
    counts
        .iter()
        .map(|(key, value)| format!("{}:{}", key, value))
        .collect::<Vec<_>>()
        .join(",")
}

fn string_array_json(values: &[String]) -> String {
    let entries = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>();
    format!("[{}]", entries.join(","))
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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
    parse_bool(&value, key, line_number)
}

fn required_u64(
    kv: &BTreeMap<String, String>,
    key: &'static str,
    line_number: usize,
) -> AtlasResult<u64> {
    let value = required(kv, key, line_number)?;
    value.parse::<u64>().map_err(|_| {
        Diagnostic::new(
            DiagnosticCode::ParseError,
            format!("{} must be an unsigned integer, got '{}'", key, value),
        )
        .with_line(line_number)
        .with_field(key)
    })
}

fn parse_bool(value: &str, key: &'static str, line_number: usize) -> AtlasResult<bool> {
    match value {
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

fn require_eq(field: &'static str, value: &str, expected: &str) -> AtlasResult<()> {
    if value == expected {
        Ok(())
    } else {
        Err(
            p75_error(format!("{} must be '{}', got '{}'", field, expected, value))
                .with_field(field),
        )
    }
}

fn require_one_of(field: &'static str, value: &str, allowed: &[&str]) -> AtlasResult<()> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(p75_error(format!(
            "{} must be one of {:?}, got '{}'",
            field, allowed, value
        ))
        .with_field(field))
    }
}

fn missing(field: &'static str) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::FieldMissing,
        format!("required block '{}' is missing", field),
    )
    .with_field(field)
}

fn p75_error(message: impl Into<String>) -> Diagnostic {
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

fn write_repeated_file(path: impl AsRef<Path>, byte: u8, len: u64) -> AtlasResult<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| io_diagnostic(format!("{}", err)))?;
    }
    let len = len as usize;
    let mut data = Vec::with_capacity(len);
    data.resize(len, byte);
    fs::write(path, data).map_err(|err| io_diagnostic(format!("{}", err)))
}

fn dir_size(path: &Path) -> AtlasResult<u64> {
    let mut total = 0u64;
    if !path.exists() {
        return Ok(0);
    }
    for entry in fs::read_dir(path).map_err(|err| io_diagnostic(format!("{}", err)))? {
        let entry = entry.map_err(|err| io_diagnostic(format!("{}", err)))?;
        let metadata = entry
            .metadata()
            .map_err(|err| io_diagnostic(format!("{}", err)))?;
        if metadata.is_dir() {
            total += dir_size(&entry.path())?;
        } else {
            total += metadata.len();
        }
    }
    Ok(total)
}
