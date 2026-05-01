use astra_atlas_lang::{
    p71_all_corpora, p75_all_router_policies, p75_mixed_topology_bench, p75_mixed_topology_json,
    p75_parse_router_file, p75_router_contract_report_file, MixedTopologyRouter,
    P74CompactionPolicy, P74LocalityProfile, P74UpdatePressure, RouterDecision,
    RouterLivingOptions, RouterPolicy, TopologyKind,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID_ROUTER: &str = "examples/valid/p75_mixed_topology_router.atlas";

#[test]
fn p75_mixed_topology_router_exists() {
    let policies = p75_all_router_policies();

    assert!(policies.contains(&RouterPolicy::Mixed));
    assert!(policies.contains(&RouterPolicy::HierarchicalOnly));
    assert!(policies.contains(&RouterPolicy::LinearOnly));
    assert!(policies.contains(&RouterPolicy::CubicalOnly));
    assert!(policies.contains(&RouterPolicy::TrieOnly));
    assert!(policies.contains(&RouterPolicy::GraphOnly));
    assert!(policies.contains(&RouterPolicy::HypergraphOnly));
    assert_eq!(RouterPolicy::from_str("unknown"), None);
}

#[test]
fn p75_router_contract_parses_and_typechecks() {
    let contract = p75_parse_router_file(VALID_ROUTER).expect("P75 router contract parses");
    let report = p75_router_contract_report_file(VALID_ROUTER).expect("P75 report");

    assert_eq!(contract.router_id, "mixed_router");
    assert_eq!(contract.default_topology, "hierarchical_tile");
    assert_eq!(contract.guard_policy, "refuse_or_raw_no_gain");
    assert_eq!(contract.fallback, "bounded");
    assert!(contract.living_memory_only);
    assert!(contract.target_source_bytes >= 10_485_760);
    assert!(report.typecheck_ok);
    assert_eq!(report.route_count, 4);
}

#[test]
fn p75_invalid_router_contracts_are_refused() {
    let cases = [
        "examples/invalid/p75_unknown_router_topology.atlas",
        "examples/invalid/p75_missing_guard_policy.atlas",
        "examples/invalid/p75_hidden_router_storage.atlas",
        "examples/invalid/p75_non_living_decision_gate.atlas",
        "examples/invalid/p75_missing_target_source_bytes.atlas",
        "examples/invalid/p75_unbounded_router_fallback.atlas",
        "examples/invalid/p75_bad_route_condition.atlas",
        "examples/invalid/p75_missing_reopen_equivalence_gate.atlas",
    ];

    for case in cases {
        assert!(
            p75_parse_router_file(case).is_err(),
            "{} should be refused",
            case
        );
    }
}

#[test]
fn p75_routes_expected_feature_families() {
    let report = p75_report(&temp_dir("routes"));

    assert!(report
        .selected_topology_counts
        .contains_key(TopologyKind::TriePrefixFiber.as_str()));
    assert!(report
        .selected_topology_counts
        .contains_key(TopologyKind::GraphAdjacencyFiber.as_str()));
    assert!(report
        .selected_topology_counts
        .contains_key(TopologyKind::HypergraphTagFiber.as_str()));
    assert!(report
        .selected_topology_counts
        .contains_key(TopologyKind::HierarchicalTileFiber.as_str()));
    assert!(report
        .selected_topology_counts
        .contains_key(TopologyKind::BaselineLinearFiber.as_str()));
    assert!(report
        .selected_topology_counts
        .contains_key("refused_guard"));

    let code_counts = report
        .selected_topology_by_corpus
        .get("real_code_corpus_10m")
        .expect("code counts");
    assert!(code_counts.contains_key(TopologyKind::TriePrefixFiber.as_str()));
    assert!(code_counts.contains_key(TopologyKind::GraphAdjacencyFiber.as_str()));
}

#[test]
fn p75_guard_is_not_routed_to_success() {
    let report = p75_report(&temp_dir("guard"));

    assert_eq!(report.guard_decision, "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED");
    assert!(report.guard_no_false_gain);
    assert!(report.guard_refusal_count > 0);
    assert!(report
        .route_decisions
        .iter()
        .filter(|decision| decision.corpus_name == "incompressible_guard_10m")
        .all(|decision| decision.selected_topology == "refused_guard"));
}

#[test]
fn p75_living_benchmark_reports_ratio_and_baseline_comparison() {
    let report = p75_report(&temp_dir("ratio"));

    assert_eq!(report.target_source_bytes, 1_048_576);
    assert_eq!(report.actual_source_bytes, 1_048_576);
    assert!(report.comparison.ratio_living_router > 0.0);
    assert!(report.comparison.ratio_living_hierarchical_only > 0.0);
    assert!(report.comparison.router_vs_hierarchical_ratio >= 0.95);
    assert!(report.comparison.update_cost_router <= report.comparison.update_cost_hierarchical);
    assert!(result_policy(&report, RouterPolicy::Mixed).retrieval_success_rate >= 1.0);
    assert_eq!(report.decision, RouterDecision::RecalibrateRouterPolicy);
}

#[test]
fn p75_phase_map_has_green_yellow_red() {
    let report = p75_report(&temp_dir("phase"));

    assert_eq!(
        report.phase_map.phase_map_version,
        "p75_router_phase_map_v1"
    );
    assert!(report.phase_map.green_count > 0);
    assert!(report.phase_map.yellow_count > 0);
    assert!(report.phase_map.red_count > 0);
    assert_eq!(report.phase_map.best_router_policy, "mixed_router");
}

#[test]
fn p75_reopen_drift_and_no_timing_goldens() {
    let report = p75_report(&temp_dir("safety"));
    let json = p75_mixed_topology_json(&report);

    assert!(report.reopen_equivalence);
    assert_eq!(report.drift_status, "NO_DRIFT");
    assert!(json.contains("\"selected_topology_counts\""));
    assert!(json.contains("\"ratio_living_router\""));
    assert!(!json.contains("duration_ns"));
}

#[test]
fn p75_exports_are_written() {
    let export_dir = temp_dir("exports");
    let report = p75_report(&export_dir);

    assert_eq!(report.astra_step, "P75");
    assert!(export_dir.join("p75_mixed_topology_report.json").exists());
    assert!(export_dir.join("p75_router_decisions.jsonl").exists());
    assert!(export_dir.join("p75_topology_comparison.csv").exists());
    assert!(export_dir.join("p75_phase_map.csv").exists());
    assert!(export_dir.join("p75_cost_breakdown.csv").exists());
    assert!(export_dir.join("p75_summary.md").exists());
    assert!(export_dir
        .join("topology_stores/mixed-router/cold/router/router.policy")
        .exists());
}

#[test]
fn p75_cli_mixed_topology_bench_succeeds() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "mixed-topology-bench",
            "--corpus",
            "all",
            "--router",
            "mixed",
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
        .expect("mixed-topology-bench");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"astra_step\": \"P75\""));
    assert!(export_dir.join("p75_mixed_topology_report.json").exists());
}

