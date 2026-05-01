use astra_atlas_lang::{
    p71_all_corpora, p74_all_topologies, p74_parse_topology_file, p74_topology_living_bench,
    p74_topology_living_json, P74CompactionPolicy, P74LocalityProfile, P74TopologyLivingOptions,
    P74UpdatePressure, RealDataCorpusKind, TopologyDecision, TopologyKind,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID_TOPOLOGY: &str = "examples/valid/p74_living_topology_search.atlas";

#[test]
fn p74_topologies_exist() {
    let topologies = p74_all_topologies();

    assert_eq!(topologies.len(), 6);
    assert!(topologies.contains(&TopologyKind::BaselineLinearFiber));
    assert!(topologies.contains(&TopologyKind::Cubical6FaceFiber));
    assert!(topologies.contains(&TopologyKind::TriePrefixFiber));
    assert!(topologies.contains(&TopologyKind::GraphAdjacencyFiber));
    assert!(topologies.contains(&TopologyKind::HypergraphTagFiber));
    assert!(topologies.contains(&TopologyKind::HierarchicalTileFiber));
    assert_eq!(TopologyKind::from_str("unknown"), None);
}

#[test]
fn p74_topology_contract_parses_and_typechecks() {
    let contract = p74_parse_topology_file(VALID_TOPOLOGY).expect("P74 topology contract parses");

    assert_eq!(contract.topology_kind, "trie_prefix");
    assert_eq!(contract.adjacency, "bounded");
    assert_eq!(contract.interface_policy, "compact");
    assert_eq!(contract.update_scope, "local");
    assert!(contract.reopen_equivalence);
    assert!(contract.guard_no_false_gain);
    assert!(!contract.hidden_topology_storage);
    assert!(contract.topology_overhead_counted);
    assert!(contract.ratio_living_reported);
}

#[test]
fn p74_invalid_topology_contracts_are_refused() {
    let cases = [
        "examples/invalid/p74_unknown_topology_kind.atlas",
        "examples/invalid/p74_unbounded_adjacency.atlas",
        "examples/invalid/p74_hidden_topology_storage.atlas",
        "examples/invalid/p74_missing_reopen_gate.atlas",
        "examples/invalid/p74_cache_required_for_correctness.atlas",
        "examples/invalid/p74_bad_graph_edge_policy.atlas",
        "examples/invalid/p74_bad_hyperedge_policy.atlas",
        "examples/invalid/p74_missing_guard_gate.atlas",
    ];

    for case in cases {
        assert!(
            p74_parse_topology_file(case).is_err(),
            "{} should be refused",
            case
        );
    }
}

#[test]
fn p74_living_benchmark_uses_target_source_bytes_and_reports_ratio() {
    let report = p74_report(&temp_dir("target"));

    assert_eq!(report.target_source_bytes, 1_048_576);
    assert_eq!(report.actual_source_bytes, 1_048_576);
    assert!(!report.results.is_empty());
    assert!(report.comparison.best_ratio_living > 0.0);
    assert!(report
        .results
        .iter()
        .all(|result| result.cold_persisted_bytes > 0));
    assert_eq!(
        report.decision,
        TopologyDecision::RecalibrateFiberTopologySearch
    );
}

#[test]
fn p74_living_benchmark_preserves_guard_and_reopen() {
    let report = p74_report(&temp_dir("guard"));

    assert!(report.reopen_equivalence);
    assert!(report.guard_no_false_gain);
    assert_eq!(report.guard_decision, "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED");
    assert!(report
        .results
        .iter()
        .filter(|result| result.corpus_kind == RealDataCorpusKind::IncompressibleGuardBlob)
        .all(|result| result.guard_no_false_gain && result.ratio_living <= 1.0));
}

#[test]
fn p74_phase_map_and_rankings_are_structured() {
    let report = p74_report(&temp_dir("phase"));

    assert_eq!(report.phase_map.phase_map_version, "p74_phase_map_v1");
    assert!(report.phase_map.green_count > 0);
    assert!(report.phase_map.yellow_count > 0);
    assert!(report.phase_map.red_count > 0);
    assert_ne!(report.comparison.best_topology_overall, "not_available");
    assert_ne!(
        report.comparison.best_topology_by_ratio_living,
        "not_available"
    );
    assert_ne!(
        report.comparison.best_topology_by_retrieval,
        "not_available"
    );
    assert_ne!(
        report.comparison.best_topology_by_update_cost,
        "not_available"
    );
}

#[test]
fn p74_topology_overhead_is_counted() {
    let report = p74_report(&temp_dir("overhead"));

    assert!(report
        .results
        .iter()
        .all(|result| result.topology_overhead_bytes > 0));
    assert!(report
        .results
        .iter()
        .all(|result| result.topology_overhead_ratio >= 0.0));
    assert!(report
        .topologies
        .iter()
        .all(|topology| topology.hidden_topology_storage_risk == "low"));
}

#[test]
fn p74_corpus_specific_topologies_are_relevant() {
    let code = focused_report(
        &temp_dir("code"),
        RealDataCorpusKind::RealCode,
        TopologyKind::GraphAdjacencyFiber,
    );
    let logs = focused_report(
        &temp_dir("logs"),
        RealDataCorpusKind::RealishLogs,
        TopologyKind::HypergraphTagFiber,
    );
    let json = focused_report(
        &temp_dir("json"),
        RealDataCorpusKind::RealishJsonRecords,
        TopologyKind::TriePrefixFiber,
    );
    let csv = focused_report(
        &temp_dir("csv"),
        RealDataCorpusKind::SparseCsvTable,
        TopologyKind::HierarchicalTileFiber,
    );

    assert_eq!(
        code.comparison.best_topology_by_ratio_living,
        "graph_adjacency_fiber"
    );
    assert_eq!(
        logs.comparison.best_topology_by_ratio_living,
        "hypergraph_tag_fiber"
    );
    assert_eq!(
        json.comparison.best_topology_by_ratio_living,
        "trie_prefix_fiber"
    );
    assert_eq!(
        csv.comparison.best_topology_by_ratio_living,
        "hierarchical_tile_fiber"
    );
}

#[test]
fn p74_exports_are_written() {
    let export_dir = temp_dir("exports");
    let report = p74_report(&export_dir);

    assert_eq!(report.astra_step, "P74");
    assert!(export_dir.join("p74_topology_living_report.json").exists());
    assert!(export_dir.join("p74_topology_results.jsonl").exists());
    assert!(export_dir.join("p74_phase_map.csv").exists());
    assert!(export_dir.join("p74_cost_breakdown.csv").exists());
    assert!(export_dir.join("p74_summary.md").exists());
    assert!(export_dir
        .join("topology_stores/trie_prefix_fiber/real_code_corpus_10m/cold/topology/topology.meta")
        .exists());
}

#[test]
fn p74_json_contains_required_sections_without_timing_goldens() {
    let json = p74_topology_living_json(&p74_report(&temp_dir("json-shape")));

    assert!(json.contains("\"astra_step\": \"P74\""));
    assert!(json.contains("\"best_topology_overall\""));
    assert!(json.contains("\"phase_map_summary\""));
    assert!(json.contains("\"ratio_living_p72_baseline\""));
    assert!(json.contains("RECALIBRATE_P74_FIBER_TOPOLOGY_SEARCH"));
    assert!(!json.contains("duration_ns"));
}

#[test]
fn p74_cli_topology_living_bench_succeeds() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "topology-living-bench",
            "--corpus",
            "all",
            "--topology",
            "all",
            "--target-source-bytes",
            "1048576",
            "--cycles",
            "2",
            "--queries",
            "500",
            "--updates",
            "50",
            "--deletes",
            "5",
            "--compact",
            "threshold",
            "--adaptive",
            "on",
            "--locality",
            "mixed",
            "--update-pressure",
            "medium",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("topology-living-bench");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"astra_step\": \"P74\""));
    assert!(export_dir.join("p74_topology_living_report.json").exists());
}

