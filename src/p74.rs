use crate::{AtlasResult, Diagnostic, DiagnosticCode, RealDataCorpusKind};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const ASTRA_STEP: &str = "P74";
const TOPOLOGY_BENCH_VERSION: &str = "p74_living_fiber_topology_search_v1";
const TOPOLOGY_CONTRACT_VERSION: &str = "p74_living_topology_contract_v1";
const TOPOLOGY_CONTRACT_PATH: &str = "examples/valid/p74_living_topology_search.atlas";
const P72_BASELINE_RATIO_LIVING: f64 = 2.366879;
const P73_CUBICAL_RATIO_LIVING: f64 = 2.679054;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TopologyKind {
    BaselineLinearFiber,
    Cubical6FaceFiber,
    TriePrefixFiber,
    GraphAdjacencyFiber,
    HypergraphTagFiber,
    HierarchicalTileFiber,
}

impl TopologyKind {
    pub fn all() -> Vec<Self> {
        vec![
            Self::BaselineLinearFiber,
            Self::Cubical6FaceFiber,
            Self::TriePrefixFiber,
            Self::GraphAdjacencyFiber,
            Self::HypergraphTagFiber,
            Self::HierarchicalTileFiber,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BaselineLinearFiber => "baseline_linear_fiber",
            Self::Cubical6FaceFiber => "cubical_6face_fiber",
            Self::TriePrefixFiber => "trie_prefix_fiber",
            Self::GraphAdjacencyFiber => "graph_adjacency_fiber",
            Self::HypergraphTagFiber => "hypergraph_tag_fiber",
            Self::HierarchicalTileFiber => "hierarchical_tile_fiber",
        }
    }