#[test]
fn p75_keeps_p74_topology_living_path_available() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["check", "examples/valid/p74_living_topology_search.atlas"])
        .output()
        .expect("p74 check");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("p74_topology"));
}

#[test]
fn p75_test_stack_audit_report_is_versioned() {
    assert!(fs::metadata("docs/analysis/ASTRA-P75-test-stack-audit.md").is_ok());
}

#[test]
fn p75_router_default_is_deterministic() {
    let router = MixedTopologyRouter::default_router();

    assert_eq!(router.router_id, "mixed_router");
    assert_eq!(
        router.policy.default_topology,
        TopologyKind::HierarchicalTileFiber
    );
    assert!(router
        .feature_extractor
        .features
        .contains(&"guard_flag".to_string()));
}

fn p75_report(export_dir: &std::path::Path) -> astra_atlas_lang::RouterLivingReport {
    p75_mixed_topology_bench(
        RouterLivingOptions {
            corpora: p71_all_corpora(),
            router: RouterPolicy::Mixed,
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
    .expect("P75 mixed topology report")
}

fn result_policy(
    report: &astra_atlas_lang::RouterLivingReport,
    policy: RouterPolicy,
) -> &astra_atlas_lang::RouterPolicyResult {
    report
        .policy_results
        .iter()
        .find(|result| result.router_policy == policy)
        .expect("policy result")
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "astra-p75-test-{}-{}-{}",
        std::process::id(),
        label,
        nanos
    ))
}