#[test]
fn p74_keeps_p73_cubical_path_available() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["check", "examples/valid/p73_cubical_living_store.atlas"])
        .output()
        .expect("p73 check");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("p73_topology"));
}

#[test]
fn p74_test_stack_audit_report_is_versioned() {
    assert!(fs::metadata("docs/analysis/ASTRA-P74-test-stack-audit.md").is_ok());
}

fn p74_report(export_dir: &std::path::Path) -> astra_atlas_lang::TopologyStoreReport {
    p74_topology_living_bench(
        P74TopologyLivingOptions {
            corpora: p71_all_corpora(),
            topologies: p74_all_topologies(),
            target_source_bytes: 1_048_576,
            cycles: 2,
            queries: 500,
            updates: 50,
            deletes: 5,
            compact: P74CompactionPolicy::Threshold,
            adaptive: true,
            locality: P74LocalityProfile::Mixed,
            update_pressure: P74UpdatePressure::Medium,
        },
        export_dir,
    )
    .expect("P74 topology living report")
}

fn focused_report(
    export_dir: &std::path::Path,
    corpus: RealDataCorpusKind,
    topology: TopologyKind,
) -> astra_atlas_lang::TopologyStoreReport {
    p74_topology_living_bench(
        P74TopologyLivingOptions {
            corpora: vec![corpus],
            topologies: vec![TopologyKind::BaselineLinearFiber, topology],
            target_source_bytes: 1_048_576,
            cycles: 2,
            queries: 500,
            updates: 50,
            deletes: 5,
            compact: P74CompactionPolicy::Threshold,
            adaptive: true,
            locality: P74LocalityProfile::Mixed,
            update_pressure: P74UpdatePressure::Medium,
        },
        export_dir,
    )
    .expect("focused P74 topology living report")
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "astra-p74-test-{}-{}-{}",
        std::process::id(),
        label,
        nanos
    ))
}
