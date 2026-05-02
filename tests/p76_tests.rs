use astra_atlas_lang::{
    p71_all_corpora, p76_all_compare_targets, p76_parse_process_file,
    p76_process_contract_report_file, p76_routing_oracle_bench, p76_routing_oracle_json,
    p76_virtual_space_estimate, P74LocalityProfile, P74UpdatePressure, P76CompareTarget,
    P76Decision, P76VirtualSpaceEstimateOptions, RouterPolicy, RoutingOracleOptions,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID_PROCESS: &str = "examples/valid/p76_routing_oracle_virtual_space.atlas";

#[test]
fn p76_routing_oracle_exists_and_targets_are_available() {
    let targets = p76_all_compare_targets();

    assert!(targets.contains(&P76CompareTarget::Oracle));
    assert!(targets.contains(&P76CompareTarget::Policy(RouterPolicy::Mixed)));
    assert!(targets.contains(&P76CompareTarget::Policy(RouterPolicy::HierarchicalOnly)));
    assert!(targets.contains(&P76CompareTarget::Policy(RouterPolicy::LinearOnly)));
}

#[test]
fn p76_process_contract_parses_and_typechecks() {
    let contract = p76_parse_process_file(VALID_PROCESS).expect("P76 contract parses");
    let report = p76_process_contract_report_file(VALID_PROCESS).expect("P76 report");

    assert_eq!(contract.oracle_id, "oracle_v1");
    assert!(contract.compare_targets.contains(&"oracle".to_string()));
    assert!(contract.local_on_address);
    assert_eq!(contract.virtual_bytes_claim, "equivalent");
    assert!(report.typecheck_ok);
    assert!(report.virtual_space_metrics_required);
    assert!(!report.hidden_router_overhead);
}

#[test]
fn p76_invalid_process_contracts_are_refused() {
    let cases = [
        "examples/invalid/p76_missing_virtual_space_metrics.atlas",
        "examples/invalid/p76_non_living_routing_decision.atlas",
        "examples/invalid/p76_missing_guard_gate.atlas",
        "examples/invalid/p76_hidden_router_overhead.atlas",
        "examples/invalid/p76_unbounded_oracle_compare.atlas",
        "examples/invalid/p76_bad_virtual_bytes_claim.atlas",
        "examples/invalid/p76_missing_local_on_address.atlas",
        "examples/invalid/p76_ratio_not_reported.atlas",
    ];

    for case in cases {
        assert!(
            p76_parse_process_file(case).is_err(),
            "{} should be refused",
            case
        );
    }
}

#[test]
fn p76_virtual_space_estimator_marks_virtual_bytes_as_equivalent() {
    let metrics = p76_virtual_space_estimate(P76VirtualSpaceEstimateOptions {
        topology: "mixed".to_string(),
        target_source_bytes: 10_485_760,
        cells: 10_000,
        fibers_per_cell: 4,
        hierarchy_depth: 5,
    });

    assert_eq!(metrics.virtual_cell_count, 10_000);
    assert_eq!(metrics.virtual_fiber_count, 40_000);
    assert!(metrics.virtual_effective_bytes_equivalent > 10_485_760);
    assert!(metrics.bytes_are_equivalent_not_stored);
}

#[test]
fn p76_oracle_compares_mixed_and_calculates_regret() {
    let report = p76_report(&temp_dir("oracle"));

    assert!(report.routing_regret.router_vs_oracle_ratio >= 0.95);
    assert!(report.routing_regret.wrong_route_count > 0);
    assert!(report.routing_regret.wrong_route_cost > 0);
    assert!(report.routing_regret.routing_accuracy > 0.90);
    assert_eq!(
        report.decision,
        P76Decision::FreezeCoreSpecAndRecalibrateRouter
    );
}

#[test]
fn p76_living_report_contains_crud_and_guard_metrics() {
    let report = p76_report(&temp_dir("crud"));
    let json = p76_routing_oracle_json(&report);

    assert_eq!(report.target_source_bytes, 1_048_576);
    assert_eq!(report.actual_source_bytes, 1_048_576);
    assert!(report.crud_metrics.address_lookup_steps_p95 > 0.0);
    assert_eq!(report.crud_metrics.crud_success_rate, 1.0);
    assert_eq!(report.crud_metrics.update_success_rate, 1.0);
    assert!(report
        .target_results
        .iter()
        .all(|result| result.guard_decision == "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED"));
    assert!(!json.contains("duration_ns"));
}

#[test]
fn p76_phase_map_produces_green_yellow_red() {
    let report = p76_report(&temp_dir("phase"));

    assert_eq!(
        report.phase_map.phase_map_version,
        "p76_router_oracle_phase_map_v1"
    );
    assert!(report.phase_map.green_count > 0);
    assert!(report.phase_map.yellow_count > 0);
    assert!(report.phase_map.red_count > 0);
}

#[test]
fn p76_exports_are_written() {
    let export_dir = temp_dir("exports");
    let report = p76_report(&export_dir);

    assert_eq!(report.astra_step, "P76");
    assert!(export_dir.join("p76_routing_oracle_report.json").exists());
    assert!(export_dir.join("p76_route_decisions.jsonl").exists());
    assert!(export_dir.join("p76_virtual_space_metrics.json").exists());
    assert!(export_dir.join("p76_virtual_space_metrics.csv").exists());
    assert!(export_dir.join("p76_crud_metrics.csv").exists());
    assert!(export_dir.join("p76_phase_map.csv").exists());
    assert!(export_dir.join("p76_historical_comparison.json").exists());
    assert!(export_dir.join("p76_summary.md").exists());
    assert!(export_dir
        .join("stores/mixed_router/cold/router_oracle/router_oracle.bin")
        .exists());
}

#[test]
fn p76_cli_routing_oracle_and_virtual_space_succeed() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "routing-oracle-bench",
            "--corpus",
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
            "--locality",
            "all",
            "--update-pressure",
            "all",
            "--compare",
            "oracle,mixed,hierarchical,linear,cubical,trie,graph,hypergraph",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("routing-oracle-bench");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"astra_step\": \"P76\""));

    let estimate = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "virtual-space-estimate",
            "--topology",
            "mixed",
            "--target-source-bytes",
            "10485760",
            "--cells",
            "10000",
            "--fibers-per-cell",
            "4",
            "--hierarchy-depth",
            "5",
            "--format",
            "json",
        ])
        .output()
        .expect("virtual-space-estimate");
    assert!(
        estimate.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&estimate.stderr)
    );
    assert!(String::from_utf8_lossy(&estimate.stdout)
        .contains("\"bytes_are_equivalent_not_stored\": true"));
}