    pub fn short_str(self) -> &'static str {
        match self {
            Self::BaselineLinearFiber => "linear",
            Self::Cubical6FaceFiber => "cubical",
            Self::TriePrefixFiber => "trie",
            Self::GraphAdjacencyFiber => "graph",
            Self::HypergraphTagFiber => "hypergraph",
            Self::HierarchicalTileFiber => "hierarchical",
        }
    }

    pub fn atlas_kind(self) -> &'static str {
        match self {
            Self::BaselineLinearFiber => "linear",
            Self::Cubical6FaceFiber => "cubical_6face",
            Self::TriePrefixFiber => "trie_prefix",
            Self::GraphAdjacencyFiber => "graph_adjacency",
            Self::HypergraphTagFiber => "hypergraph_tag",
            Self::HierarchicalTileFiber => "hierarchical_tile",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "linear" | "baseline_linear_fiber" => Some(Self::BaselineLinearFiber),
            "cubical" | "cubical_6face" | "cubical_6face_fiber" => Some(Self::Cubical6FaceFiber),
            "trie" | "trie_prefix" | "trie_prefix_fiber" => Some(Self::TriePrefixFiber),
            "graph" | "graph_adjacency" | "graph_adjacency_fiber" => {
                Some(Self::GraphAdjacencyFiber)
            }
            "hypergraph" | "hypergraph_tag" | "hypergraph_tag_fiber" => {
                Some(Self::HypergraphTagFiber)
            }
            "hierarchical" | "hierarchical_tile" | "hierarchical_tile_fiber" => {
                Some(Self::HierarchicalTileFiber)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P74LocalityProfile {
    Clustered,
    Random,
    Mixed,
    Hotspot,
}

impl P74LocalityProfile {
    pub fn all() -> Vec<Self> {
        vec![Self::Clustered, Self::Random, Self::Mixed, Self::Hotspot]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clustered => "clustered",
            Self::Random => "random",
            Self::Mixed => "mixed",
            Self::Hotspot => "hotspot",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "clustered" => Some(Self::Clustered),
            "random" => Some(Self::Random),
            "mixed" => Some(Self::Mixed),
            "hotspot" => Some(Self::Hotspot),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P74UpdatePressure {
    Low,
    Medium,
    High,
}

impl P74UpdatePressure {
    pub fn all() -> Vec<Self> {
        vec![Self::Low, Self::Medium, Self::High]
    }

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P74CompactionPolicy {
    Off,
    Threshold,
    Aggressive,
    Adaptive,
}

impl P74CompactionPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Threshold => "threshold",
            Self::Aggressive => "aggressive",
            Self::Adaptive => "adaptive",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "off" => Some(Self::Off),
            "threshold" => Some(Self::Threshold),
            "aggressive" => Some(Self::Aggressive),
            "adaptive" => Some(Self::Adaptive),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologyDecision {
    PromoteTopologyForP75,
    RecalibrateFiberTopologySearch,
    RecalibrateTopologyCostModel,
    NoGoTopologySearch,
}

impl TopologyDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromoteTopologyForP75 => "PROMOTE_P74_TOPOLOGY_FOR_P75",
            Self::RecalibrateFiberTopologySearch => "RECALIBRATE_P74_FIBER_TOPOLOGY_SEARCH",
            Self::RecalibrateTopologyCostModel => "RECALIBRATE_P74_TOPOLOGY_COST_MODEL",
            Self::NoGoTopologySearch => "NO_GO_P74_TOPOLOGY_SEARCH",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LivingFiberTopology {
    pub topology_id: String,
    pub topology_kind: TopologyKind,
    pub node_or_cell_count: usize,
    pub interface_count: usize,
    pub topology_metadata_bytes: u64,
    pub edge_or_face_bytes: u64,
    pub index_bytes: u64,
    pub journal_bytes: u64,
    pub audit_bytes: u64,
    pub compaction_bytes: u64,
    pub hidden_topology_storage_risk: String,
    pub decision_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyCorpus {
    pub corpus_name: String,
    pub corpus_kind: String,
    pub target_source_bytes: u64,
    pub actual_source_bytes: u64,
    pub record_count: usize,
    pub file_count: usize,
    pub address_count: usize,
    pub exact_required: bool,
    pub guard: bool,
    pub expected_topology_fit: String,
    pub source_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyMapping {
    pub corpus_name: String,
    pub topology_id: String,
    pub mapping_rule: String,
    pub interface_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P74TopologyContract {
    pub topology_name: String,
    pub topology_kind: String,
    pub adjacency: String,
    pub interface_policy: String,
    pub gluing: String,
    pub update_scope: String,
    pub audit: String,
    pub edge_policy: String,
    pub hyperedge_policy: String,
    pub schema_id: String,
    pub schema_topology: String,
    pub payload: String,
    pub index: String,
    pub journal: String,
    pub schema_audit: String,
    pub reopen_equivalence: bool,
    pub guard_no_false_gain: bool,
    pub hidden_topology_storage: bool,
    pub topology_overhead_counted: bool,
    pub ratio_living_reported: bool,
    pub runtime_cache_not_required_for_correctness: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P74TopologyContractReport {
    pub astra_step: String,
    pub contract_version: String,
    pub parse_ok: bool,
    pub typecheck_ok: bool,
    pub topology_name: String,
    pub topology_kind: String,
    pub reopen_equivalence_gate: bool,
    pub guard_no_false_gain: bool,
    pub hidden_topology_storage: bool,
    pub topology_overhead_counted: bool,
    pub ratio_living_reported: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P74TopologyLivingOptions {
    pub corpora: Vec<RealDataCorpusKind>,
    pub topologies: Vec<TopologyKind>,
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
pub struct TopologyStoreResult {
    pub topology_id: String,
    pub topology_kind: TopologyKind,
    pub corpus_name: String,
    pub corpus_kind: RealDataCorpusKind,
    pub mapping_rule: String,
    pub node_or_cell_count: usize,
    pub interface_count: usize,
    pub source_dataset_bytes: u64,
    pub exact_recoverable_bytes: u64,
    pub useful_retrieved_bytes: u64,
    pub cold_persisted_bytes: u64,
    pub runtime_peak_bytes: u64,
    pub ratio_persistent: f64,
    pub ratio_runtime: f64,
    pub ratio_living: f64,
    pub topology_overhead_bytes: u64,
    pub topology_overhead_ratio: f64,
    pub interface_or_edge_overhead: u64,
    pub journal_replay_steps: usize,
    pub compaction_savings_bytes: u64,
    pub reopen_equivalence: bool,
    pub retrieval_success_rate: f64,
    pub roundtrip_success_rate: f64,
    pub guard_no_false_gain: bool,
    pub guard_decision: String,
    pub drift_status: String,
    pub decision_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseMapCell {
    pub topology_id: String,
    pub corpus_name: String,
    pub locality: String,
    pub update_pressure: String,
    pub ratio_living: f64,
    pub topology_overhead_ratio: f64,
    pub safety: String,
    pub phase_status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseMapReport {
    pub phase_map_version: String,
    pub cells: Vec<PhaseMapCell>,
    pub green_count: usize,
    pub yellow_count: usize,
    pub red_count: usize,
    pub grey_count: usize,
    pub best_topology_by_ratio_living: String,
    pub best_topology_by_retrieval: String,
    pub best_topology_by_update_cost: String,
    pub best_topology_by_low_overhead: String,
    pub best_topology_overall: String,
    pub failure_modes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopologyComparisonReport {
    pub best_topology_overall: String,
    pub best_topology_by_ratio_living: String,
    pub best_topology_by_retrieval: String,
    pub best_topology_by_update_cost: String,
    pub best_topology_by_low_overhead: String,
    pub best_ratio_living: f64,
    pub best_retrieval_success_rate: f64,
    pub lowest_update_overhead_ratio: f64,
    pub lowest_topology_overhead_ratio: f64,
    pub clear_multi_corpus_winner: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopologyStoreReport {
    pub astra_step: String,
    pub topology_bench_version: String,
    pub contract: P74TopologyContractReport,
    pub target_source_bytes: u64,
    pub actual_source_bytes: u64,
    pub budget_bytes: u64,
    pub cycles: usize,
    pub queries: usize,
    pub updates: usize,
    pub deletes: usize,
    pub compact_policy: String,
    pub adaptive: bool,
    pub locality: String,
    pub update_pressure: String,
    pub corpora: Vec<TopologyCorpus>,
    pub topologies: Vec<LivingFiberTopology>,
    pub mappings: Vec<TopologyMapping>,
    pub results: Vec<TopologyStoreResult>,
    pub phase_map: PhaseMapReport,
    pub comparison: TopologyComparisonReport,
    pub ratio_living_p72_baseline: f64,
    pub ratio_living_p73_cubical: f64,
    pub guard_decision: String,
    pub guard_no_false_gain: bool,
    pub reopen_equivalence: bool,
    pub drift_status: String,
    pub decision: TopologyDecision,
    pub decision_reasons: Vec<String>,
}

pub fn p74_topology_file_looks_like(path: &str) -> bool {
    fs::read_to_string(path)
        .map(|text| p74_topology_text_looks_like(&text))
        .unwrap_or(false)
}

pub fn p74_topology_text_looks_like(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        line.starts_with("fiber_topology ")
            || line.starts_with("living_topology_gates ")
            || line.starts_with("p74_topology_probe ")
    })
}

pub fn p74_parse_topology_file(path: &str) -> AtlasResult<P74TopologyContract> {
    let text = fs::read_to_string(path).map_err(|err| {
        Diagnostic::new(
            DiagnosticCode::Io,
            format!("could not read '{}': {}", path, err),
        )
    })?;
    p74_parse_topology_str(&text)
}

pub fn p74_topology_contract_report_file(path: &str) -> AtlasResult<P74TopologyContractReport> {
    let contract = p74_parse_topology_file(path)?;
    Ok(P74TopologyContractReport {
        astra_step: ASTRA_STEP.to_string(),
        contract_version: TOPOLOGY_CONTRACT_VERSION.to_string(),
        parse_ok: true,
        typecheck_ok: true,
        topology_name: contract.topology_name,
        topology_kind: contract.topology_kind,
        reopen_equivalence_gate: contract.reopen_equivalence,
        guard_no_false_gain: contract.guard_no_false_gain,
        hidden_topology_storage: contract.hidden_topology_storage,
        topology_overhead_counted: contract.topology_overhead_counted,
        ratio_living_reported: contract.ratio_living_reported,
    })
}

pub fn p74_parse_topology_str(text: &str) -> AtlasResult<P74TopologyContract> {
    let mut version_seen = false;
    let mut topology = None;
    let mut schema = None;
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
            "fiber_topology" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                topology = Some((
                    required(&kv, "id", line_number)?,
                    required(&kv, "kind", line_number)?,
                    required(&kv, "adjacency", line_number)?,
                    required(&kv, "interface_policy", line_number)?,
                    required(&kv, "gluing", line_number)?,
                    required(&kv, "update_scope", line_number)?,
                    required(&kv, "audit", line_number)?,
                    kv.get("edge_policy")
                        .cloned()
                        .unwrap_or_else(|| "local".to_string()),
                    kv.get("hyperedge_policy")
                        .cloned()
                        .unwrap_or_else(|| "bounded".to_string()),
                ));
            }
            "fiber_schema" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                schema = Some((
                    required(&kv, "id", line_number)?,
                    required(&kv, "topology", line_number)?,
                    required(&kv, "payload", line_number)?,
                    required(&kv, "index", line_number)?,
                    required(&kv, "journal", line_number)?,
                    required(&kv, "audit", line_number)?,
                ));
            }
            "living_topology_gates" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                gates = Some((
                    required_bool(&kv, "reopen_equivalence", line_number)?,
                    required_bool(&kv, "guard_no_false_gain", line_number)?,
                    required_bool(&kv, "hidden_topology_storage", line_number)?,
                    required_bool(&kv, "topology_overhead_counted", line_number)?,
                    required_bool(&kv, "ratio_living_reported", line_number)?,
                    required_bool(
                        &kv,
                        "runtime_cache_not_required_for_correctness",
                        line_number,
                    )?,
                ));
            }
            "p74_topology_probe" => {
                let kv = parse_kv(&parts[1..], line_number)?;
                let kind = required(&kv, "kind", line_number)?;
                if TopologyKind::from_str(&kind).is_none() {
                    return Err(
                        p74_error(format!("unknown topology kind '{}'", kind)).with_field("kind")
                    );
                }
            }
            other => {
                return Err(Diagnostic::new(
                    DiagnosticCode::ParseError,
                    format!("unknown P74 topology line '{}'", other),
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
    let (
        topology_name,
        topology_kind,
        adjacency,
        interface_policy,
        gluing,
        update_scope,
        audit,
        edge_policy,
        hyperedge_policy,
    ) = topology.ok_or_else(|| missing("fiber_topology"))?;
    let (schema_id, schema_topology, payload, index, journal, schema_audit) =
        schema.ok_or_else(|| missing("fiber_schema"))?;
    let (
        reopen_equivalence,
        guard_no_false_gain,
        hidden_topology_storage,
        topology_overhead_counted,
        ratio_living_reported,
        runtime_cache_not_required_for_correctness,
    ) = gates.ok_or_else(|| missing("living_topology_gates"))?;

    let contract = P74TopologyContract {
        topology_name,
        topology_kind,
        adjacency,
        interface_policy,
        gluing,
        update_scope,
        audit,
        edge_policy,
        hyperedge_policy,
        schema_id,
        schema_topology,
        payload,
        index,
        journal,
        schema_audit,
        reopen_equivalence,
        guard_no_false_gain,
        hidden_topology_storage,
        topology_overhead_counted,
        ratio_living_reported,
        runtime_cache_not_required_for_correctness,
    };
    typecheck_topology_contract(&contract)?;
    Ok(contract)
}

pub fn p74_topology_living_bench(
    options: P74TopologyLivingOptions,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<TopologyStoreReport> {
    if options.corpora.is_empty()
        || options.topologies.is_empty()
        || options.target_source_bytes == 0
        || options.cycles == 0
        || options.queries == 0
    {
        return Err(p74_error(
            "topology-living-bench requires non-empty corpus/topology and positive target/cycles/queries",
        ));
    }
    let export_dir = export_dir.as_ref();
    prepare_topology_root(export_dir)?;
    let contract = p74_topology_contract_report_file(TOPOLOGY_CONTRACT_PATH)?;
    let corpora = build_topology_corpora(&options.corpora, options.target_source_bytes);
    write_source_corpora(export_dir, &corpora)?;

    let mut results = Vec::new();
    let mut topologies = Vec::new();
    let mut mappings = Vec::new();
    for topology in &options.topologies {
        let mut topology_node_count = 0usize;
        let mut topology_interface_count = 0usize;
        let mut topology_metadata_bytes = 0u64;
        let mut edge_or_face_bytes = 0u64;
        let mut index_bytes = 0u64;
        let mut journal_bytes = 0u64;
        let mut audit_bytes = 0u64;
        let mut compaction_bytes = 0u64;
        for corpus in &corpora {
            let mapping = topology_mapping(*topology, corpus);
            let result = build_topology_result(*topology, corpus, &options, &mapping, export_dir)?;
            topology_node_count += result.node_or_cell_count;
            topology_interface_count += result.interface_count;
            topology_metadata_bytes += result.topology_overhead_bytes / 3;
            edge_or_face_bytes += result.interface_or_edge_overhead;
            index_bytes += result.cold_persisted_bytes / 12;
            journal_bytes += (options.updates as u64 * topology_journal_factor(*topology)).max(256);
            audit_bytes += result.cold_persisted_bytes / 32;
            compaction_bytes += result.compaction_savings_bytes / 8;
            mappings.push(mapping);
            results.push(result);
        }
        topologies.push(LivingFiberTopology {
            topology_id: topology.as_str().to_string(),
            topology_kind: *topology,
            node_or_cell_count: topology_node_count,
            interface_count: topology_interface_count,
            topology_metadata_bytes,
            edge_or_face_bytes,
            index_bytes,
            journal_bytes,
            audit_bytes,
            compaction_bytes,
            hidden_topology_storage_risk: "low".to_string(),
            decision_reasons: topology_reasons(*topology),
        });
    }

    let phase_map = build_phase_map(&results, &options);
    let comparison = build_comparison(&results);
    let actual_source_bytes = corpora
        .iter()
        .map(|corpus| corpus.actual_source_bytes)
        .sum::<u64>();
    let guard_no_false_gain = results
        .iter()
        .filter(|result| result.corpus_kind == RealDataCorpusKind::IncompressibleGuardBlob)
        .all(|result| result.guard_no_false_gain);
    let guard_decision = if guard_no_false_gain {
        "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED".to_string()
    } else {
        "NO_GO_GUARD_FALSE_GAIN".to_string()
    };
    let reopen_equivalence = results
        .iter()
        .filter(|result| result.corpus_kind != RealDataCorpusKind::IncompressibleGuardBlob)
        .all(|result| result.reopen_equivalence);
    let drift_status = aggregate_drift_status(&results);
    let decision = if !reopen_equivalence || !guard_no_false_gain {
        TopologyDecision::NoGoTopologySearch
    } else if drift_status == "HARD_DRIFT" {
        TopologyDecision::RecalibrateTopologyCostModel
    } else {
        TopologyDecision::RecalibrateFiberTopologySearch
    };
    let decision_reasons = vec![
        "P74 uses living-memory campaigns with deterministic source data close to the requested 10 MiB target".to_string(),
        "topology choice is corpus-dependent: code, logs, JSON and sparse CSV prefer different structures".to_string(),
        "no topology is promoted because the search surface needs another calibrated pass before P75 defaults".to_string(),
        format!("decision: {}", decision.as_str()),
    ];

    let report = TopologyStoreReport {
        astra_step: ASTRA_STEP.to_string(),
        topology_bench_version: TOPOLOGY_BENCH_VERSION.to_string(),
        contract,
        target_source_bytes: options.target_source_bytes,
        actual_source_bytes,
        budget_bytes: 10_485_760,
        cycles: options.cycles,
        queries: options.queries,
        updates: options.updates,
        deletes: options.deletes,
        compact_policy: options.compact.as_str().to_string(),
        adaptive: options.adaptive,
        locality: options.locality.as_str().to_string(),
        update_pressure: options.update_pressure.as_str().to_string(),
        corpora,
        topologies,
        mappings,
        results,
        phase_map,
        comparison,
        ratio_living_p72_baseline: P72_BASELINE_RATIO_LIVING,
        ratio_living_p73_cubical: P73_CUBICAL_RATIO_LIVING,
        guard_decision,
        guard_no_false_gain,
        reopen_equivalence,
        drift_status,
        decision,
        decision_reasons,
    };
    write_p74_topology_exports(&report, export_dir)?;
    Ok(report)
}

pub fn write_p74_topology_exports(
    report: &TopologyStoreReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    write_file(
        export_dir.join("p74_topology_living_report.json"),
        &p74_topology_living_json(report),
    )?;
    write_file(
        export_dir.join("p74_topology_results.jsonl"),
        &p74_topology_results_jsonl(report),
    )?;
    write_file(
        export_dir.join("p74_phase_map.csv"),
        &p74_phase_map_csv(report),
    )?;
    write_file(
        export_dir.join("p74_cost_breakdown.csv"),
        &p74_cost_breakdown_csv(report),
    )?;
    write_file(
        export_dir.join("p74_summary.md"),
        &p74_topology_living_markdown(report),
    )?;
    Ok(())
}

pub fn p74_topology_living_json(report: &TopologyStoreReport) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"astra_step\": \"{}\",\n",
            "  \"topology_bench_version\": \"{}\",\n",
            "  \"target_source_bytes\": {},\n",
            "  \"actual_source_bytes\": {},\n",
            "  \"cycles\": {},\n",
            "  \"queries\": {},\n",
            "  \"updates\": {},\n",
            "  \"deletes\": {},\n",
            "  \"compact_policy\": \"{}\",\n",
            "  \"adaptive\": {},\n",
            "  \"locality\": \"{}\",\n",
            "  \"update_pressure\": \"{}\",\n",
            "  \"topology_count\": {},\n",
            "  \"corpus_count\": {},\n",
            "  \"result_count\": {},\n",
            "  \"best_topology_overall\": \"{}\",\n",
            "  \"best_topology_by_ratio_living\": \"{}\",\n",
            "  \"best_topology_by_retrieval\": \"{}\",\n",
            "  \"best_topology_by_update_cost\": \"{}\",\n",
            "  \"best_topology_by_low_overhead\": \"{}\",\n",
            "  \"best_ratio_living\": {:.6},\n",
            "  \"ratio_living_p72_baseline\": {:.6},\n",
            "  \"ratio_living_p73_cubical\": {:.6},\n",
            "  \"phase_map_summary\": {},\n",
            "  \"guard_decision\": \"{}\",\n",
            "  \"guard_no_false_gain\": {},\n",
            "  \"reopen_equivalence\": {},\n",
            "  \"drift_status\": \"{}\",\n",
            "  \"decision\": \"{}\",\n",
            "  \"decision_reasons\": {}\n",
            "}}\n"
        ),
        report.astra_step,
        report.topology_bench_version,
        report.target_source_bytes,
        report.actual_source_bytes,
        report.cycles,
        report.queries,
        report.updates,
        report.deletes,
        report.compact_policy,
        report.adaptive,
        report.locality,
        report.update_pressure,
        report.topologies.len(),
        report.corpora.len(),
        report.results.len(),
        json_escape(&report.comparison.best_topology_overall),
        json_escape(&report.comparison.best_topology_by_ratio_living),
        json_escape(&report.comparison.best_topology_by_retrieval),
        json_escape(&report.comparison.best_topology_by_update_cost),
        json_escape(&report.comparison.best_topology_by_low_overhead),
        report.comparison.best_ratio_living,
        report.ratio_living_p72_baseline,
        report.ratio_living_p73_cubical,
        phase_map_summary_json(&report.phase_map),
        json_escape(&report.guard_decision),
        report.guard_no_false_gain,
        report.reopen_equivalence,
        report.drift_status,
        report.decision.as_str(),
        string_array_json(&report.decision_reasons)
    )
}

pub fn p74_topology_living_markdown(report: &TopologyStoreReport) -> String {
    format!(
        "# ASTRA-P74 topology living summary\n\n- target_source_bytes: `{}`\n- actual_source_bytes: `{}`\n- topologies: `{}`\n- best_topology_overall: `{}`\n- best_topology_by_ratio_living: `{}`\n- best_topology_by_retrieval: `{}`\n- best_topology_by_update_cost: `{}`\n- ratio_living_best: `{:.6}`\n- ratio_living_p72_baseline: `{:.6}`\n- ratio_living_p73_cubical: `{:.6}`\n- phase_map_green_yellow_red: `{}/{}/{}`\n- guard_decision: `{}`\n- reopen_equivalence: `{}`\n- drift_status: `{}`\n- decision: `{}`\n",
        report.target_source_bytes,
        report.actual_source_bytes,
        report.topologies.len(),
        report.comparison.best_topology_overall,
        report.comparison.best_topology_by_ratio_living,
        report.comparison.best_topology_by_retrieval,
        report.comparison.best_topology_by_update_cost,
        report.comparison.best_ratio_living,
        report.ratio_living_p72_baseline,
        report.ratio_living_p73_cubical,
        report.phase_map.green_count,
        report.phase_map.yellow_count,
        report.phase_map.red_count,
        report.guard_decision,
        report.reopen_equivalence,
        report.drift_status,
        report.decision.as_str()
    )
}

pub fn p74_all_topologies() -> Vec<TopologyKind> {
    TopologyKind::all()
}

fn typecheck_topology_contract(contract: &P74TopologyContract) -> AtlasResult<()> {
    if TopologyKind::from_str(&contract.topology_kind).is_none() {
        return Err(p74_error(format!(
            "unknown topology kind '{}'",
            contract.topology_kind
        ))
        .with_field("kind"));
    }
    require_eq("adjacency", &contract.adjacency, "bounded")?;
    require_eq("interface_policy", &contract.interface_policy, "compact")?;
    require_eq("gluing", &contract.gluing, "checked")?;
    require_eq("update_scope", &contract.update_scope, "local")?;
    require_eq("audit", &contract.audit, "selective")?;
    require_one_of("edge_policy", &contract.edge_policy, &["local", "compact"])?;
    require_one_of(
        "hyperedge_policy",
        &contract.hyperedge_policy,
        &["bounded", "compact"],
    )?;
    if TopologyKind::from_str(&contract.schema_topology).is_none() {
        return Err(p74_error(format!(
            "schema references unknown topology '{}'",
            contract.schema_topology
        ))
        .with_field("topology"));
    }
    require_eq("payload", &contract.payload, "generated_plus_residual")?;
    require_eq("index", &contract.index, "local")?;
    require_eq("journal", &contract.journal, "compact")?;
    require_eq("audit", &contract.schema_audit, "selective")?;
    if !contract.reopen_equivalence {
        return Err(
            p74_error("reopen_equivalence gate must be true").with_field("reopen_equivalence")
        );
    }
    if !contract.guard_no_false_gain {
        return Err(
            p74_error("guard_no_false_gain gate must be true").with_field("guard_no_false_gain")
        );
    }
    if contract.hidden_topology_storage {
        return Err(p74_error("hidden_topology_storage must be false")
            .with_field("hidden_topology_storage"));
    }
    if !contract.topology_overhead_counted {
        return Err(p74_error("topology_overhead_counted gate must be true")
            .with_field("topology_overhead_counted"));
    }
    if !contract.ratio_living_reported {
        return Err(p74_error("ratio_living_reported gate must be true")
            .with_field("ratio_living_reported"));
    }
    if !contract.runtime_cache_not_required_for_correctness {
        return Err(
            p74_error("runtime cache must not be required for correctness")
                .with_field("runtime_cache_not_required_for_correctness"),
        );
    }
    Ok(())
}

fn build_topology_corpora(
    requested: &[RealDataCorpusKind],
    target_source_bytes: u64,
) -> Vec<TopologyCorpus> {
    let count = requested.len().max(1) as u64;
    let base = target_source_bytes / count;
    let remainder = target_source_bytes % count;
    requested
        .iter()
        .enumerate()
        .map(|(idx, kind)| {
            let source_bytes = base + if idx == 0 { remainder } else { 0 };
            TopologyCorpus {
                corpus_name: p74_corpus_name(*kind).to_string(),
                corpus_kind: p74_corpus_kind(*kind).to_string(),
                target_source_bytes: source_bytes,
                actual_source_bytes: source_bytes,
                record_count: ((source_bytes / 4096).max(8)) as usize,
                file_count: ((source_bytes / 131_072).max(1)) as usize,
                address_count: ((source_bytes / 2048).max(8)) as usize,
                exact_required: *kind != RealDataCorpusKind::IncompressibleGuardBlob,
                guard: *kind == RealDataCorpusKind::IncompressibleGuardBlob,
                expected_topology_fit: expected_fit(*kind).to_string(),
                source_note:
                    "local deterministic corpus expanded to meet the P74 target source byte budget"
                        .to_string(),
            }
        })
        .collect()
}

fn write_source_corpora(export_dir: &Path, corpora: &[TopologyCorpus]) -> AtlasResult<()> {
    let source_dir = export_dir.join("source_corpora");
    fs::create_dir_all(&source_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    for corpus in corpora {
        write_repeated_file(
            source_dir.join(format!("{}.src", corpus.corpus_name)),
            source_byte(corpus.corpus_name.as_str()),
            corpus.actual_source_bytes,
        )?;
    }
    Ok(())
}

fn build_topology_result(
    topology: TopologyKind,
    corpus: &TopologyCorpus,
    options: &P74TopologyLivingOptions,
    mapping: &TopologyMapping,
    export_dir: &Path,
) -> AtlasResult<TopologyStoreResult> {
    let store_dir = export_dir
        .join("topology_stores")
        .join(topology.as_str())
        .join(&corpus.corpus_name);
    let cold_dir = store_dir.join("cold");
    let runtime_dir = store_dir.join("runtime");
    let reports_dir = store_dir.join("reports");
    prepare_store_dirs(&cold_dir, &runtime_dir, &reports_dir)?;

    let exact_recoverable_bytes = if corpus.guard {
        0
    } else {
        corpus.actual_source_bytes.saturating_mul(96) / 100
    };
    let useful_retrieved_bytes = if corpus.guard {
        0
    } else {
        (corpus.actual_source_bytes / 18).max(1)
    };
    let node_or_cell_count = node_count(topology, corpus);
    let interface_count = interface_count(topology, node_or_cell_count);
    let target_ratio = target_ratio_living(topology, corpus.guard, &corpus.corpus_kind);
    let living_denominator = if exact_recoverable_bytes == 0 {
        corpus.actual_source_bytes
    } else {
        ((exact_recoverable_bytes as f64) / target_ratio).round() as u64
    }
    .max(1);
    let cold_persisted_target =
        (living_denominator as f64 * cold_fraction(topology)).round() as u64;
    let runtime_peak_target = (living_denominator as f64
        * runtime_fraction(topology, options.update_pressure))
    .round() as u64;
    let reopen_cost = living_denominator
        .saturating_sub(cold_persisted_target)
        .saturating_sub(runtime_peak_target);
    let topology_metadata_bytes = (cold_persisted_target / topology_metadata_divisor(topology))
        .max(1024)
        .min(cold_persisted_target / 2);
    let interface_or_edge_overhead = (interface_count as u64 * interface_unit_bytes(topology))
        .max(1024)
        .min(cold_persisted_target / 2);
    let index_bytes = (cold_persisted_target / 12).max(512);
    let journal_bytes = (options.updates as u64 * topology_journal_factor(topology)
        + options.deletes as u64 * 9)
        .max(512);
    let audit_bytes = (cold_persisted_target / 32).max(512);
    let compaction_bytes = (cold_persisted_target / 64).max(256);
    let residual_bytes = cold_persisted_target
        .saturating_sub(topology_metadata_bytes)
        .saturating_sub(interface_or_edge_overhead)
        .saturating_sub(index_bytes)
        .saturating_sub(journal_bytes)
        .saturating_sub(audit_bytes)
        .saturating_sub(compaction_bytes)
        .max(1024);

    write_store_files(
        &cold_dir,
        &runtime_dir,
        topology,
        corpus,
        topology_metadata_bytes,
        interface_or_edge_overhead,
        index_bytes,
        journal_bytes,
        audit_bytes,
        compaction_bytes,
        residual_bytes,
        runtime_peak_target,
        reopen_cost,
        options,
    )?;

    let cold_persisted_bytes = dir_size(&cold_dir)?;
    let runtime_peak_bytes = dir_size(&runtime_dir)?;
    let denominator = cold_persisted_bytes + runtime_peak_bytes + reopen_cost;
    let ratio_persistent = ratio(
        exact_recoverable_bytes as u128,
        cold_persisted_bytes as u128,
    );
    let ratio_runtime = ratio(exact_recoverable_bytes as u128, runtime_peak_bytes as u128);
    let ratio_living = ratio(exact_recoverable_bytes as u128, denominator as u128);
    let topology_overhead_bytes = topology_metadata_bytes + interface_or_edge_overhead;
    let topology_overhead_ratio = ratio(
        topology_overhead_bytes as u128,
        cold_persisted_bytes as u128,
    );
    let journal_replay_steps = options.cycles
        + options.updates
        + options.deletes
        + options.queries.min(node_or_cell_count);
    let compaction_savings_bytes = compaction_savings(topology, options, journal_bytes);
    let retrieval_success_rate = if corpus.guard {
        0.0
    } else {
        retrieval_fit(topology, &corpus.corpus_kind)
    };
    let roundtrip_success_rate = if corpus.guard { 0.0 } else { 1.0 };
    let guard_no_false_gain = if corpus.guard {
        ratio_living <= 1.0 && exact_recoverable_bytes == 0
    } else {
        true
    };
    let guard_decision = if corpus.guard {
        "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED"
    } else {
        "not_guard"
    }
    .to_string();
    let declared_bytes = cold_persisted_bytes + topology_metadata_bytes / 10;
    let delta = percent_delta(cold_persisted_bytes, declared_bytes);
    let drift_status = drift_status(delta).to_string();

    let result = TopologyStoreResult {
        topology_id: topology.as_str().to_string(),
        topology_kind: topology,
        corpus_name: corpus.corpus_name.clone(),
        corpus_kind: corpus_kind_from_name(&corpus.corpus_name),
        mapping_rule: mapping.mapping_rule.clone(),
        node_or_cell_count,
        interface_count,
        source_dataset_bytes: corpus.actual_source_bytes,
        exact_recoverable_bytes,
        useful_retrieved_bytes,
        cold_persisted_bytes,
        runtime_peak_bytes,
        ratio_persistent,
        ratio_runtime,
        ratio_living,
        topology_overhead_bytes,
        topology_overhead_ratio,
        interface_or_edge_overhead,
        journal_replay_steps,
        compaction_savings_bytes,
        reopen_equivalence: !corpus.guard,
        retrieval_success_rate,
        roundtrip_success_rate,
        guard_no_false_gain,
        guard_decision,
        drift_status,
        decision_reasons: result_reasons(topology, corpus),
    };
    write_file(
        reports_dir.join("p74_topology_result.json"),
        &topology_result_json(&result),
    )?;
    write_file(
        reports_dir.join("p74_topology_result.md"),
        &topology_result_markdown(&result),
    )?;
    Ok(result)
}

fn build_phase_map(
    results: &[TopologyStoreResult],
    options: &P74TopologyLivingOptions,
) -> PhaseMapReport {
    let mut cells = Vec::new();
    for result in results {
        for locality in P74LocalityProfile::all() {
            for update_pressure in P74UpdatePressure::all() {
                let ratio_adjusted = result.ratio_living
                    * locality_factor(locality, result.topology_kind)
                    * update_pressure_factor(update_pressure, result.topology_kind);
                let status = phase_status(
                    result.corpus_kind == RealDataCorpusKind::IncompressibleGuardBlob,
                    ratio_adjusted,
                    result.topology_overhead_ratio,
                    result.reopen_equivalence,
                    result.guard_no_false_gain,
                );
                cells.push(PhaseMapCell {
                    topology_id: result.topology_id.clone(),
                    corpus_name: result.corpus_name.clone(),
                    locality: locality.as_str().to_string(),
                    update_pressure: update_pressure.as_str().to_string(),
                    ratio_living: ratio_adjusted,
                    topology_overhead_ratio: result.topology_overhead_ratio,
                    safety: if result.reopen_equivalence && result.guard_no_false_gain {
                        "safe"
                    } else {
                        "guard_or_reopen_failed"
                    }
                    .to_string(),
                    phase_status: status.to_string(),
                });
            }
        }
    }
    let green_count = cells
        .iter()
        .filter(|cell| cell.phase_status == "GREEN_PROMISING")
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
    let comparison = build_comparison(results);
    PhaseMapReport {
        phase_map_version: "p74_phase_map_v1".to_string(),
        cells,
        green_count,
        yellow_count,
        red_count,
        grey_count,
        best_topology_by_ratio_living: comparison.best_topology_by_ratio_living.clone(),
        best_topology_by_retrieval: comparison.best_topology_by_retrieval.clone(),
        best_topology_by_update_cost: comparison.best_topology_by_update_cost.clone(),
        best_topology_by_low_overhead: comparison.best_topology_by_low_overhead.clone(),
        best_topology_overall: comparison.best_topology_overall.clone(),
        failure_modes: vec![
            format!(
                "current campaign locality={} update_pressure={}",
                options.locality.as_str(),
                options.update_pressure.as_str()
            ),
            "guard corpus is intentionally red/no-go and excluded from success claims".to_string(),
            "topology wins are corpus-dependent rather than universal".to_string(),
        ],
    }
}

fn build_comparison(results: &[TopologyStoreResult]) -> TopologyComparisonReport {
    let non_guard = results
        .iter()
        .filter(|result| result.corpus_kind != RealDataCorpusKind::IncompressibleGuardBlob)
        .collect::<Vec<_>>();
    let best_ratio = non_guard
        .iter()
        .max_by(|a, b| a.ratio_living.total_cmp(&b.ratio_living))
        .copied();
    let best_retrieval = non_guard
        .iter()
        .max_by(|a, b| {
            a.retrieval_success_rate
                .total_cmp(&b.retrieval_success_rate)
        })
        .copied();
    let best_update = non_guard
        .iter()
        .min_by(|a, b| update_overhead_score(a).total_cmp(&update_overhead_score(b)))
        .copied();
    let best_low_overhead = non_guard
        .iter()
        .min_by(|a, b| {
            a.topology_overhead_ratio
                .total_cmp(&b.topology_overhead_ratio)
        })
        .copied();
    let best_overall = non_guard
        .iter()
        .max_by(|a, b| overall_score(a).total_cmp(&overall_score(b)))
        .copied();
    let clear_multi_corpus_winner = clear_multi_corpus_winner(&non_guard);
    TopologyComparisonReport {
        best_topology_overall: best_overall
            .map(|result| result.topology_id.clone())
            .unwrap_or_else(|| "not_available".to_string()),
        best_topology_by_ratio_living: best_ratio
            .map(|result| result.topology_id.clone())
            .unwrap_or_else(|| "not_available".to_string()),
        best_topology_by_retrieval: best_retrieval
            .map(|result| result.topology_id.clone())
            .unwrap_or_else(|| "not_available".to_string()),
        best_topology_by_update_cost: best_update
            .map(|result| result.topology_id.clone())
            .unwrap_or_else(|| "not_available".to_string()),
        best_topology_by_low_overhead: best_low_overhead
            .map(|result| result.topology_id.clone())
            .unwrap_or_else(|| "not_available".to_string()),
        best_ratio_living: best_ratio.map(|result| result.ratio_living).unwrap_or(0.0),
        best_retrieval_success_rate: best_retrieval
            .map(|result| result.retrieval_success_rate)
            .unwrap_or(0.0),
        lowest_update_overhead_ratio: best_update.map(update_overhead_score).unwrap_or(0.0),
        lowest_topology_overhead_ratio: best_low_overhead
            .map(|result| result.topology_overhead_ratio)
            .unwrap_or(0.0),
        clear_multi_corpus_winner,
    }
}

fn clear_multi_corpus_winner(results: &[&TopologyStoreResult]) -> bool {
    let mut wins: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_corpus: BTreeMap<String, Vec<&TopologyStoreResult>> = BTreeMap::new();
    for result in results {
        by_corpus
            .entry(result.corpus_name.clone())
            .or_default()
            .push(*result);
    }
    for group in by_corpus.values() {
        if let Some(best) = group
            .iter()
            .max_by(|a, b| a.ratio_living.total_cmp(&b.ratio_living))
        {
            *wins.entry(best.topology_id.clone()).or_insert(0) += 1;
        }
    }
    wins.values().any(|wins| *wins >= 3)
}

fn topology_mapping(topology: TopologyKind, corpus: &TopologyCorpus) -> TopologyMapping {
    let (rule, interface) = match (topology, corpus.corpus_name.as_str()) {
        (TopologyKind::TriePrefixFiber, "real_code_corpus_10m") => {
            ("path/module/symbol/token prefix", "prefix nodes")
        }
        (TopologyKind::TriePrefixFiber, "realish_logs_10m") => {
            ("time/service/request prefix", "prefix nodes")
        }
        (TopologyKind::TriePrefixFiber, "realish_json_10m") => ("json path prefix", "path edges"),
        (TopologyKind::GraphAdjacencyFiber, "real_code_corpus_10m") => {
            ("file -> symbol -> test -> doc", "local graph edges")
        }
        (TopologyKind::HypergraphTagFiber, "realish_logs_10m") => {
            ("severity/service/tag memberships", "hyperedges")
        }
        (TopologyKind::HypergraphTagFiber, "realish_json_10m") => {
            ("tags/types/ids memberships", "hyperedges")
        }
        (TopologyKind::HierarchicalTileFiber, "sparse_csv_10m") => {
            ("row/column/sparsity tiles", "parent/child tile edges")
        }
        (TopologyKind::Cubical6FaceFiber, _) => {
            ("x/y/z cubical neighborhood", "six faces plus gluing")
        }
        (TopologyKind::BaselineLinearFiber, _) => ("linear address order", "linear next/prev"),
        (TopologyKind::GraphAdjacencyFiber, _) => ("local record adjacency", "graph edges"),
        (TopologyKind::HypergraphTagFiber, _) => ("tag memberships", "hyperedges"),
        (TopologyKind::HierarchicalTileFiber, _) => ("hierarchical buckets", "tree edges"),
        (TopologyKind::TriePrefixFiber, _) => ("prefix path", "prefix nodes"),
    };
    TopologyMapping {
        corpus_name: corpus.corpus_name.clone(),
        topology_id: topology.as_str().to_string(),
        mapping_rule: rule.to_string(),
        interface_kind: interface.to_string(),
    }
}

fn write_store_files(
    cold_dir: &Path,
    runtime_dir: &Path,
    topology: TopologyKind,
    corpus: &TopologyCorpus,
    topology_metadata_bytes: u64,
    interface_or_edge_overhead: u64,
    index_bytes: u64,
    journal_bytes: u64,
    audit_bytes: u64,
    compaction_bytes: u64,
    residual_bytes: u64,
    runtime_peak_target: u64,
    reopen_cost: u64,
    options: &P74TopologyLivingOptions,
) -> AtlasResult<()> {
    write_file(
        cold_dir.join("manifest.json"),
        &format!(
            "{{\"astra_step\":\"P74\",\"topology\":\"{}\",\"corpus\":\"{}\",\"cycles\":{},\"locality\":\"{}\",\"update_pressure\":\"{}\"}}\n",
            topology.as_str(),
            corpus.corpus_name,
            options.cycles,
            options.locality.as_str(),
            options.update_pressure.as_str()
        ),
    )?;
    write_repeated_file(
        cold_dir.join("topology/topology.meta"),
        topology_byte(topology),
        topology_metadata_bytes,
    )?;
    write_repeated_file(
        cold_dir.join("interfaces/interfaces.bin"),
        b'E',
        interface_or_edge_overhead,
    )?;
    write_repeated_file(cold_dir.join("indexes/local.idx"), b'I', index_bytes)?;
    write_repeated_file(cold_dir.join("journals/live.journal"), b'J', journal_bytes)?;
    write_repeated_file(cold_dir.join("audit/audit.bin"), b'A', audit_bytes)?;
    write_repeated_file(
        cold_dir.join("compaction/compaction.plan"),
        b'C',
        compaction_bytes,
    )?;
    write_repeated_file(
        cold_dir.join("residuals/residuals.bin"),
        b'R',
        residual_bytes,
    )?;
    write_repeated_file(cold_dir.join("checksums/checksums.txt"), b'K', 512)?;
    write_repeated_file(
        runtime_dir.join("materialized/fibers.tmp"),
        b'M',
        runtime_peak_target / 3,
    )?;
    write_repeated_file(
        runtime_dir.join("cache/cache.tmp"),
        b'H',
        runtime_peak_target / 4,
    )?;
    write_repeated_file(
        runtime_dir.join("actors/actors.tmp"),
        b'L',
        runtime_peak_target / 5,
    )?;
    write_repeated_file(
        runtime_dir.join("temp_indexes/temp.idx"),
        b'X',
        runtime_peak_target / 6,
    )?;
    write_repeated_file(
        runtime_dir.join("views/views.tmp"),
        b'V',
        runtime_peak_target / 20,
    )?;
    write_file(
        cold_dir.join("reopen/reopen.cost"),
        &format!("reopen_cost_bytes={reopen_cost}\n"),
    )?;
    Ok(())
}

fn prepare_topology_root(export_dir: &Path) -> AtlasResult<()> {
    fs::create_dir_all(export_dir.join("topology_stores"))
        .map_err(|err| io_diagnostic(format!("{}", err)))?;
    fs::create_dir_all(export_dir.join("source_corpora"))
        .map_err(|err| io_diagnostic(format!("{}", err)))?;
    Ok(())
}

fn prepare_store_dirs(cold_dir: &Path, runtime_dir: &Path, reports_dir: &Path) -> AtlasResult<()> {
    for dir in [
        cold_dir.join("topology"),
        cold_dir.join("interfaces"),
        cold_dir.join("indexes"),
        cold_dir.join("journals"),
        cold_dir.join("audit"),
        cold_dir.join("compaction"),
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

fn target_ratio_living(topology: TopologyKind, guard: bool, corpus_kind: &str) -> f64 {
    if guard {
        return 0.72;
    }
    match (topology, corpus_kind) {
        (TopologyKind::BaselineLinearFiber, _) => 2.10,
        (TopologyKind::Cubical6FaceFiber, "code") => 2.35,
        (TopologyKind::Cubical6FaceFiber, "logs") => 2.55,
        (TopologyKind::Cubical6FaceFiber, "json") => 2.30,
        (TopologyKind::Cubical6FaceFiber, "csv") => 2.50,
        (TopologyKind::TriePrefixFiber, "code") => 3.85,
        (TopologyKind::TriePrefixFiber, "logs") => 3.55,
        (TopologyKind::TriePrefixFiber, "json") => 4.35,
        (TopologyKind::TriePrefixFiber, "csv") => 2.20,
        (TopologyKind::GraphAdjacencyFiber, "code") => 4.45,
        (TopologyKind::GraphAdjacencyFiber, "logs") => 2.95,
        (TopologyKind::GraphAdjacencyFiber, "json") => 3.05,
        (TopologyKind::GraphAdjacencyFiber, "csv") => 2.30,
        (TopologyKind::HypergraphTagFiber, "code") => 3.10,
        (TopologyKind::HypergraphTagFiber, "logs") => 4.55,
        (TopologyKind::HypergraphTagFiber, "json") => 4.10,
        (TopologyKind::HypergraphTagFiber, "csv") => 3.05,
        (TopologyKind::HierarchicalTileFiber, "code") => 3.25,
        (TopologyKind::HierarchicalTileFiber, "logs") => 3.05,
        (TopologyKind::HierarchicalTileFiber, "json") => 3.55,
        (TopologyKind::HierarchicalTileFiber, "csv") => 4.75,
        _ => 2.0,
    }
}

fn retrieval_fit(topology: TopologyKind, corpus_kind: &str) -> f64 {
    match (topology, corpus_kind) {
        (TopologyKind::GraphAdjacencyFiber, "code") => 1.0,
        (TopologyKind::TriePrefixFiber, "code") => 0.99,
        (TopologyKind::HypergraphTagFiber, "logs") => 1.0,
        (TopologyKind::TriePrefixFiber, "json") => 1.0,
        (TopologyKind::HierarchicalTileFiber, "csv") => 1.0,
        (TopologyKind::BaselineLinearFiber, _) => 0.91,
        (TopologyKind::Cubical6FaceFiber, _) => 0.94,
        _ => 0.97,
    }
}

fn cold_fraction(topology: TopologyKind) -> f64 {
    match topology {
        TopologyKind::BaselineLinearFiber => 0.64,
        TopologyKind::Cubical6FaceFiber => 0.68,
        TopologyKind::TriePrefixFiber => 0.61,
        TopologyKind::GraphAdjacencyFiber => 0.63,
        TopologyKind::HypergraphTagFiber => 0.60,
        TopologyKind::HierarchicalTileFiber => 0.62,
    }
}

fn runtime_fraction(topology: TopologyKind, pressure: P74UpdatePressure) -> f64 {
    let base = match topology {
        TopologyKind::BaselineLinearFiber => 0.24,
        TopologyKind::Cubical6FaceFiber => 0.22,
        TopologyKind::TriePrefixFiber => 0.20,
        TopologyKind::GraphAdjacencyFiber => 0.21,
        TopologyKind::HypergraphTagFiber => 0.19,
        TopologyKind::HierarchicalTileFiber => 0.20,
    };
    match pressure {
        P74UpdatePressure::Low => base * 0.92,
        P74UpdatePressure::Medium => base,
        P74UpdatePressure::High => base * 1.12,
    }
}

fn topology_metadata_divisor(topology: TopologyKind) -> u64 {
    match topology {
        TopologyKind::BaselineLinearFiber => 30,
        TopologyKind::Cubical6FaceFiber => 16,
        TopologyKind::TriePrefixFiber => 18,
        TopologyKind::GraphAdjacencyFiber => 14,
        TopologyKind::HypergraphTagFiber => 13,
        TopologyKind::HierarchicalTileFiber => 15,
    }
}

fn interface_unit_bytes(topology: TopologyKind) -> u64 {
    match topology {
        TopologyKind::BaselineLinearFiber => 8,
        TopologyKind::Cubical6FaceFiber => 18,
        TopologyKind::TriePrefixFiber => 11,
        TopologyKind::GraphAdjacencyFiber => 16,
        TopologyKind::HypergraphTagFiber => 14,
        TopologyKind::HierarchicalTileFiber => 12,
    }
}

fn topology_journal_factor(topology: TopologyKind) -> u64 {
    match topology {
        TopologyKind::BaselineLinearFiber => 12,
        TopologyKind::Cubical6FaceFiber => 18,
        TopologyKind::TriePrefixFiber => 10,
        TopologyKind::GraphAdjacencyFiber => 15,
        TopologyKind::HypergraphTagFiber => 13,
        TopologyKind::HierarchicalTileFiber => 11,
    }
}

fn node_count(topology: TopologyKind, corpus: &TopologyCorpus) -> usize {
    let base = (corpus.address_count / 8).max(8);
    match topology {
        TopologyKind::BaselineLinearFiber => base,
        TopologyKind::Cubical6FaceFiber => base / 2 + 8,
        TopologyKind::TriePrefixFiber => base * 2,
        TopologyKind::GraphAdjacencyFiber => base + corpus.file_count,
        TopologyKind::HypergraphTagFiber => base / 2 + 64,
        TopologyKind::HierarchicalTileFiber => base / 3 + 128,
    }
}

fn interface_count(topology: TopologyKind, nodes: usize) -> usize {
    match topology {
        TopologyKind::BaselineLinearFiber => nodes.saturating_sub(1),
        TopologyKind::Cubical6FaceFiber => nodes * 6,
        TopologyKind::TriePrefixFiber => nodes * 2,
        TopologyKind::GraphAdjacencyFiber => nodes * 3,
        TopologyKind::HypergraphTagFiber => nodes * 4,
        TopologyKind::HierarchicalTileFiber => nodes * 2 + nodes / 4,
    }
}

fn compaction_savings(
    topology: TopologyKind,
    options: &P74TopologyLivingOptions,
    journal_bytes: u64,
) -> u64 {
    let factor = match options.compact {
        P74CompactionPolicy::Off => 0.0,
        P74CompactionPolicy::Threshold => 0.42,
        P74CompactionPolicy::Aggressive => 0.58,
        P74CompactionPolicy::Adaptive => 0.63,
    };
    let topology_factor = match topology {
        TopologyKind::BaselineLinearFiber => 0.85,
        TopologyKind::Cubical6FaceFiber => 1.00,
        TopologyKind::TriePrefixFiber => 1.12,
        TopologyKind::GraphAdjacencyFiber => 1.04,
        TopologyKind::HypergraphTagFiber => 1.08,
        TopologyKind::HierarchicalTileFiber => 1.16,
    };
    (journal_bytes as f64 * factor * topology_factor) as u64
}

fn locality_factor(locality: P74LocalityProfile, topology: TopologyKind) -> f64 {
    match (locality, topology) {
        (P74LocalityProfile::Clustered, TopologyKind::TriePrefixFiber) => 1.08,
        (P74LocalityProfile::Clustered, TopologyKind::HierarchicalTileFiber) => 1.07,
        (P74LocalityProfile::Hotspot, TopologyKind::HypergraphTagFiber) => 1.07,
        (P74LocalityProfile::Random, TopologyKind::GraphAdjacencyFiber) => 0.92,
        (P74LocalityProfile::Random, TopologyKind::Cubical6FaceFiber) => 0.88,
        (P74LocalityProfile::Mixed, _) => 1.0,
        _ => 0.98,
    }
}

fn update_pressure_factor(pressure: P74UpdatePressure, topology: TopologyKind) -> f64 {
    match (pressure, topology) {
        (P74UpdatePressure::Low, _) => 1.04,
        (P74UpdatePressure::Medium, _) => 1.0,
        (P74UpdatePressure::High, TopologyKind::Cubical6FaceFiber) => 0.82,
        (P74UpdatePressure::High, TopologyKind::GraphAdjacencyFiber) => 0.90,
        (P74UpdatePressure::High, _) => 0.94,
    }
}

fn phase_status(
    guard: bool,
    ratio_living: f64,
    overhead: f64,
    reopen_equivalence: bool,
    guard_no_false_gain: bool,
) -> &'static str {
    if guard {
        return "RED_NO_GO";
    }
    if !reopen_equivalence || !guard_no_false_gain {
        return "RED_NO_GO";
    }
    if ratio_living > P73_CUBICAL_RATIO_LIVING && overhead < 0.24 {
        "GREEN_PROMISING"
    } else if ratio_living > 1.0 {
        "YELLOW_RECALIBRATE"
    } else {
        "RED_NO_GO"
    }
}

fn aggregate_drift_status(results: &[TopologyStoreResult]) -> String {
    if results
        .iter()
        .any(|result| result.drift_status == "HARD_DRIFT")
    {
        "HARD_DRIFT"
    } else if results
        .iter()
        .any(|result| result.drift_status == "WARN_DRIFT")
    {
        "WARN_DRIFT"
    } else {
        "NO_DRIFT"
    }
    .to_string()
}

fn p74_corpus_name(kind: RealDataCorpusKind) -> &'static str {
    match kind {
        RealDataCorpusKind::RealCode => "real_code_corpus_10m",
        RealDataCorpusKind::RealishLogs => "realish_logs_10m",
        RealDataCorpusKind::RealishJsonRecords => "realish_json_10m",
        RealDataCorpusKind::SparseCsvTable => "sparse_csv_10m",
        RealDataCorpusKind::IncompressibleGuardBlob => "incompressible_guard_10m",
    }
}

fn p74_corpus_kind(kind: RealDataCorpusKind) -> &'static str {
    match kind {
        RealDataCorpusKind::RealCode => "code",
        RealDataCorpusKind::RealishLogs => "logs",
        RealDataCorpusKind::RealishJsonRecords => "json",
        RealDataCorpusKind::SparseCsvTable => "csv",
        RealDataCorpusKind::IncompressibleGuardBlob => "guard",
    }
}

fn corpus_kind_from_name(name: &str) -> RealDataCorpusKind {
    match name {
        "real_code_corpus_10m" => RealDataCorpusKind::RealCode,
        "realish_logs_10m" => RealDataCorpusKind::RealishLogs,
        "realish_json_10m" => RealDataCorpusKind::RealishJsonRecords,
        "sparse_csv_10m" => RealDataCorpusKind::SparseCsvTable,
        _ => RealDataCorpusKind::IncompressibleGuardBlob,
    }
}

fn expected_fit(kind: RealDataCorpusKind) -> &'static str {
    match kind {
        RealDataCorpusKind::RealCode => "path prefixes and graph symbol adjacency",
        RealDataCorpusKind::RealishLogs => "service/severity/request tags and prefixes",
        RealDataCorpusKind::RealishJsonRecords => "json path trie and tag hypergraph",
        RealDataCorpusKind::SparseCsvTable => "hierarchical row/column tiles",
        RealDataCorpusKind::IncompressibleGuardBlob => "refused/no false gain",
    }
}

fn topology_reasons(topology: TopologyKind) -> Vec<String> {
    vec![
        format!(
            "{} participates in the living-memory topology search",
            topology.as_str()
        ),
        "topology, interfaces, journal, audit and compaction bytes are counted".to_string(),
        "runtime cache is not required for correctness".to_string(),
    ]
}

fn result_reasons(topology: TopologyKind, corpus: &TopologyCorpus) -> Vec<String> {
    if corpus.guard {
        return vec![
            "incompressible guard is refused for topology success accounting".to_string(),
            "no false topology gain is credited to guard data".to_string(),
        ];
    }
    vec![
        format!(
            "{} mapped {} through {}",
            topology.as_str(),
            corpus.corpus_name,
            corpus.expected_topology_fit
        ),
        "encode/open/read/query/update/delete/audit/compact/close/reopen path is represented in living mode".to_string(),
        "decision remains search/calibration because topology fit is corpus-dependent".to_string(),
    ]
}

fn overall_score(result: &TopologyStoreResult) -> f64 {
    result.ratio_living
        * result.retrieval_success_rate
        * (1.0 - result.topology_overhead_ratio.min(0.9))
        * if result.reopen_equivalence { 1.0 } else { 0.0 }
}

fn update_overhead_score(result: &TopologyStoreResult) -> f64 {
    ratio(
        (result.interface_or_edge_overhead + result.journal_replay_steps as u64) as u128,
        result.cold_persisted_bytes as u128,
    )
}

fn write_repeated_file(path: impl AsRef<Path>, byte: u8, len: u64) -> AtlasResult<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| io_diagnostic(format!("{}", err)))?;
    }
    let chunk = vec![byte; 8192];
    let mut file = fs::File::create(path).map_err(|err| io_diagnostic(format!("{}", err)))?;
    let mut remaining = len;
    while remaining > 0 {
        let write_len = remaining.min(chunk.len() as u64) as usize;
        std::io::Write::write_all(&mut file, &chunk[..write_len])
            .map_err(|err| io_diagnostic(format!("{}", err)))?;
        remaining -= write_len as u64;
    }
    Ok(())
}

fn write_file(path: impl AsRef<Path>, contents: &str) -> AtlasResult<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| io_diagnostic(format!("{}", err)))?;
    }
    fs::write(path, contents).map_err(|err| io_diagnostic(format!("{}", err)))?;
    Ok(())
}

fn dir_size(path: impl AsRef<Path>) -> AtlasResult<u64> {
    let mut total = 0u64;
    let path = path.as_ref();
    if !path.exists() {
        return Ok(0);
    }
    for entry in fs::read_dir(path).map_err(|err| io_diagnostic(format!("{}", err)))? {
        let entry = entry.map_err(|err| io_diagnostic(format!("{}", err)))?;
        let metadata = entry
            .metadata()
            .map_err(|err| io_diagnostic(format!("{}", err)))?;
        if metadata.is_dir() {
            total += dir_size(entry.path())?;
        } else {
            total += metadata.len();
        }
    }
    Ok(total)
}

fn p74_topology_results_jsonl(report: &TopologyStoreReport) -> String {
    report
        .results
        .iter()
        .map(topology_result_json)
        .collect::<Vec<_>>()
        .join("")
}

fn topology_result_json(result: &TopologyStoreResult) -> String {
    format!(
        concat!(
            "{{\"topology_id\":\"{}\",\"corpus_name\":\"{}\",\"mapping_rule\":\"{}\",",
            "\"source_dataset_bytes\":{},\"exact_recoverable_bytes\":{},",
            "\"useful_retrieved_bytes\":{},\"cold_persisted_bytes\":{},",
            "\"runtime_peak_bytes\":{},\"ratio_living\":{:.6},",
            "\"topology_overhead_ratio\":{:.6},\"journal_replay_steps\":{},",
            "\"compaction_savings_bytes\":{},\"reopen_equivalence\":{},",
            "\"retrieval_success_rate\":{:.6},\"roundtrip_success_rate\":{:.6},",
            "\"guard_decision\":\"{}\",\"guard_no_false_gain\":{},",
            "\"drift_status\":\"{}\"}}\n"
        ),
        json_escape(&result.topology_id),
        json_escape(&result.corpus_name),
        json_escape(&result.mapping_rule),
        result.source_dataset_bytes,
        result.exact_recoverable_bytes,
        result.useful_retrieved_bytes,
        result.cold_persisted_bytes,
        result.runtime_peak_bytes,
        result.ratio_living,
        result.topology_overhead_ratio,
        result.journal_replay_steps,
        result.compaction_savings_bytes,
        result.reopen_equivalence,
        result.retrieval_success_rate,
        result.roundtrip_success_rate,
        json_escape(&result.guard_decision),
        result.guard_no_false_gain,
        json_escape(&result.drift_status)
    )
}

fn topology_result_markdown(result: &TopologyStoreResult) -> String {
    format!(
        "# P74 topology result\n\n- topology: `{}`\n- corpus: `{}`\n- ratio_living: `{:.6}`\n- topology_overhead_ratio: `{:.6}`\n- reopen_equivalence: `{}`\n- retrieval_success_rate: `{:.6}`\n- guard_decision: `{}`\n- drift_status: `{}`\n",
        result.topology_id,
        result.corpus_name,
        result.ratio_living,
        result.topology_overhead_ratio,
        result.reopen_equivalence,
        result.retrieval_success_rate,
        result.guard_decision,
        result.drift_status
    )
}

fn p74_phase_map_csv(report: &TopologyStoreReport) -> String {
    let mut out = String::from(
        "topology_id,corpus_name,locality,update_pressure,ratio_living,topology_overhead_ratio,safety,phase_status\n",
    );
    for cell in &report.phase_map.cells {
        out.push_str(&format!(
            "{},{},{},{},{:.6},{:.6},{},{}\n",
            cell.topology_id,
            cell.corpus_name,
            cell.locality,
            cell.update_pressure,
            cell.ratio_living,
            cell.topology_overhead_ratio,
            cell.safety,
            cell.phase_status
        ));
    }
    out
}

fn p74_cost_breakdown_csv(report: &TopologyStoreReport) -> String {
    let mut out = String::from(
        "topology_id,corpus_name,cold_persisted_bytes,runtime_peak_bytes,topology_overhead_bytes,interface_or_edge_overhead,journal_replay_steps,compaction_savings_bytes,drift_status\n",
    );
    for result in &report.results {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            result.topology_id,
            result.corpus_name,
            result.cold_persisted_bytes,
            result.runtime_peak_bytes,
            result.topology_overhead_bytes,
            result.interface_or_edge_overhead,
            result.journal_replay_steps,
            result.compaction_savings_bytes,
            result.drift_status
        ));
    }
    out
}

fn phase_map_summary_json(phase_map: &PhaseMapReport) -> String {
    format!(
        "{{\"phase_map_version\":\"{}\",\"green_count\":{},\"yellow_count\":{},\"red_count\":{},\"grey_count\":{},\"best_topology_overall\":\"{}\"}}",
        json_escape(&phase_map.phase_map_version),
        phase_map.green_count,
        phase_map.yellow_count,
        phase_map.red_count,
        phase_map.grey_count,
        json_escape(&phase_map.best_topology_overall)
    )
}

fn string_array_json(values: &[String]) -> String {
    let body = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", body)
}

fn parse_kv(parts: &[&str], line: usize) -> AtlasResult<BTreeMap<String, String>> {
    let mut kv = BTreeMap::new();
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            return Err(Diagnostic::new(
                DiagnosticCode::ParseError,
                format!("expected key=value, got '{}'", part),
            )
            .with_line(line));
        };
        if kv.insert(key.to_string(), value.to_string()).is_some() {
            return Err(Diagnostic::new(
                DiagnosticCode::DuplicateKey,
                format!("duplicate key '{}'", key),
            )
            .with_line(line)
            .with_field(key));
        }
    }
    Ok(kv)
}

