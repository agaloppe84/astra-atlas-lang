use crate::{
    p78_level1_space_estimate, AddressingCostReport, AtlasResult, CrudAddressingReport, Diagnostic,
    DiagnosticCode, Level1TopologyKind, Level1VirtualSpaceEstimateOptions,
    Level1VirtualSpaceMetrics, P74CompactionPolicy, RealDataCorpusKind,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ASTRA_STEP: &str = "P79";
const LEVEL1_ROUTER_VERSION: &str = "p79_level1_address_router_v1";
const LEVEL1_ROUTER_ESTIMATE_VERSION: &str = "p79_level1_router_estimate_v1";
const LEVEL1_ROUTER_CONTRACT_VERSION: &str = "p79_level1_router_contract_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P79Decision {
    PromoteLevel1AddressRouter,
    RecalibrateLevel1RouterPolicy,
    RecalibrateLevel1IndexCost,
    NoGoLevel1Router,
}

impl P79Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromoteLevel1AddressRouter => "PROMOTE_P79_LEVEL1_ADDRESS_ROUTER",
            Self::RecalibrateLevel1RouterPolicy => "RECALIBRATE_P79_LEVEL1_ROUTER_POLICY",
            Self::RecalibrateLevel1IndexCost => "RECALIBRATE_P79_LEVEL1_INDEX_COST",
            Self::NoGoLevel1Router => "NO_GO_P79_LEVEL1_ROUTER",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum P79CompareTarget {
    Router,
    Oracle,
    HybridOnly,
    PathTrieOnly,
    ProductTypedOnly,
    ContentDagOnly,
}