#[test]
fn p76_keeps_p75_router_path_available() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["check", "examples/valid/p75_mixed_topology_router.atlas"])
        .output()
        .expect("p75 check");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("p75_router"));
}

#[test]
fn p76_specs_and_test_stack_audit_are_versioned() {
    assert!(fs::metadata("docs/specs/ASTRA_CORE_SPEC_P76.md").is_ok());
    assert!(fs::metadata("docs/specs/ATLAS_LANGUAGE_SPEC_P76.md").is_ok());
    assert!(fs::metadata("docs/specs/ASTRA_PATTERNS_CATALOG_P76.md").is_ok());
    assert!(fs::metadata("docs/analysis/ASTRA-P76-test-stack-audit.md").is_ok());
}

fn p76_report(export_dir: &std::path::Path) -> astra_atlas_lang::RoutingOracleReport {
    p76_routing_oracle_bench(
        RoutingOracleOptions {
            corpora: p71_all_corpora(),
            target_source_bytes: 1_048_576,
            cycles: 2,
            queries: 500,
            updates: 50,
            deletes: 5,
            locality_profiles: P74LocalityProfile::all(),
            update_pressures: P74UpdatePressure::all(),
            compare: p76_all_compare_targets(),
        },
        export_dir,
    )
    .expect("P76 routing oracle report")
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "astra-p76-test-{}-{}-{}",
        std::process::id(),
        label,
        nanos
    ))
}