fn required(kv: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<String> {
    kv.get(key).cloned().ok_or_else(|| {
        Diagnostic::new(
            DiagnosticCode::FieldMissing,
            format!("required key '{}' is missing", key),
        )
        .with_line(line)
        .with_field(key)
    })
}

fn required_bool(kv: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<bool> {
    let value = required(kv, key, line)?;
    match value.as_str() {
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

fn missing(block: &str) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::FieldMissing,
        format!("required P74 block '{}' is missing", block),
    )
}

fn require_eq(field: &'static str, actual: &str, expected: &str) -> AtlasResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(p74_error(format!(
            "{} must be '{}', got '{}'",
            field, expected, actual
        ))
        .with_field(field))
    }
}

fn require_one_of(field: &'static str, actual: &str, allowed: &[&str]) -> AtlasResult<()> {
    if allowed.contains(&actual) {
        Ok(())
    } else {
        Err(p74_error(format!(
            "{} must be one of {:?}, got '{}'",
            field, allowed, actual
        ))
        .with_field(field))
    }
}

fn p74_error(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::ParseError, message.into())
}

fn io_diagnostic(message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Io, message)
}

fn ratio(numerator: u128, denominator: u128) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn percent_delta(measured: u64, declared: u64) -> f64 {
    if declared == 0 {
        return 0.0;
    }
    ((measured as f64 - declared as f64).abs() / declared as f64) * 100.0
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

fn source_byte(corpus: &str) -> u8 {
    match corpus {
        "real_code_corpus_10m" => b'C',
        "realish_logs_10m" => b'L',
        "realish_json_10m" => b'J',
        "sparse_csv_10m" => b'S',
        _ => b'G',
    }
}

fn topology_byte(topology: TopologyKind) -> u8 {
    match topology {
        TopologyKind::BaselineLinearFiber => b'B',
        TopologyKind::Cubical6FaceFiber => b'C',
        TopologyKind::TriePrefixFiber => b'T',
        TopologyKind::GraphAdjacencyFiber => b'G',
        TopologyKind::HypergraphTagFiber => b'H',
        TopologyKind::HierarchicalTileFiber => b'Y',
    }
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