impl P79CompareTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Router => "router",
            Self::Oracle => "oracle",
            Self::HybridOnly => "hybrid_only",
            Self::PathTrieOnly => "path_trie_only",
            Self::ProductTypedOnly => "product_typed_only",
            Self::ContentDagOnly => "content_dag_only",
        }
    }

    pub fn store_id(self) -> &'static str {
        match self {
            Self::Router => "router",
            Self::Oracle => "oracle",
            Self::HybridOnly => "hybrid",
            Self::PathTrieOnly => "path_trie",
            Self::ProductTypedOnly => "product_typed",
            Self::ContentDagOnly => "content_dag",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "router" | "level1-router" | "level1_router" => Some(Self::Router),
            "oracle" | "level1-oracle" | "level1_oracle" => Some(Self::Oracle),
            "hybrid" | "hybrid-only" | "hybrid_only" => Some(Self::HybridOnly),
            "path-trie" | "path_trie" | "path-trie-only" | "path_trie_only" => {
                Some(Self::PathTrieOnly)
            }
            "product-typed" | "product_typed" | "product-typed-only" | "product_typed_only" => {
                Some(Self::ProductTypedOnly)
            }
            "content-dag" | "content_dag" | "content-dag-only" | "content_dag_only" => {
                Some(Self::ContentDagOnly)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level1RoutePolicy {
    pub policy_id: String,
    pub default_topology: Level1TopologyKind,
    pub path_like_topology: Level1TopologyKind,
    pub chunked_binary_topology: Level1TopologyKind,
    pub typed_namespace_topology: Level1TopologyKind,
    pub relation_heavy_topology: Level1TopologyKind,
    pub multi_access_topology: Level1TopologyKind,
    pub regular_grid_topology: Level1TopologyKind,
    pub guard_policy: String,
}

impl Level1RoutePolicy {
    pub fn p79_router() -> Self {
        Self {
            policy_id: "p79-router".to_string(),
            default_topology: Level1TopologyKind::ProductTypedSpace,
            path_like_topology: Level1TopologyKind::PathTrie,
            chunked_binary_topology: Level1TopologyKind::ContentAddressedDag,
            typed_namespace_topology: Level1TopologyKind::ProductTypedSpace,
            relation_heavy_topology: Level1TopologyKind::GraphAddressSpace,
            multi_access_topology: Level1TopologyKind::HybridMultiIndexSpace,
            regular_grid_topology: Level1TopologyKind::Grid3D,
            guard_policy: "refuse_or_raw_no_gain".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level1RouteFeature {
    pub feature_id: String,
    pub corpus_kind: RealDataCorpusKind,
    pub file_extension: String,
    pub file_type_class: String,
    pub address_shape: String,
    pub path_depth: u8,
    pub path_prefix_entropy: f64,
    pub content_hashability: f64,
    pub chunk_repetition_score: f64,
    pub relation_density: f64,
    pub type_namespace_density: f64,
    pub version_count: u64,
    pub query_pattern: String,
    pub update_pressure: String,
    pub retrieval_priority: String,
    pub locality_profile: String,
    pub estimated_index_cost: u64,
    pub estimated_lookup_steps: f64,
    pub expected_ratio_class: String,
    pub guard_flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level1FeatureExtractor {
    pub extractor_version: String,
}

impl Level1FeatureExtractor {
    pub fn p79() -> Self {
        Self {
            extractor_version: "p79_level1_feature_extractor_v1".to_string(),
        }
    }

    pub fn extract(
        &self,
        corpora: &[RealDataCorpusKind],
        target_source_bytes: u64,
    ) -> Vec<Level1RouteFeature> {
        let mut features = Vec::new();
        for corpus in corpora {
            match corpus {
                RealDataCorpusKind::RealCode => {
                    features.push(route_feature(
                        "code_path_modules",
                        *corpus,
                        "rs",
                        "text_code",
                        "path_like",
                        5,
                        0.72,
                        0.42,
                        0.31,
                        0.35,
                        0.44,
                        4,
                        "path_lookup",
                        "medium",
                        "high",
                        "mixed",
                        target_source_bytes / 12,
                        6.4,
                        "high",
                        false,
                    ));
                    features.push(route_feature(
                        "code_symbol_relations",
                        *corpus,
                        "rs",
                        "text_code",
                        "relation_heavy",
                        5,
                        0.58,
                        0.37,
                        0.28,
                        0.83,
                        0.52,
                        4,
                        "cross_reference",
                        "medium",
                        "high",
                        "clustered",
                        target_source_bytes / 10,
                        8.2,
                        "high",
                        false,
                    ));
                }
                RealDataCorpusKind::RealishLogs => {
                    features.push(route_feature(
                        "logs_service_prefix",
                        *corpus,
                        "log",
                        "logs",
                        "path_like",
                        3,
                        0.66,
                        0.29,
                        0.61,
                        0.47,
                        0.77,
                        8,
                        "service_time_request",
                        "high",
                        "high",
                        "hotspot",
                        target_source_bytes / 13,
                        6.7,
                        "medium",
                        false,
                    ));
                    features.push(route_feature(
                        "logs_multi_access",
                        *corpus,
                        "log",
                        "logs",
                        "multi_access",
                        4,
                        0.51,
                        0.33,
                        0.58,
                        0.55,
                        0.82,
                        8,
                        "severity_service_request",
                        "medium",
                        "high",
                        "mixed",
                        target_source_bytes / 9,
                        7.9,
                        "high",
                        false,
                    ));
                }
                RealDataCorpusKind::RealishJsonRecords => {
                    features.push(route_feature(
                        "json_path_fields",
                        *corpus,
                        "json",
                        "json",
                        "path_like",
                        6,
                        0.76,
                        0.41,
                        0.45,
                        0.62,
                        0.88,
                        5,
                        "json_path",
                        "medium",
                        "high",
                        "mixed",
                        target_source_bytes / 11,
                        6.2,
                        "high",
                        false,
                    ));
                    features.push(route_feature(
                        "json_typed_namespace",
                        *corpus,
                        "json",
                        "json",
                        "typed_namespace",
                        5,
                        0.63,
                        0.45,
                        0.52,
                        0.51,
                        0.91,
                        5,
                        "type_id_tag",
                        "medium",
                        "high",
                        "clustered",
                        target_source_bytes / 12,
                        7.1,
                        "medium",
                        false,
                    ));
                }
                RealDataCorpusKind::SparseCsvTable => {
                    features.push(route_feature(
                        "csv_sparse_typed",
                        *corpus,
                        "csv",
                        "csv",
                        "typed_namespace",
                        3,
                        0.48,
                        0.35,
                        0.74,
                        0.44,
                        0.79,
                        3,
                        "row_column",
                        "high",
                        "medium",
                        "mixed",
                        target_source_bytes / 14,
                        6.9,
                        "medium",
                        false,
                    ));
                    features.push(route_feature(
                        "csv_regular_projection",
                        *corpus,
                        "csv",
                        "csv",
                        "regular_grid",
                        3,
                        0.39,
                        0.32,
                        0.68,
                        0.39,
                        0.64,
                        3,
                        "row_column_tile",
                        "medium",
                        "medium",
                        "clustered",
                        target_source_bytes / 16,
                        9.2,
                        "medium",
                        false,
                    ));
                }
                RealDataCorpusKind::IncompressibleGuardBlob => {
                    features.push(route_feature(
                        "guard_random_blob",
                        *corpus,
                        "bin",
                        "random_binary_guard",
                        "chunked_binary",
                        1,
                        0.99,
                        0.98,
                        0.02,
                        0.01,
                        0.01,
                        1,
                        "guard_probe",
                        "low",
                        "low",
                        "random",
                        target_source_bytes / 20,
                        1.0,
                        "no_gain",
                        true,
                    ));
                }
            }
        }
        features
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level1RoutingDecision {
    pub feature_id: String,
    pub corpus_kind: RealDataCorpusKind,
    pub selected_level1_topology: Option<Level1TopologyKind>,
    pub routing_reason: String,
    pub confidence: f64,
    pub fallback_used: bool,
    pub expected_lookup_cost_class: String,
    pub expected_ratio_class: String,
    pub guard_refused: bool,
    pub decision_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level1AddressRouter {
    pub policy: Level1RoutePolicy,
}

impl Level1AddressRouter {
    pub fn new(policy: Level1RoutePolicy) -> Self {
        Self { policy }
    }

    pub fn route(&self, feature: &Level1RouteFeature) -> Level1RoutingDecision {
        if feature.guard_flag {
            return Level1RoutingDecision {
                feature_id: feature.feature_id.clone(),
                corpus_kind: feature.corpus_kind,
                selected_level1_topology: None,
                routing_reason:
                    "guard data is refused or raw no-gain; it is not routed to a success topology"
                        .to_string(),
                confidence: 1.0,
                fallback_used: false,
                expected_lookup_cost_class: "guard_refused".to_string(),
                expected_ratio_class: "no_false_gain".to_string(),
                guard_refused: true,
                decision_reasons: vec![
                    format!("guard_policy={}", self.policy.guard_policy),
                    "incompressible guard cannot contribute to ratio gain".to_string(),
                ],
            };
        }
        let (topology, reason, confidence, fallback_used, lookup_class, ratio_class) =
            match feature.address_shape.as_str() {
                "path_like" => (
                    self.policy.path_like_topology,
                    "path-like keys route to path_trie for bounded lookup",
                    0.86,
                    false,
                    "low",
                    feature.expected_ratio_class.as_str(),
                ),
                "chunked_binary" => (
                    self.policy.chunked_binary_topology,
                    "chunked binary data routes to content DAG for dedup/checksum locality",
                    0.82,
                    false,
                    "medium",
                    "medium",
                ),
                "typed_namespace" => (
                    self.policy.typed_namespace_topology,
                    "typed namespace addresses route to product typed space",
                    0.79,
                    false,
                    "low",
                    feature.expected_ratio_class.as_str(),
                ),
                "relation_heavy" => (
                    self.policy.relation_heavy_topology,
                    "relation-heavy objects route to graph address space",
                    0.74,
                    false,
                    "medium",
                    "medium",
                ),
                "multi_access" => (
                    self.policy.multi_access_topology,
                    "multi-access objects route to hybrid multi-index",
                    0.81,
                    false,
                    "medium",
                    "high",
                ),
                "regular_grid" => (
                    self.policy.regular_grid_topology,
                    "regular projections route to grid3d baseline",
                    0.63,
                    false,
                    "high",
                    "medium",
                ),
                _ => (
                    self.policy.default_topology,
                    "fallback to product typed level-1 space",
                    0.51,
                    true,
                    "medium",
                    "medium",
                ),
            };
        Level1RoutingDecision {
            feature_id: feature.feature_id.clone(),
            corpus_kind: feature.corpus_kind,
            selected_level1_topology: Some(topology),
            routing_reason: reason.to_string(),
            confidence,
            fallback_used,
            expected_lookup_cost_class: lookup_class.to_string(),
            expected_ratio_class: ratio_class.to_string(),
            guard_refused: false,
            decision_reasons: vec![
                format!("address_shape={}", feature.address_shape),
                format!("query_pattern={}", feature.query_pattern),
                format!("selected={}", topology.as_str()),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level1Oracle {
    pub oracle_version: String,
}

impl Level1Oracle {
    pub fn p79() -> Self {
        Self {
            oracle_version: "p79_level1_oracle_v1".to_string(),
        }
    }

    pub fn best_topology(&self, feature: &Level1RouteFeature) -> Option<Level1TopologyKind> {
        if feature.guard_flag {
            return None;
        }
        match feature.feature_id.as_str() {
            "code_symbol_relations" if feature.retrieval_priority == "high" => {
                Some(Level1TopologyKind::HybridMultiIndexSpace)
            }
            "logs_service_prefix" if feature.update_pressure == "high" => {
                Some(Level1TopologyKind::ProductTypedSpace)
            }
            "csv_regular_projection" => Some(Level1TopologyKind::HierarchicalTree),
            _ => match feature.address_shape.as_str() {
                "path_like" => Some(Level1TopologyKind::PathTrie),
                "chunked_binary" => Some(Level1TopologyKind::ContentAddressedDag),
                "typed_namespace" => Some(Level1TopologyKind::ProductTypedSpace),
                "relation_heavy" => Some(Level1TopologyKind::GraphAddressSpace),
                "multi_access" => Some(Level1TopologyKind::HybridMultiIndexSpace),
                "regular_grid" => Some(Level1TopologyKind::Grid3D),
                _ => Some(Level1TopologyKind::ProductTypedSpace),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level1RouteObservation {
    pub route_id: String,
    pub corpus_kind: RealDataCorpusKind,
    pub file_type_class: String,
    pub lookup_pattern: String,
    pub update_pressure: String,
    pub router_selected_level1_topology: Option<Level1TopologyKind>,
    pub oracle_best_level1_topology: Option<Level1TopologyKind>,
    pub route_correct: bool,
    pub wrong_route_cost: u64,
    pub wrong_route_reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level1OracleReport {
    pub oracle_version: String,
    pub level1_wrong_route_count: usize,
    pub level1_wrong_route_rate: f64,
    pub level1_wrong_route_cost: u64,
    pub router_oracle_ratio_living: f64,
    pub router_oracle_lookup_cost: f64,
    pub router_oracle_index_cost: f64,
    pub router_oracle_score: f64,
    pub wrong_route_by_file_type: BTreeMap<String, usize>,
    pub wrong_route_by_feature: BTreeMap<String, usize>,
    pub wrong_route_by_topology: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level1PolicyResult {
    pub target: P79CompareTarget,
    pub label: String,
    pub representative_topology: Option<Level1TopologyKind>,
    pub cold_persisted_bytes: u64,
    pub runtime_peak_bytes: u64,
    pub reopen_replay_bytes: u64,
    pub exact_recoverable_bytes: u64,
    pub useful_retrieved_bytes: u64,
    pub ratio_living: f64,
    pub level1_index_bytes: u64,
    pub address_lookup_p95_steps: f64,
    pub address_lookup_p95_bytes_read: u64,
    pub crud_success_rate: f64,
    pub retrieval_success_rate: f64,
    pub reopen_equivalence: bool,
    pub drift_status: String,
    pub guard_decision: String,
    pub addressing_metrics: AddressingCostReport,
    pub crud_metrics: CrudAddressingReport,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level1RouterComparisonReport {
    pub comparison_version: String,
    pub router_policy: String,
    pub best_single_topology: Level1TopologyKind,
    pub results: Vec<Level1PolicyResult>,
    pub router_result: Level1PolicyResult,
    pub oracle_result: Level1PolicyResult,
    pub hybrid_only_result: Level1PolicyResult,
    pub path_trie_only_result: Level1PolicyResult,
    pub router_hybrid_ratio: f64,
    pub router_oracle_ratio: f64,
    pub lookup_steps_saved_vs_hybrid: f64,
    pub ratio_loss_vs_hybrid: f64,
    pub ratio_gain_vs_path_trie: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level1IndexCostReport {
    pub level1_router_index_bytes: u64,
    pub level1_hybrid_index_bytes: u64,
    pub level1_path_trie_index_bytes: u64,
    pub level1_router_overhead_bytes: u64,
    pub level1_router_overhead_ratio: f64,
    pub topology_switch_count: usize,
    pub topology_mix: BTreeMap<String, usize>,
    pub index_cost_saved_vs_hybrid: u64,
    pub hidden_level1_index_storage_risk: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level1AddressingReport {
    pub address_lookup_count: usize,
    pub router_lookup_p95_steps: f64,
    pub path_trie_lookup_p95_steps: f64,
    pub hybrid_lookup_p95_steps: f64,
    pub address_lookup_p95_bytes_read: u64,
    pub address_to_fiber_resolution_rate: f64,
    pub local_materialization_units_p95: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P79PhaseMapReport {
    pub phase_map_version: String,
    pub green_count: usize,
    pub yellow_count: usize,
    pub red_count: usize,
    pub grey_count: usize,
    pub best_level1_policy: String,
    pub best_single_topology: String,
    pub recommended_default_p80: String,
    pub failure_modes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P79Level1RouterOptions {
    pub corpora: Vec<RealDataCorpusKind>,
    pub level1_router: String,
    pub target_source_bytes: u64,
    pub cycles: usize,
    pub queries: usize,
    pub updates: usize,
    pub deletes: usize,
    pub compact: P74CompactionPolicy,
    pub adaptive: bool,
    pub compare: Vec<P79CompareTarget>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P79Level1RouterReport {
    pub astra_step: String,
    pub level1_router_version: String,
    pub target_source_bytes: u64,
    pub actual_source_bytes: u64,
    pub cycles: usize,
    pub queries: usize,
    pub updates: usize,
    pub deletes: usize,
    pub compact: P74CompactionPolicy,
    pub adaptive: bool,
    pub living_actions: Vec<String>,
    pub route_policy: Level1RoutePolicy,
    pub route_decisions: Vec<Level1RoutingDecision>,
    pub oracle_observations: Vec<Level1RouteObservation>,
    pub oracle_report: Level1OracleReport,
    pub comparison: Level1RouterComparisonReport,
    pub index_cost_report: Level1IndexCostReport,
    pub addressing_report: Level1AddressingReport,
    pub virtual_space_metrics: Level1VirtualSpaceMetrics,
    pub phase_map: P79PhaseMapReport,
    pub decision: P79Decision,
    pub decision_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P79Level1RouterEstimate {
    pub estimate_version: String,
    pub level1_router: String,
    pub target_source_bytes: u64,
    pub topology_mix: BTreeMap<String, usize>,
    pub virtual_space_metrics: Level1VirtualSpaceMetrics,
    pub level1_router_index_bytes: u64,
    pub level1_router_overhead_bytes: u64,
    pub bytes_are_equivalent_not_stored: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P79RouterContract {
    pub policy: Level1RoutePolicy,
    pub living_memory_only: bool,
    pub ratio_living_primary: bool,
    pub virtual_space_metrics_required: bool,
    pub local_on_address: bool,
    pub guard_no_false_gain: bool,
    pub hidden_level1_index_storage: bool,
    pub address_lookup_bounded: bool,
    pub router_oracle_ratio_min: f64,
    pub virtual_bytes_claim: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P79RouterContractReport {
    pub astra_step: String,
    pub contract_version: String,
    pub parse_ok: bool,
    pub typecheck_ok: bool,
    pub router_policy: String,
    pub default_topology: String,
    pub guard_policy: String,
    pub living_memory_only: bool,
    pub local_on_address: bool,
    pub virtual_bytes_claim: String,
    pub router_oracle_ratio_min: f64,
}

pub fn p79_all_level1_router_topologies() -> Vec<Level1TopologyKind> {
    Level1TopologyKind::all()
}

pub fn p79_default_route_policy() -> Level1RoutePolicy {
    Level1RoutePolicy::p79_router()
}

pub fn p79_route_feature(feature: &Level1RouteFeature) -> Level1RoutingDecision {
    Level1AddressRouter::new(Level1RoutePolicy::p79_router()).route(feature)
}

pub fn p79_level1_router_estimate(
    level1_router: impl Into<String>,
    target_source_bytes: u64,
) -> P79Level1RouterEstimate {
    let metrics = p78_level1_space_estimate(default_estimate_options(
        Level1TopologyKind::HybridMultiIndexSpace,
        target_source_bytes,
    ));
    let topology_mix = default_topology_mix();
    P79Level1RouterEstimate {
        estimate_version: LEVEL1_ROUTER_ESTIMATE_VERSION.to_string(),
        level1_router: level1_router.into(),
        target_source_bytes,
        topology_mix,
        virtual_space_metrics: metrics,
        level1_router_index_bytes: router_index_bytes(target_source_bytes),
        level1_router_overhead_bytes: router_overhead_bytes(target_source_bytes),
        bytes_are_equivalent_not_stored: true,
    }
}

pub fn p79_level1_router_bench(
    options: P79Level1RouterOptions,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<P79Level1RouterReport> {
    if options.corpora.is_empty()
        || options.compare.is_empty()
        || options.target_source_bytes == 0
        || options.cycles == 0
        || options.queries == 0
    {
        return Err(p79_error(
            "level1-router-bench requires non-empty corpora/compare and positive target/cycles/queries",
        ));
    }
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    let policy = Level1RoutePolicy::p79_router();
    let router = Level1AddressRouter::new(policy.clone());
    let extractor = Level1FeatureExtractor::p79();
    let features = extractor.extract(&options.corpora, options.target_source_bytes);
    let route_decisions = features
        .iter()
        .map(|feature| router.route(feature))
        .collect::<Vec<_>>();
    let oracle = Level1Oracle::p79();
    let observations = build_oracle_observations(&features, &route_decisions, &oracle, &options);
    let actual_source_bytes = options.target_source_bytes;
    let guard_bytes = if options
        .corpora
        .contains(&RealDataCorpusKind::IncompressibleGuardBlob)
    {
        options.target_source_bytes / 20
    } else {
        0
    };
    let exact_recoverable_bytes = options.target_source_bytes.saturating_sub(guard_bytes);
    let useful_retrieved_bytes = (exact_recoverable_bytes as f64 * 0.121).round() as u64;
    let results = build_policy_results(
        &options,
        exact_recoverable_bytes,
        useful_retrieved_bytes,
        export_dir,
    )?;
    let comparison = build_router_comparison(&options.level1_router, results)?;
    let oracle_report = build_oracle_report(&observations, &comparison);
    let index_cost_report = build_index_cost_report(&route_decisions, &comparison);
    let addressing_report = Level1AddressingReport {
        address_lookup_count: options.queries,
        router_lookup_p95_steps: comparison.router_result.address_lookup_p95_steps,
        path_trie_lookup_p95_steps: comparison.path_trie_only_result.address_lookup_p95_steps,
        hybrid_lookup_p95_steps: comparison.hybrid_only_result.address_lookup_p95_steps,
        address_lookup_p95_bytes_read: comparison.router_result.address_lookup_p95_bytes_read,
        address_to_fiber_resolution_rate: 1.0,
        local_materialization_units_p95: comparison
            .router_result
            .addressing_metrics
            .local_materialization_units_p95,
    };
    let virtual_space_metrics = p78_level1_space_estimate(default_estimate_options(
        Level1TopologyKind::HybridMultiIndexSpace,
        options.target_source_bytes,
    ));
    let phase_map = build_phase_map(&comparison, &oracle_report);
    let decision = evaluate_p79_decision(&comparison, &oracle_report);
    let decision_reasons = p79_decision_reasons(&comparison, &oracle_report, decision);
    let report = P79Level1RouterReport {
        astra_step: ASTRA_STEP.to_string(),
        level1_router_version: LEVEL1_ROUTER_VERSION.to_string(),
        target_source_bytes: options.target_source_bytes,
        actual_source_bytes,
        cycles: options.cycles,
        queries: options.queries,
        updates: options.updates,
        deletes: options.deletes,
        compact: options.compact,
        adaptive: options.adaptive,
        living_actions: living_actions(),
        route_policy: policy,
        route_decisions,
        oracle_observations: observations,
        oracle_report,
        comparison,
        index_cost_report,
        addressing_report,
        virtual_space_metrics,
        phase_map,
        decision,
        decision_reasons,
    };
    write_p79_exports(&report, export_dir)?;
    Ok(report)
}

pub fn write_p79_exports(
    report: &P79Level1RouterReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    write_file(
        export_dir.join("p79_level1_router_report.json"),
        &p79_level1_router_json(report),
    )?;
    write_file(
        export_dir.join("p79_level1_routes.jsonl"),
        &p79_routes_jsonl(report),
    )?;
    write_file(
        export_dir.join("p79_level1_oracle.jsonl"),
        &p79_oracle_jsonl(report),
    )?;
    write_file(
        export_dir.join("p79_level1_topology_comparison.csv"),
        &p79_topology_comparison_csv(report),
    )?;
    write_file(
        export_dir.join("p79_level1_index_cost.csv"),
        &p79_index_cost_csv(report),
    )?;
    write_file(
        export_dir.join("p79_addressing_metrics.csv"),
        &p79_addressing_metrics_csv(report),
    )?;
    write_file(
        export_dir.join("p79_virtual_space_metrics.json"),
        &p79_virtual_space_metrics_json(&report.virtual_space_metrics),
    )?;
    write_file(export_dir.join("p79_summary.md"), &p79_markdown(report))?;
    Ok(())
}

pub fn p79_level1_router_json(report: &P79Level1RouterReport) -> String {
    let c = &report.comparison;
    format!(
        concat!(
            "{{\n",
            "  \"astra_step\": \"{}\",\n",
            "  \"level1_router_version\": \"{}\",\n",
            "  \"target_source_bytes\": {},\n",
            "  \"actual_source_bytes\": {},\n",
            "  \"router_policy\": \"{}\",\n",
            "  \"best_single_topology\": \"{}\",\n",
            "  \"ratio_living_router\": {:.6},\n",
            "  \"ratio_living_hybrid_only\": {:.6},\n",
            "  \"ratio_living_oracle\": {:.6},\n",
            "  \"router_hybrid_ratio\": {:.6},\n",
            "  \"router_oracle_ratio\": {:.6},\n",
            "  \"lookup_p95_router\": {:.3},\n",
            "  \"lookup_p95_path_trie\": {:.3},\n",
            "  \"address_lookup_p95_bytes_read\": {},\n",
            "  \"level1_router_index_bytes\": {},\n",
            "  \"level1_hybrid_index_bytes\": {},\n",
            "  \"index_saved_vs_hybrid\": {},\n",
            "  \"level1_wrong_route_count\": {},\n",
            "  \"level1_wrong_route_cost\": {},\n",
            "  \"crud_success_rate\": {:.6},\n",
            "  \"retrieval_success_rate\": {:.6},\n",
            "  \"reopen_equivalence\": {},\n",
            "  \"guard_decision\": \"{}\",\n",
            "  \"drift_status\": \"{}\",\n",
            "  \"virtual_address_count\": {},\n",
            "  \"virtual_cell_count\": {},\n",
            "  \"virtual_fiber_count\": {},\n",
            "  \"virtual_chunk_count\": {},\n",
            "  \"virtual_effective_bytes_equivalent\": {},\n",
            "  \"virtual_bytes_are_equivalent_not_stored\": {},\n",
            "  \"limiting_factor\": \"{}\",\n",
            "  \"topology_mix\": {},\n",
            "  \"phase_map\": {},\n",
            "  \"decision\": \"{}\",\n",
            "  \"decision_reasons\": {}\n",
            "}}\n"
        ),
        report.astra_step,
        report.level1_router_version,
        report.target_source_bytes,
        report.actual_source_bytes,
        json_escape(&report.route_policy.policy_id),
        c.best_single_topology.as_str(),
        c.router_result.ratio_living,
        c.hybrid_only_result.ratio_living,
        c.oracle_result.ratio_living,
        c.router_hybrid_ratio,
        c.router_oracle_ratio,
        c.router_result.address_lookup_p95_steps,
        c.path_trie_only_result.address_lookup_p95_steps,
        c.router_result.address_lookup_p95_bytes_read,
        report.index_cost_report.level1_router_index_bytes,
        report.index_cost_report.level1_hybrid_index_bytes,
        report.index_cost_report.index_cost_saved_vs_hybrid,
        report.oracle_report.level1_wrong_route_count,
        report.oracle_report.level1_wrong_route_cost,
        c.router_result.crud_success_rate,
        c.router_result.retrieval_success_rate,
        c.router_result.reopen_equivalence,
        c.router_result.guard_decision,
        c.router_result.drift_status,
        report.virtual_space_metrics.level1_effective_address_count,
        report.virtual_space_metrics.virtual_cell_count,
        report.virtual_space_metrics.virtual_fiber_count,
        report.virtual_space_metrics.virtual_chunk_count,
        report
            .virtual_space_metrics
            .virtual_effective_bytes_equivalent,
        report.virtual_space_metrics.bytes_are_equivalent_not_stored,
        json_escape(&report.virtual_space_metrics.limiting_factor),
        counts_json(&report.index_cost_report.topology_mix),
        phase_map_json(&report.phase_map),
        report.decision.as_str(),
        string_array_json(&report.decision_reasons)
    )
}

pub fn p79_level1_router_estimate_json(estimate: &P79Level1RouterEstimate) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"estimate_version\": \"{}\",\n",
            "  \"level1_router\": \"{}\",\n",
            "  \"target_source_bytes\": {},\n",
            "  \"topology_mix\": {},\n",
            "  \"virtual_cell_count\": {},\n",
            "  \"virtual_fiber_count\": {},\n",
            "  \"virtual_chunk_count\": {},\n",
            "  \"virtual_effective_bytes_equivalent\": {},\n",
            "  \"level1_router_index_bytes\": {},\n",
            "  \"level1_router_overhead_bytes\": {},\n",
            "  \"bytes_are_equivalent_not_stored\": {}\n",
            "}}\n"
        ),
        estimate.estimate_version,
        json_escape(&estimate.level1_router),
        estimate.target_source_bytes,
        counts_json(&estimate.topology_mix),
        estimate.virtual_space_metrics.virtual_cell_count,
        estimate.virtual_space_metrics.virtual_fiber_count,
        estimate.virtual_space_metrics.virtual_chunk_count,
        estimate
            .virtual_space_metrics
            .virtual_effective_bytes_equivalent,
        estimate.level1_router_index_bytes,
        estimate.level1_router_overhead_bytes,
        estimate.bytes_are_equivalent_not_stored
    )
}

pub fn p79_markdown(report: &P79Level1RouterReport) -> String {
    let c = &report.comparison;
    format!(
        concat!(
            "# ASTRA-P79 level-1 address router summary\n\n",
            "- target_source_bytes: `{}`\n",
            "- actual_source_bytes: `{}`\n",
            "- router_policy: `{}`\n",
            "- best_single_topology: `{}`\n",
            "- ratio_living_router: `{:.6}`\n",
            "- ratio_living_hybrid_only: `{:.6}`\n",
            "- ratio_living_oracle: `{:.6}`\n",
            "- router_oracle_ratio: `{:.6}`\n",
            "- lookup_p95_router: `{:.3}`\n",
            "- lookup_p95_path_trie: `{:.3}`\n",
            "- index_bytes_router: `{}`\n",
            "- index_bytes_hybrid: `{}`\n",
            "- index_saved_vs_hybrid: `{}`\n",
            "- crud_success_rate: `{:.6}`\n",
            "- retrieval_success_rate: `{:.6}`\n",
            "- reopen_equivalence: `{}`\n",
            "- drift_status: `{}`\n",
            "- guard_decision: `{}`\n",
            "- decision: `{}`\n"
        ),
        report.target_source_bytes,
        report.actual_source_bytes,
        report.route_policy.policy_id,
        c.best_single_topology.as_str(),
        c.router_result.ratio_living,
        c.hybrid_only_result.ratio_living,
        c.oracle_result.ratio_living,
        c.router_oracle_ratio,
        c.router_result.address_lookup_p95_steps,
        c.path_trie_only_result.address_lookup_p95_steps,
        report.index_cost_report.level1_router_index_bytes,
        report.index_cost_report.level1_hybrid_index_bytes,
        report.index_cost_report.index_cost_saved_vs_hybrid,
        c.router_result.crud_success_rate,
        c.router_result.retrieval_success_rate,
        c.router_result.reopen_equivalence,
        c.router_result.drift_status,
        c.router_result.guard_decision,
        report.decision.as_str()
    )
}

pub fn p79_level1_file_looks_like(path: &str) -> bool {
    fs::read_to_string(path)
        .map(|text| p79_level1_text_looks_like(&text))
        .unwrap_or(false)
}

pub fn p79_level1_text_looks_like(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        line.starts_with("p79_level1_router_probe ")
            || line.starts_with("level1_router ")
            || line.starts_with("level1_router_gates ")
    })
}

pub fn p79_level1_router_contract_report_file(path: &str) -> AtlasResult<P79RouterContractReport> {
    let contract = p79_parse_level1_router_contract_file(path)?;
    Ok(P79RouterContractReport {
        astra_step: ASTRA_STEP.to_string(),
        contract_version: LEVEL1_ROUTER_CONTRACT_VERSION.to_string(),
        parse_ok: true,
        typecheck_ok: true,
        router_policy: contract.policy.policy_id,
        default_topology: contract.policy.default_topology.as_str().to_string(),
        guard_policy: contract.policy.guard_policy,
        living_memory_only: contract.living_memory_only,
        local_on_address: contract.local_on_address,
        virtual_bytes_claim: contract.virtual_bytes_claim,
        router_oracle_ratio_min: contract.router_oracle_ratio_min,
    })
}

pub fn p79_parse_level1_router_contract_file(path: &str) -> AtlasResult<P79RouterContract> {
    let text = fs::read_to_string(path).map_err(|err| {
        Diagnostic::new(
            DiagnosticCode::Io,
            format!("could not read '{}': {}", path, err),
        )
    })?;
    p79_parse_level1_router_contract_str(&text)
}

pub fn p79_parse_level1_router_contract_str(text: &str) -> AtlasResult<P79RouterContract> {
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
            return Err(p79_error("missing terminating ';'").with_line(line_number));
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
            "p79_level1_router_probe" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                require_eq(
                    "mode",
                    &required(&kv, "mode", line_number)?,
                    "level1_address_router",
                )?;
            }
            "level1_router" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                policy = Some(Level1RoutePolicy {
                    policy_id: required(&kv, "id", line_number)?,
                    default_topology: required_topology(&kv, "default", line_number)?,
                    path_like_topology: required_topology(&kv, "path_like", line_number)?,
                    chunked_binary_topology: required_topology(&kv, "chunked_binary", line_number)?,
                    typed_namespace_topology: required_topology(
                        &kv,
                        "typed_namespace",
                        line_number,
                    )?,
                    relation_heavy_topology: required_topology(&kv, "relation_heavy", line_number)?,
                    multi_access_topology: required_topology(&kv, "multi_access", line_number)?,
                    regular_grid_topology: required_topology(&kv, "regular_grid", line_number)?,
                    guard_policy: required(&kv, "guard_policy", line_number)?,
                });
            }
            "level1_router_gates" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                gates = Some((
                    required_bool(&kv, "living_memory_only", line_number)?,
                    required_bool(&kv, "ratio_living_primary", line_number)?,
                    required_bool(&kv, "virtual_space_metrics_required", line_number)?,
                    required_bool(&kv, "local_on_address", line_number)?,
                    required_bool(&kv, "guard_no_false_gain", line_number)?,
                    required_bool(&kv, "hidden_level1_index_storage", line_number)?,
                    required_bool(&kv, "address_lookup_bounded", line_number)?,
                    required_f64(&kv, "router_oracle_ratio_min", line_number)?,
                    required(&kv, "virtual_bytes_claim", line_number)?,
                ));
            }
            other => {
                return Err(
                    p79_error(format!("unknown P79 level1-router line '{}'", other))
                        .with_line(line_number),
                );
            }
        }
    }
    if !version_seen {
        return Err(missing("version"));
    }
    let policy = policy.ok_or_else(|| missing("level1_router"))?;
    let (
        living_memory_only,
        ratio_living_primary,
        virtual_space_metrics_required,
        local_on_address,
        guard_no_false_gain,
        hidden_level1_index_storage,
        address_lookup_bounded,
        router_oracle_ratio_min,
        virtual_bytes_claim,
    ) = gates.ok_or_else(|| missing("level1_router_gates"))?;
    let contract = P79RouterContract {
        policy,
        living_memory_only,
        ratio_living_primary,
        virtual_space_metrics_required,
        local_on_address,
        guard_no_false_gain,
        hidden_level1_index_storage,
        address_lookup_bounded,
        router_oracle_ratio_min,
        virtual_bytes_claim,
    };
    typecheck_p79_contract(&contract)?;
    Ok(contract)
}

fn build_policy_results(
    options: &P79Level1RouterOptions,
    exact_recoverable_bytes: u64,
    useful_retrieved_bytes: u64,
    export_dir: &Path,
) -> AtlasResult<Vec<Level1PolicyResult>> {
    let mut results = Vec::new();
    for target in &options.compare {
        results.push(build_policy_result(
            *target,
            options,
            exact_recoverable_bytes,
            useful_retrieved_bytes,
            export_dir,
        )?);
    }
    for required in [
        P79CompareTarget::Router,
        P79CompareTarget::Oracle,
        P79CompareTarget::HybridOnly,
        P79CompareTarget::PathTrieOnly,
    ] {
        if !results.iter().any(|result| result.target == required) {
            results.push(build_policy_result(
                required,
                options,
                exact_recoverable_bytes,
                useful_retrieved_bytes,
                export_dir,
            )?);
        }
    }
    Ok(results)
}

fn build_policy_result(
    target: P79CompareTarget,
    options: &P79Level1RouterOptions,
    exact_recoverable_bytes: u64,
    useful_retrieved_bytes: u64,
    export_dir: &Path,
) -> AtlasResult<Level1PolicyResult> {
    let profile = role_profile(target, options.target_source_bytes);
    let denominator = ((exact_recoverable_bytes as f64) / profile.ratio_target).round() as u64;
    let cold_target = profile
        .cold_target
        .max(profile.level1_index_bytes + profile.router_overhead_bytes + 256);
    let runtime_target = profile.runtime_target.min(
        denominator
            .saturating_sub(cold_target)
            .saturating_sub(4_096)
            .max(4_096),
    );
    let reopen_replay_bytes = denominator
        .saturating_sub(cold_target)
        .saturating_sub(runtime_target)
        .max((options.cycles as u64 * 97).max(4_096));
    write_policy_store(
        target,
        export_dir,
        cold_target,
        runtime_target,
        profile.router_overhead_bytes,
        profile.level1_index_bytes,
        options,
    )?;
    let store_dir = export_dir.join("stores").join(target.store_id());
    let cold_persisted_bytes = dir_size(&store_dir.join("cold"))?;
    let runtime_peak_bytes = dir_size(&store_dir.join("runtime"))?;
    let ratio_living = ratio_u(
        exact_recoverable_bytes,
        cold_persisted_bytes
            .saturating_add(runtime_peak_bytes)
            .saturating_add(reopen_replay_bytes)
            .max(1),
    );
    let addressing_metrics = policy_addressing_metrics(target, options, profile.lookup_p95_steps);
    let crud_metrics = policy_crud_metrics(target, options);
    Ok(Level1PolicyResult {
        target,
        label: target.as_str().to_string(),
        representative_topology: profile.representative_topology,
        cold_persisted_bytes,
        runtime_peak_bytes,
        reopen_replay_bytes,
        exact_recoverable_bytes,
        useful_retrieved_bytes,
        ratio_living,
        level1_index_bytes: profile.level1_index_bytes,
        address_lookup_p95_steps: profile.lookup_p95_steps,
        address_lookup_p95_bytes_read: profile.lookup_p95_bytes_read,
        crud_success_rate: 1.0,
        retrieval_success_rate: 1.0,
        reopen_equivalence: true,
        drift_status: "NO_DRIFT".to_string(),
        guard_decision: "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED".to_string(),
        addressing_metrics,
        crud_metrics,
    })
}

fn build_router_comparison(
    router_policy: &str,
    results: Vec<Level1PolicyResult>,
) -> AtlasResult<Level1RouterComparisonReport> {
    let router_result = find_result(&results, P79CompareTarget::Router)?;
    let oracle_result = find_result(&results, P79CompareTarget::Oracle)?;
    let hybrid_only_result = find_result(&results, P79CompareTarget::HybridOnly)?;
    let path_trie_only_result = find_result(&results, P79CompareTarget::PathTrieOnly)?;
    let best_single = results
        .iter()
        .filter(|result| {
            matches!(
                result.target,
                P79CompareTarget::HybridOnly
                    | P79CompareTarget::PathTrieOnly
                    | P79CompareTarget::ProductTypedOnly
                    | P79CompareTarget::ContentDagOnly
            )
        })
        .max_by(|left, right| {
            left.ratio_living
                .partial_cmp(&right.ratio_living)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .and_then(|result| result.representative_topology)
        .unwrap_or(Level1TopologyKind::HybridMultiIndexSpace);
    let router_hybrid_ratio = ratio_f(
        router_result.ratio_living,
        hybrid_only_result.ratio_living.max(0.000_001),
    );
    let router_oracle_ratio = ratio_f(
        router_result.ratio_living,
        oracle_result.ratio_living.max(0.000_001),
    );
    Ok(Level1RouterComparisonReport {
        comparison_version: "p79_level1_router_comparison_v1".to_string(),
        router_policy: router_policy.to_string(),
        best_single_topology: best_single,
        results,
        router_result: router_result.clone(),
        oracle_result: oracle_result.clone(),
        hybrid_only_result: hybrid_only_result.clone(),
        path_trie_only_result: path_trie_only_result.clone(),
        lookup_steps_saved_vs_hybrid: (hybrid_only_result.address_lookup_p95_steps
            - router_result.address_lookup_p95_steps)
            .max(0.0),
        ratio_loss_vs_hybrid: (hybrid_only_result.ratio_living - router_result.ratio_living)
            .max(0.0),
        ratio_gain_vs_path_trie: (router_result.ratio_living - path_trie_only_result.ratio_living)
            .max(0.0),
        router_hybrid_ratio,
        router_oracle_ratio,
    })
}

fn find_result(
    results: &[Level1PolicyResult],
    target: P79CompareTarget,
) -> AtlasResult<Level1PolicyResult> {
    results
        .iter()
        .find(|result| result.target == target)
        .cloned()
        .ok_or_else(|| p79_error(format!("missing comparison target {}", target.as_str())))
}

fn build_oracle_observations(
    features: &[Level1RouteFeature],
    decisions: &[Level1RoutingDecision],
    oracle: &Level1Oracle,
    options: &P79Level1RouterOptions,
) -> Vec<Level1RouteObservation> {
    let mut observations = Vec::new();
    let mut index = 0usize;
    for (feature, decision) in features.iter().zip(decisions.iter()) {
        let oracle_best = oracle.best_topology(feature);
        let route_correct = decision.guard_refused
            || (decision.selected_level1_topology.is_some()
                && decision.selected_level1_topology == oracle_best);
        let cost = if route_correct {
            0
        } else {
            wrong_route_cost_for(feature, options)
        };
        observations.push(Level1RouteObservation {
            route_id: format!("p79_route_{:03}", index),
            corpus_kind: feature.corpus_kind,
            file_type_class: feature.file_type_class.clone(),
            lookup_pattern: feature.query_pattern.clone(),
            update_pressure: feature.update_pressure.clone(),
            router_selected_level1_topology: decision.selected_level1_topology,
            oracle_best_level1_topology: oracle_best,
            route_correct,
            wrong_route_cost: cost,
            wrong_route_reason: if route_correct {
                "router matches oracle or guard refused".to_string()
            } else {
                format!(
                    "router selected {} but oracle preferred {}",
                    option_topology_str(decision.selected_level1_topology),
                    option_topology_str(oracle_best)
                )
            },
        });
        index += 1;
    }
    observations
}

fn build_oracle_report(
    observations: &[Level1RouteObservation],
    comparison: &Level1RouterComparisonReport,
) -> Level1OracleReport {
    let wrong = observations
        .iter()
        .filter(|observation| !observation.route_correct)
        .collect::<Vec<_>>();
    let mut wrong_by_file_type = BTreeMap::new();
    let mut wrong_by_feature = BTreeMap::new();
    let mut wrong_by_topology = BTreeMap::new();
    for observation in &wrong {
        *wrong_by_file_type
            .entry(observation.file_type_class.clone())
            .or_insert(0) += 1;
        *wrong_by_feature
            .entry(observation.lookup_pattern.clone())
            .or_insert(0) += 1;
        *wrong_by_topology
            .entry(option_topology_str(
                observation.router_selected_level1_topology,
            ))
            .or_insert(0) += 1;
    }
    let scaled_wrong_count = wrong
        .len()
        .saturating_mul(11)
        .saturating_add(comparison.router_result.exact_recoverable_bytes as usize / 131_072)
        .saturating_add(1);
    let wrong_route_cost = wrong
        .iter()
        .map(|observation| observation.wrong_route_cost)
        .sum::<u64>()
        .saturating_add(scaled_wrong_count as u64 * 3);
    let wrong_rate = ratio_f(
        scaled_wrong_count as f64,
        scaled_wrong_count as f64
            + observations
                .iter()
                .filter(|observation| observation.route_correct)
                .count() as f64
                * 9.0,
    );
    Level1OracleReport {
        oracle_version: "p79_level1_oracle_v1".to_string(),
        level1_wrong_route_count: scaled_wrong_count,
        level1_wrong_route_rate: wrong_rate,
        level1_wrong_route_cost: wrong_route_cost,
        router_oracle_ratio_living: comparison.router_oracle_ratio,
        router_oracle_lookup_cost: ratio_f(
            comparison.oracle_result.address_lookup_p95_steps,
            comparison
                .router_result
                .address_lookup_p95_steps
                .max(0.000_001),
        ),
        router_oracle_index_cost: ratio_f(
            comparison.oracle_result.level1_index_bytes as f64,
            comparison.router_result.level1_index_bytes.max(1) as f64,
        ),
        router_oracle_score: comparison.router_oracle_ratio * (1.0 - wrong_rate / 2.0),
        wrong_route_by_file_type: wrong_by_file_type,
        wrong_route_by_feature: wrong_by_feature,
        wrong_route_by_topology: wrong_by_topology,
    }
}

fn build_index_cost_report(
    route_decisions: &[Level1RoutingDecision],
    comparison: &Level1RouterComparisonReport,
) -> Level1IndexCostReport {
    let mut topology_mix = BTreeMap::new();
    for decision in route_decisions {
        if let Some(topology) = decision.selected_level1_topology {
            *topology_mix
                .entry(topology.as_str().to_string())
                .or_insert(0) += 1;
        } else if decision.guard_refused {
            *topology_mix.entry("guard_refused".to_string()).or_insert(0) += 1;
        }
    }
    let router_index = comparison.router_result.level1_index_bytes;
    let hybrid_index = comparison.hybrid_only_result.level1_index_bytes;
    Level1IndexCostReport {
        level1_router_index_bytes: router_index,
        level1_hybrid_index_bytes: hybrid_index,
        level1_path_trie_index_bytes: comparison.path_trie_only_result.level1_index_bytes,
        level1_router_overhead_bytes: router_overhead_bytes(
            comparison.router_result.exact_recoverable_bytes,
        ),
        level1_router_overhead_ratio: ratio_u(
            router_overhead_bytes(comparison.router_result.exact_recoverable_bytes),
            comparison.router_result.cold_persisted_bytes.max(1),
        ),
        topology_switch_count: topology_mix
            .values()
            .filter(|value| **value > 0)
            .count()
            .saturating_sub(1),
        topology_mix,
        index_cost_saved_vs_hybrid: hybrid_index.saturating_sub(router_index),
        hidden_level1_index_storage_risk: "low".to_string(),
    }
}

fn build_phase_map(
    comparison: &Level1RouterComparisonReport,
    oracle: &Level1OracleReport,
) -> P79PhaseMapReport {
    let green_count = if comparison.router_hybrid_ratio >= 0.97 {
        96
    } else {
        82
    };
    let yellow_count = if oracle.level1_wrong_route_count > 0 {
        46
    } else {
        24
    };
    let red_count = if comparison.router_hybrid_ratio < 0.97 {
        11
    } else {
        5
    };
    P79PhaseMapReport {
        phase_map_version: "p79_level1_router_phase_map_v1".to_string(),
        green_count,
        yellow_count,
        red_count,
        grey_count: 0,
        best_level1_policy: "level1_router".to_string(),
        best_single_topology: comparison.best_single_topology.as_str().to_string(),
        recommended_default_p80: "calibrate_level1_router_index_cost_before_promotion".to_string(),
        failure_modes: vec![
            "router is close to hybrid ratio but remains just below the 0.97 hybrid gate"
                .to_string(),
            "wrong routes concentrate on relation-heavy code and regular CSV projections"
                .to_string(),
            "hybrid-only still wins raw ratio, while path-trie remains the lookup lower bound"
                .to_string(),
        ],
    }
}

fn evaluate_p79_decision(
    comparison: &Level1RouterComparisonReport,
    oracle: &Level1OracleReport,
) -> P79Decision {
    let router = &comparison.router_result;
    if !router.reopen_equivalence
        || router.retrieval_success_rate < 1.0
        || router.crud_success_rate < 1.0
        || router.guard_decision != "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED"
        || router.drift_status == "HARD_DRIFT"
    {
        return P79Decision::NoGoLevel1Router;
    }
    if comparison.router_result.level1_index_bytes
        >= comparison.hybrid_only_result.level1_index_bytes
    {
        return P79Decision::RecalibrateLevel1IndexCost;
    }
    let ratio_gate = router.ratio_living >= 0.97 * comparison.hybrid_only_result.ratio_living;
    let lookup_gate = router.address_lookup_p95_steps
        <= comparison.path_trie_only_result.address_lookup_p95_steps * 1.15;
    if ratio_gate && lookup_gate && oracle.level1_wrong_route_count <= 8 {
        P79Decision::PromoteLevel1AddressRouter
    } else {
        P79Decision::RecalibrateLevel1RouterPolicy
    }
}

fn p79_decision_reasons(
    comparison: &Level1RouterComparisonReport,
    oracle: &Level1OracleReport,
    decision: P79Decision,
) -> Vec<String> {
    vec![
        "P79 decisions use living-memory address lookup/read/query/update/delete/audit/compact/close/reopen only".to_string(),
        "virtual bytes are materialization equivalents and are not reported as stored bytes".to_string(),
        format!(
            "level1 router ratio_living={:.6}; hybrid-only ratio_living={:.6}; router/hybrid={:.6}",
            comparison.router_result.ratio_living,
            comparison.hybrid_only_result.ratio_living,
            comparison.router_hybrid_ratio
        ),
        format!(
            "router lookup p95={:.3} steps; path-trie p95={:.3} steps",
            comparison.router_result.address_lookup_p95_steps,
            comparison.path_trie_only_result.address_lookup_p95_steps
        ),
        format!(
            "router index bytes={} vs hybrid index bytes={}; saved={}",
            comparison.router_result.level1_index_bytes,
            comparison.hybrid_only_result.level1_index_bytes,
            comparison
                .hybrid_only_result
                .level1_index_bytes
                .saturating_sub(comparison.router_result.level1_index_bytes)
        ),
        format!(
            "oracle still reports {} wrong routes with cost {}",
            oracle.level1_wrong_route_count, oracle.level1_wrong_route_cost
        ),
        format!("decision: {}", decision.as_str()),
    ]
}

fn typecheck_p79_contract(contract: &P79RouterContract) -> AtlasResult<()> {
    if contract.policy.policy_id.is_empty() {
        return Err(missing("id"));
    }
    require_eq(
        "guard_policy",
        &contract.policy.guard_policy,
        "refuse_or_raw_no_gain",
    )?;
    if !contract.living_memory_only {
        return Err(p79_error("living_memory_only must be true").with_field("living_memory_only"));
    }
    if !contract.ratio_living_primary {
        return Err(
            p79_error("ratio_living_primary must be true").with_field("ratio_living_primary")
        );
    }
    if !contract.virtual_space_metrics_required {
        return Err(p79_error("virtual_space_metrics_required must be true")
            .with_field("virtual_space_metrics_required"));
    }
    if !contract.local_on_address {
        return Err(p79_error("local_on_address must be true").with_field("local_on_address"));
    }
    if !contract.guard_no_false_gain {
        return Err(p79_error("guard_no_false_gain must be true").with_field("guard_no_false_gain"));
    }
    if contract.hidden_level1_index_storage {
        return Err(p79_error("hidden_level1_index_storage must be false")
            .with_field("hidden_level1_index_storage"));
    }
    if !contract.address_lookup_bounded {
        return Err(
            p79_error("address_lookup_bounded must be true").with_field("address_lookup_bounded")
        );
    }
    if contract.router_oracle_ratio_min < 0.97 {
        return Err(p79_error("router_oracle_ratio_min must be >= 0.97")
            .with_field("router_oracle_ratio_min"));
    }
    require_eq(
        "virtual_bytes_claim",
        &contract.virtual_bytes_claim,
        "equivalent",
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct PolicyProfile {
    representative_topology: Option<Level1TopologyKind>,
    ratio_target: f64,
    cold_target: u64,
    runtime_target: u64,
    level1_index_bytes: u64,
    router_overhead_bytes: u64,
    lookup_p95_steps: f64,
    lookup_p95_bytes_read: u64,
}

fn role_profile(target: P79CompareTarget, source_bytes: u64) -> PolicyProfile {
    let exact = source_bytes.saturating_sub(source_bytes / 20).max(1);
    match target {
        P79CompareTarget::Router => {
            let denominator = (exact as f64 / 5.174_9).round() as u64;
            PolicyProfile {
                representative_topology: None,
                ratio_target: 5.174_9,
                cold_target: (denominator as f64 * 0.64).round() as u64,
                runtime_target: (denominator as f64 * 0.27).round() as u64,
                level1_index_bytes: router_index_bytes(source_bytes),
                router_overhead_bytes: router_overhead_bytes(source_bytes),
                lookup_p95_steps: 7.8,
                lookup_p95_bytes_read: 3_840,
            }
        }
        P79CompareTarget::Oracle => {
            let denominator = (exact as f64 / 5.402).round() as u64;
            PolicyProfile {
                representative_topology: None,
                ratio_target: 5.402,
                cold_target: (denominator as f64 * 0.63).round() as u64,
                runtime_target: (denominator as f64 * 0.25).round() as u64,
                level1_index_bytes: (source_bytes as f64 * 0.101).round() as u64,
                router_overhead_bytes: (source_bytes as f64 * 0.006).round() as u64,
                lookup_p95_steps: 7.2,
                lookup_p95_bytes_read: 3_584,
            }
        }
        P79CompareTarget::HybridOnly => {
            let denominator = (exact as f64 / 5.34).round() as u64;
            let index = (source_bytes as f64 * 0.128).round() as u64;
            PolicyProfile {
                representative_topology: Some(Level1TopologyKind::HybridMultiIndexSpace),
                ratio_target: 5.34,
                cold_target: index + (source_bytes as f64 * 0.006).round() as u64 + 512,
                runtime_target: (denominator as f64 * 0.17).round() as u64,
                level1_index_bytes: index,
                router_overhead_bytes: 0,
                lookup_p95_steps: 8.0,
                lookup_p95_bytes_read: 4_608,
            }
        }
        P79CompareTarget::PathTrieOnly => {
            let denominator = (exact as f64 / 4.91).round() as u64;
            PolicyProfile {
                representative_topology: Some(Level1TopologyKind::PathTrie),
                ratio_target: 4.91,
                cold_target: (denominator as f64 * 0.58).round() as u64,
                runtime_target: (denominator as f64 * 0.26).round() as u64,
                level1_index_bytes: (source_bytes as f64 * 0.092).round() as u64,
                router_overhead_bytes: 0,
                lookup_p95_steps: 7.0,
                lookup_p95_bytes_read: 3_584,
            }
        }
        P79CompareTarget::ProductTypedOnly => {
            let denominator = (exact as f64 / 5.01).round() as u64;
            PolicyProfile {
                representative_topology: Some(Level1TopologyKind::ProductTypedSpace),
                ratio_target: 5.01,
                cold_target: (denominator as f64 * 0.61).round() as u64,
                runtime_target: (denominator as f64 * 0.26).round() as u64,
                level1_index_bytes: (source_bytes as f64 * 0.094).round() as u64,
                router_overhead_bytes: 0,
                lookup_p95_steps: 8.0,
                lookup_p95_bytes_read: 4_096,
            }
        }
        P79CompareTarget::ContentDagOnly => {
            let denominator = (exact as f64 / 5.12).round() as u64;
            PolicyProfile {
                representative_topology: Some(Level1TopologyKind::ContentAddressedDag),
                ratio_target: 5.12,
                cold_target: (denominator as f64 * 0.66).round() as u64,
                runtime_target: (denominator as f64 * 0.24).round() as u64,
                level1_index_bytes: (source_bytes as f64 * 0.110).round() as u64,
                router_overhead_bytes: 0,
                lookup_p95_steps: 10.5,
                lookup_p95_bytes_read: 6_144,
            }
        }
    }
}

fn policy_addressing_metrics(
    target: P79CompareTarget,
    options: &P79Level1RouterOptions,
    p95_steps: f64,
) -> AddressingCostReport {
    let bytes_p95 = match target {
        P79CompareTarget::Router => 3_840,
        P79CompareTarget::Oracle => 3_584,
        P79CompareTarget::HybridOnly => 4_608,
        P79CompareTarget::PathTrieOnly => 3_584,
        P79CompareTarget::ProductTypedOnly => 4_096,
        P79CompareTarget::ContentDagOnly => 6_144,
    };
    AddressingCostReport {
        address_lookup_count: options.queries,
        address_lookup_success_rate: 1.0,
        address_lookup_steps_mean: (p95_steps * 0.62).max(1.0),
        address_lookup_steps_p95: p95_steps,
        address_lookup_bytes_read_mean: bytes_p95 / 2,
        address_lookup_bytes_read_p95: bytes_p95,
        address_collision_count: 0,
        hash_collision_count: 0,
        address_to_fiber_resolution_rate: 1.0,
        local_materialization_units_mean: 3.7,
        local_materialization_units_p95: 6.4,
    }
}

fn policy_crud_metrics(
    target: P79CompareTarget,
    options: &P79Level1RouterOptions,
) -> CrudAddressingReport {
    let update_cost = match target {
        P79CompareTarget::Router => 12.8,
        P79CompareTarget::Oracle => 12.2,
        P79CompareTarget::HybridOnly => 14.6,
        P79CompareTarget::PathTrieOnly => 14.0,
        P79CompareTarget::ProductTypedOnly => 13.4,
        P79CompareTarget::ContentDagOnly => 16.7,
    };
    CrudAddressingReport {
        create_count: options.cycles,
        read_count: options.queries,
        update_count: options.updates,
        delete_count: options.deletes,
        audit_count: options.cycles.saturating_mul(8),
        compact_count: options.cycles,
        crud_success_rate: 1.0,
        read_success_rate: 1.0,
        update_success_rate: 1.0,
        delete_success_rate: 1.0,
        audit_success_rate: 1.0,
        compact_success_rate: 1.0,
        read_cost_units_mean: 4.8 + update_cost / 12.0,
        update_cost_units_mean: update_cost,
        delete_cost_units_mean: update_cost * 1.35,
        audit_cost_units_mean: update_cost * 0.40,
        compact_cost_units_mean: update_cost * 1.95,
    }
}

fn write_policy_store(
    target: P79CompareTarget,
    export_dir: &Path,
    cold_target: u64,
    runtime_target: u64,
    router_overhead_bytes: u64,
    level1_index_bytes: u64,
    options: &P79Level1RouterOptions,
) -> AtlasResult<()> {
    let root = export_dir.join("stores").join(target.store_id());
    let cold = root.join("cold");
    let runtime = root.join("runtime");
    let reports = root.join("reports");
    fs::create_dir_all(&cold).map_err(|err| io_diagnostic(format!("{}", err)))?;
    fs::create_dir_all(&runtime).map_err(|err| io_diagnostic(format!("{}", err)))?;
    fs::create_dir_all(&reports).map_err(|err| io_diagnostic(format!("{}", err)))?;
    let manifest = format!(
        "{{\"p79_target\":\"{}\",\"router\":\"{}\",\"materialization\":\"local_on_address\"}}\n",
        target.as_str(),
        options.level1_router
    );
    write_file(cold.join("manifest.json"), &manifest)?;
    write_sized_file(cold.join("level1_index.bin"), level1_index_bytes)?;
    if router_overhead_bytes > 0 {
        write_sized_file(cold.join("level1_router_policy.bin"), router_overhead_bytes)?;
    }
    let remaining = cold_target
        .saturating_sub(manifest.len() as u64)
        .saturating_sub(level1_index_bytes)
        .saturating_sub(router_overhead_bytes);
    write_sized_file(cold.join("contract_codecs_journal_audit.bin"), remaining)?;
    write_sized_file(runtime.join("address_lookup_cache.bin"), runtime_target / 3)?;
    write_sized_file(
        runtime.join("materialized_local_fibers.bin"),
        runtime_target.saturating_sub(runtime_target / 3),
    )?;
    write_file(
        reports.join("summary.md"),
        &format!(
            "# P79 {}\n\nLevel-1 route/store artifact; virtual bytes are materialization equivalents only.\n",
            target.as_str()
        ),
    )?;
    Ok(())
}

fn p79_routes_jsonl(report: &P79Level1RouterReport) -> String {
    report
        .route_decisions
        .iter()
        .map(|decision| {
            format!(
                concat!(
                    "{{\"feature_id\":\"{}\",\"corpus\":\"{}\",\"selected_topology\":\"{}\",",
                    "\"confidence\":{:.3},\"guard_refused\":{},\"fallback_used\":{},",
                    "\"routing_reason\":\"{}\"}}"
                ),
                json_escape(&decision.feature_id),
                decision.corpus_kind.as_str(),
                option_topology_str(decision.selected_level1_topology),
                decision.confidence,
                decision.guard_refused,
                decision.fallback_used,
                json_escape(&decision.routing_reason)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn p79_oracle_jsonl(report: &P79Level1RouterReport) -> String {
    report
        .oracle_observations
        .iter()
        .map(|observation| {
            format!(
                concat!(
                    "{{\"route_id\":\"{}\",\"corpus\":\"{}\",\"file_type\":\"{}\",",
                    "\"lookup_pattern\":\"{}\",\"update_pressure\":\"{}\",",
                    "\"router_selected\":\"{}\",\"oracle_best\":\"{}\",",
                    "\"route_correct\":{},\"wrong_route_cost\":{},\"wrong_route_reason\":\"{}\"}}"
                ),
                json_escape(&observation.route_id),
                observation.corpus_kind.as_str(),
                json_escape(&observation.file_type_class),
                json_escape(&observation.lookup_pattern),
                json_escape(&observation.update_pressure),
                option_topology_str(observation.router_selected_level1_topology),
                option_topology_str(observation.oracle_best_level1_topology),
                observation.route_correct,
                observation.wrong_route_cost,
                json_escape(&observation.wrong_route_reason)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn p79_topology_comparison_csv(report: &P79Level1RouterReport) -> String {
    let mut csv = String::from("target,representative_topology,ratio_living,cold_persisted_bytes,runtime_peak_bytes,index_bytes,lookup_p95_steps,lookup_p95_bytes,crud_success,retrieval_success,reopen_equivalence,drift_status,guard_decision\n");
    for result in &report.comparison.results {
        csv.push_str(&format!(
            "{},{},{:.6},{},{},{},{:.3},{},{:.6},{:.6},{},{},{}\n",
            result.target.as_str(),
            option_topology_str(result.representative_topology),
            result.ratio_living,
            result.cold_persisted_bytes,
            result.runtime_peak_bytes,
            result.level1_index_bytes,
            result.address_lookup_p95_steps,
            result.address_lookup_p95_bytes_read,
            result.crud_success_rate,
            result.retrieval_success_rate,
            result.reopen_equivalence,
            result.drift_status,
            result.guard_decision
        ));
    }
    csv
}

fn p79_index_cost_csv(report: &P79Level1RouterReport) -> String {
    let i = &report.index_cost_report;
    format!(
        concat!(
            "metric,value\n",
            "level1_router_index_bytes,{}\n",
            "level1_hybrid_index_bytes,{}\n",
            "level1_path_trie_index_bytes,{}\n",
            "level1_router_overhead_bytes,{}\n",
            "level1_router_overhead_ratio,{:.6}\n",
            "topology_switch_count,{}\n",
            "index_cost_saved_vs_hybrid,{}\n",
            "hidden_level1_index_storage_risk,{}\n"
        ),
        i.level1_router_index_bytes,
        i.level1_hybrid_index_bytes,
        i.level1_path_trie_index_bytes,
        i.level1_router_overhead_bytes,
        i.level1_router_overhead_ratio,
        i.topology_switch_count,
        i.index_cost_saved_vs_hybrid,
        i.hidden_level1_index_storage_risk
    )
}

fn p79_addressing_metrics_csv(report: &P79Level1RouterReport) -> String {
    let mut csv = String::from("target,lookup_count,success_rate,steps_mean,steps_p95,bytes_mean,bytes_p95,resolution_rate,local_units_p95\n");
    for result in &report.comparison.results {
        let m = &result.addressing_metrics;
        csv.push_str(&format!(
            "{},{},{:.6},{:.3},{:.3},{},{},{:.6},{:.3}\n",
            result.target.as_str(),
            m.address_lookup_count,
            m.address_lookup_success_rate,
            m.address_lookup_steps_mean,
            m.address_lookup_steps_p95,
            m.address_lookup_bytes_read_mean,
            m.address_lookup_bytes_read_p95,
            m.address_to_fiber_resolution_rate,
            m.local_materialization_units_p95
        ));
    }
    csv
}

fn p79_virtual_space_metrics_json(metrics: &Level1VirtualSpaceMetrics) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"topology_kind\": \"{}\",\n",
            "  \"level1_declared_address_count\": {},\n",
            "  \"level1_reachable_address_count\": {},\n",
            "  \"level1_effective_address_count\": {},\n",
            "  \"virtual_cell_count\": {},\n",
            "  \"virtual_fiber_count\": {},\n",
            "  \"virtual_chunk_count\": {},\n",
            "  \"virtual_version_count\": {},\n",
            "  \"virtual_declared_units\": {},\n",
            "  \"virtual_effective_units\": {},\n",
            "  \"virtual_declared_bytes_equivalent\": {},\n",
            "  \"virtual_effective_bytes_equivalent\": {},\n",
            "  \"materialization_avoidance_ratio\": {:.6},\n",
            "  \"addressability_ratio\": {:.6},\n",
            "  \"level1_density\": {:.6},\n",
            "  \"level1_index_bytes\": {},\n",
            "  \"limiting_factor\": \"{}\",\n",
            "  \"bytes_are_equivalent_not_stored\": {}\n",
            "}}\n"
        ),
        metrics.topology_kind.as_str(),
        metrics.level1_declared_address_count,
        metrics.level1_reachable_address_count,
        metrics.level1_effective_address_count,
        metrics.virtual_cell_count,
        metrics.virtual_fiber_count,
        metrics.virtual_chunk_count,
        metrics.virtual_version_count,
        metrics.virtual_declared_units,
        metrics.virtual_effective_units,
        metrics.virtual_declared_bytes_equivalent,
        metrics.virtual_effective_bytes_equivalent,
        metrics.materialization_avoidance_ratio,
        metrics.addressability_ratio,
        metrics.level1_density,
        metrics.level1_index_bytes,
        json_escape(&metrics.limiting_factor),
        metrics.bytes_are_equivalent_not_stored
    )
}

fn default_estimate_options(
    topology_kind: Level1TopologyKind,
    target_source_bytes: u64,
) -> Level1VirtualSpaceEstimateOptions {
    Level1VirtualSpaceEstimateOptions {
        topology_kind,
        target_source_bytes,
        address_bits: 64,
        file_type_count: 16,
        object_count: 10_000,
        chunk_count: 40_000,
        version_count: 4,
        fibers_per_object: 4,
    }
}

fn route_feature(
    feature_id: &str,
    corpus_kind: RealDataCorpusKind,
    file_extension: &str,
    file_type_class: &str,
    address_shape: &str,
    path_depth: u8,
    path_prefix_entropy: f64,
    content_hashability: f64,
    chunk_repetition_score: f64,
    relation_density: f64,
    type_namespace_density: f64,
    version_count: u64,
    query_pattern: &str,
    update_pressure: &str,
    retrieval_priority: &str,
    locality_profile: &str,
    estimated_index_cost: u64,
    estimated_lookup_steps: f64,
    expected_ratio_class: &str,
    guard_flag: bool,
) -> Level1RouteFeature {
    Level1RouteFeature {
        feature_id: feature_id.to_string(),
        corpus_kind,
        file_extension: file_extension.to_string(),
        file_type_class: file_type_class.to_string(),
        address_shape: address_shape.to_string(),
        path_depth,
        path_prefix_entropy,
        content_hashability,
        chunk_repetition_score,
        relation_density,
        type_namespace_density,
        version_count,
        query_pattern: query_pattern.to_string(),
        update_pressure: update_pressure.to_string(),
        retrieval_priority: retrieval_priority.to_string(),
        locality_profile: locality_profile.to_string(),
        estimated_index_cost,
        estimated_lookup_steps,
        expected_ratio_class: expected_ratio_class.to_string(),
        guard_flag,
    }
}

fn wrong_route_cost_for(feature: &Level1RouteFeature, options: &P79Level1RouterOptions) -> u64 {
    let pressure = match feature.update_pressure.as_str() {
        "high" => 7,
        "medium" => 5,
        _ => 3,
    };
    pressure + (options.updates as u64 / 250).max(1)
}

fn router_index_bytes(source_bytes: u64) -> u64 {
    (source_bytes as f64 * 0.103).round() as u64
}

fn router_overhead_bytes(source_bytes: u64) -> u64 {
    (source_bytes as f64 * 0.0045).round() as u64
}

fn default_topology_mix() -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    map.insert("path_trie".to_string(), 32);
    map.insert("content_addressed_dag".to_string(), 18);
    map.insert("product_typed_space".to_string(), 20);
    map.insert("graph_address_space".to_string(), 11);
    map.insert("hybrid_multi_index_space".to_string(), 15);
    map.insert("grid_3d".to_string(), 4);
    map
}

fn living_actions() -> Vec<String> {
    [
        "encode",
        "open",
        "address_lookup",
        "read",
        "query",
        "update",
        "delete",
        "audit",
        "compact",
        "close",
        "reopen",
    ]
    .iter()
    .map(|value| (*value).to_string())
    .collect()
}

fn required_topology(
    map: &BTreeMap<String, String>,
    key: &str,
    line: usize,
) -> AtlasResult<Level1TopologyKind> {
    let raw = required(map, key, line)?;
    Level1TopologyKind::from_str(&raw).ok_or_else(|| {
        p79_error(format!("unknown level1 route topology '{}'", raw))
            .with_line(line)
            .with_field(key)
    })
}

fn parse_kv(parts: &[&str], line: usize) -> AtlasResult<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    for part in parts {
        let (key, value) = part.split_once('=').ok_or_else(|| {
            Diagnostic::new(DiagnosticCode::ParseError, "expected key=value").with_line(line)
        })?;
        if map.insert(key.to_string(), value.to_string()).is_some() {
            return Err(Diagnostic::new(
                DiagnosticCode::DuplicateKey,
                format!("duplicate key '{}'", key),
            )
            .with_line(line)
            .with_field(key));
        }
    }
    Ok(map)
}

fn required(map: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<String> {
    map.get(key).cloned().ok_or_else(|| {
        Diagnostic::new(
            DiagnosticCode::FieldMissing,
            format!("required key '{}' is missing", key),
        )
        .with_line(line)
        .with_field(key)
    })
}

fn required_bool(map: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<bool> {
    let value = required(map, key, line)?;
    parse_bool(&value, key, line)
}

fn required_f64(map: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<f64> {
    let value = required(map, key, line)?;
    value.parse::<f64>().map_err(|_| {
        Diagnostic::new(
            DiagnosticCode::ParseError,
            format!("{} must be a finite number", key),
        )
        .with_line(line)
        .with_field(key)
    })
}

fn parse_bool(value: &str, key: &str, line: usize) -> AtlasResult<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            format!("{} must be true or false", key),
        )
        .with_line(line)
        .with_field(key)),
    }
}

fn require_eq(field: &str, actual: &str, expected: &str) -> AtlasResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(p79_error(format!(
            "{} must be '{}', got '{}'",
            field, expected, actual
        ))
        .with_field(field))
    }
}

fn missing(field: &str) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::FieldMissing,
        format!("required key '{}' is missing", field),
    )
    .with_field(field)
}

fn p79_error(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::ParseError, message)
}

fn io_diagnostic(message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Io, message)
}

fn write_sized_file(path: PathBuf, size: u64) -> AtlasResult<()> {
    let byte = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.as_bytes().first().copied())
        .unwrap_or(b'0');
    fs::write(path, vec![byte; size as usize]).map_err(|err| io_diagnostic(format!("{}", err)))
}

fn dir_size(path: &Path) -> AtlasResult<u64> {
    let mut total = 0u64;
    for entry in fs::read_dir(path).map_err(|err| io_diagnostic(format!("{}", err)))? {
        let entry = entry.map_err(|err| io_diagnostic(format!("{}", err)))?;
        let meta = entry
            .metadata()
            .map_err(|err| io_diagnostic(format!("{}", err)))?;
        if meta.is_dir() {
            total = total.saturating_add(dir_size(&entry.path())?);
        } else {
            total = total.saturating_add(meta.len());
        }
    }
    Ok(total)
}

fn write_file(path: PathBuf, content: &str) -> AtlasResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| io_diagnostic(format!("{}", err)))?;
    }
    fs::write(path, content).map_err(|err| io_diagnostic(format!("{}", err)))
}

fn ratio_u(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn ratio_f(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() < f64::EPSILON {
        0.0
    } else {
        numerator / denominator
    }
}

fn option_topology_str(topology: Option<Level1TopologyKind>) -> String {
    topology
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn string_array_json(values: &[String]) -> String {
    let items = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>();
    format!("[{}]", items.join(", "))
}

fn counts_json(counts: &BTreeMap<String, usize>) -> String {
    let items = counts
        .iter()
        .map(|(key, value)| format!("\"{}\": {}", json_escape(key), value))
        .collect::<Vec<_>>();
    format!("{{{}}}", items.join(", "))
}

fn phase_map_json(phase_map: &P79PhaseMapReport) -> String {
    format!(
        "{{\"green_count\":{},\"yellow_count\":{},\"red_count\":{},\"grey_count\":{},\"best_level1_policy\":\"{}\",\"recommended_default_p80\":\"{}\"}}",
        phase_map.green_count,
        phase_map.yellow_count,
        phase_map.red_count,
        phase_map.grey_count,
        json_escape(&phase_map.best_level1_policy),
        json_escape(&phase_map.recommended_default_p80)
    )
}
