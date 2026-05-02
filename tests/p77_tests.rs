use astra_atlas_lang::{
    p71_all_corpora, p77_calibrate_router, p77_calibration_json, p77_evaluate_promotion_gates,
    p77_parse_router_policy_file, p77_router_policy_report_file, P74LocalityProfile,
    P74UpdatePressure, P77CalibrationGridKind, P77Decision, P77PromotionGateInput,
    P77RouterCalibrationOptions, RouterCalibrationGrid, RouterThresholdSet,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID_POLICY: &str = "examples/valid/p77_calibrated_router_policy.atlas";

#[test]
fn p77_calibration_grid_builds() {
    let smoke = RouterCalibrationGrid::build(P77CalibrationGridKind::Smoke);
    let standard = RouterCalibrationGrid::build(P77CalibrationGridKind::Standard);

    assert!(!smoke.threshold_sets.is_empty());
    assert!(standard.threshold_sets.len() > smoke.threshold_sets.len());
    assert!(standard
        .threshold_sets
        .iter()
        .any(|set| set.threshold_set_id == "p77_calibrated_router_v1"));
}

#[test]
fn p77_router_threshold_set_parses() {
    let contract = p77_parse_router_policy_file(VALID_POLICY).expect("P77 policy parses");
    let report = p77_router_policy_report_file(VALID_POLICY).expect("P77 report");

    assert_eq!(contract.policy.threshold_set_id, "p77_calibrated_router_v1");
    assert_eq!(contract.policy.guard_threshold, "strict");
    assert!(contract.living_memory_only);
    assert_eq!(contract.router_oracle_ratio_min, 0.985);
    assert!(report.typecheck_ok);
    assert!(!report.hidden_router_overhead);
}

#[test]
fn p77_invalid_router_policies_are_refused() {
    let cases = [
        "examples/invalid/p77_missing_oracle_ratio_gate.atlas",
        "examples/invalid/p77_low_accuracy_threshold.atlas",
        "examples/invalid/p77_missing_wrong_route_budget.atlas",
        "examples/invalid/p77_hidden_router_overhead.atlas",
        "examples/invalid/p77_non_living_calibration.atlas",
        "examples/invalid/p77_missing_guard_gate.atlas",
        "examples/invalid/p77_bad_bias_value.atlas",
        "examples/invalid/p77_unbounded_fallback.atlas",
    ];

    for case in cases {
        assert!(
            p77_parse_router_policy_file(case).is_err(),
            "{} should be refused",
            case
        );
    }
}

#[test]
fn p77_calibration_report_contains_regret_and_virtual_metrics() {
    let report = p77_report(&temp_dir("report"), P77CalibrationGridKind::Smoke);
    let json = p77_calibration_json(&report);

    assert_eq!(report.astra_step, "P77");
    assert!(report.best_calibrated.ratio_living_router >= report.p76_baseline_router_ratio_living);
    assert!(report.best_calibrated.router_oracle_ratio >= 0.95);
    assert!(report.best_calibrated.wrong_route_count < report.p76_baseline_wrong_route_count);
    assert!(report.best_calibrated.wrong_route_cost < report.p76_baseline_wrong_route_cost);
    assert!(report.best_calibrated.calibrated_score > 0.0);
    assert!(report.virtual_space_metrics.bytes_are_equivalent_not_stored);
    assert!(report.cold_persisted_bytes > 0);
    assert!(report.runtime_peak_bytes > 0);
    assert!(json.contains("\"virtual_effective_bytes_equivalent\""));
    assert!(json.contains("\"cold_persisted_bytes\""));
    assert!(json.contains("\"runtime_peak_bytes\""));
    assert!(!json.contains("duration_ns"));
}

#[test]
fn p77_wrong_route_analyzer_groups_corpus_and_feature() {
    let report = p77_report(&temp_dir("wrong-routes"), P77CalibrationGridKind::Smoke);

    assert!(!report.wrong_route_analysis.wrong_route_by_corpus.is_empty());
    assert!(!report
        .wrong_route_analysis
        .wrong_route_by_feature
        .is_empty());
    assert!(report
        .wrong_route_analysis
        .wrong_route_by_corpus
        .contains_key("real_code_corpus_10m"));
    assert!(!report.wrong_route_analysis.systematic_biases.is_empty());
}

#[test]
fn p77_safety_factor_zero_when_guard_fails() {
    let decision = p77_evaluate_promotion_gates(&P77PromotionGateInput {
        router_oracle_ratio: 0.990,
        routing_accuracy: 0.970,
        wrong_route_cost_reduction: 0.60,
        wrong_route_count_reduced: true,
        ratio_living_not_below_p76: true,
        update_audit_advantage_kept: true,
        retrieval_success_rate: 1.0,
        reopen_equivalence: true,
        drift_status: "NO_DRIFT".to_string(),
        guard_decision: "GUARD_FALSE_GAIN".to_string(),
        virtual_space_metrics_present: true,
        invalids_refused: true,
    });

    assert_eq!(decision, P77Decision::NoGoRouterCalibration);
}

#[test]
fn p77_promotion_gates_refuse_below_oracle_ratio() {
    let decision = p77_evaluate_promotion_gates(&P77PromotionGateInput {
        router_oracle_ratio: 0.984,
        routing_accuracy: 0.970,
        wrong_route_cost_reduction: 0.60,
        wrong_route_count_reduced: true,
        ratio_living_not_below_p76: true,
        update_audit_advantage_kept: true,
        retrieval_success_rate: 1.0,
        reopen_equivalence: true,
        drift_status: "NO_DRIFT".to_string(),
        guard_decision: "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED".to_string(),
        virtual_space_metrics_present: true,
        invalids_refused: true,
    });

    assert_eq!(decision, P77Decision::RecalibrateRouterThresholds);
}

#[test]
fn p77_promotion_gates_refuse_low_accuracy_and_cost_reduction() {
    let low_accuracy = p77_evaluate_promotion_gates(&P77PromotionGateInput {
        router_oracle_ratio: 0.990,
        routing_accuracy: 0.940,
        wrong_route_cost_reduction: 0.60,
        wrong_route_count_reduced: true,
        ratio_living_not_below_p76: true,
        update_audit_advantage_kept: true,
        retrieval_success_rate: 1.0,
        reopen_equivalence: true,
        drift_status: "NO_DRIFT".to_string(),
        guard_decision: "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED".to_string(),
        virtual_space_metrics_present: true,
        invalids_refused: true,
    });
    let low_cost_reduction = p77_evaluate_promotion_gates(&P77PromotionGateInput {
        wrong_route_cost_reduction: 0.30,
        ..promotion_input()
    });

    assert_eq!(low_accuracy, P77Decision::RecalibrateRouterThresholds);
    assert_eq!(low_cost_reduction, P77Decision::RecalibrateRouterThresholds);
}

#[test]
fn p77_promotion_passes_on_synthetic_strict_configuration() {
    assert_eq!(
        p77_evaluate_promotion_gates(&promotion_input()),
        P77Decision::PromoteMixedTopologyRouter
    );
}

#[test]
fn p77_exports_calibrated_policy_and_phase_map() {
    let export_dir = temp_dir("exports");
    let report = p77_report(&export_dir, P77CalibrationGridKind::Smoke);

    assert!(export_dir
        .join("p77_router_calibration_report.json")
        .exists());
    assert!(export_dir.join("p77_calibration_grid.csv").exists());
    assert!(export_dir.join("p77_wrong_routes.jsonl").exists());
    assert!(export_dir.join("p77_wrong_route_summary.csv").exists());
    assert!(export_dir.join("p77_calibrated_policy.json").exists());
    assert!(export_dir.join("p77_virtual_space_metrics.json").exists());
    assert!(export_dir.join("p77_summary.md").exists());
    assert!(report.phase_map.green_count > 0);
    assert!(report.phase_map.yellow_count > 0);
    assert!(report.phase_map.red_count > 0);
}

#[test]
fn p77_cli_routing_oracle_calibrate_succeeds() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "routing-oracle-calibrate",
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
            "--grid",
            "smoke",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("routing-oracle-calibrate");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"astra_step\": \"P77\""));
    assert!(export_dir.join("p77_calibrated_policy.json").exists());
}

#[test]
fn p77_check_command_and_p76_non_regression_remain_available() {
    let p77 = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["check", VALID_POLICY])
        .output()
        .expect("p77 check");
    assert!(
        p77.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&p77.stderr)
    );
    assert!(String::from_utf8_lossy(&p77.stdout).contains("p77_policy"));

    let p76 = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "check",
            "examples/valid/p76_routing_oracle_virtual_space.atlas",
        ])
        .output()
        .expect("p76 check");
    assert!(
        p76.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&p76.stderr)
    );
    assert!(String::from_utf8_lossy(&p76.stdout).contains("p76_oracle"));
}

#[test]
fn p77_test_stack_audit_is_versioned() {
    assert!(fs::metadata("docs/analysis/ASTRA-P77-test-stack-audit.md").is_ok());
}

fn p77_report(
    export_dir: &std::path::Path,
    grid_kind: P77CalibrationGridKind,
) -> astra_atlas_lang::P77CalibrationReport {
    p77_calibrate_router(
        P77RouterCalibrationOptions {
            corpora: p71_all_corpora(),
            target_source_bytes: 1_048_576,
            cycles: 2,
            queries: 500,
            updates: 50,
            deletes: 5,
            locality_profiles: P74LocalityProfile::all(),
            update_pressures: P74UpdatePressure::all(),
            grid_kind,
        },
        export_dir,
    )
    .expect("P77 calibration report")
}

fn promotion_input() -> P77PromotionGateInput {
    P77PromotionGateInput {
        router_oracle_ratio: 0.990,
        routing_accuracy: 0.970,
        wrong_route_cost_reduction: 0.60,
        wrong_route_count_reduced: true,
        ratio_living_not_below_p76: true,
        update_audit_advantage_kept: true,
        retrieval_success_rate: 1.0,
        reopen_equivalence: true,
        drift_status: "NO_DRIFT".to_string(),
        guard_decision: "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED".to_string(),
        virtual_space_metrics_present: true,
        invalids_refused: true,
    }
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    path.push(format!("astra_p77_{}_{}", label, nanos));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

#[test]
fn p77_threshold_set_default_is_deterministic() {
    let policy = RouterThresholdSet::calibrated_default();
    assert_eq!(policy.threshold_set_id, "p77_calibrated_router_v1");
    assert_eq!(policy.guard_threshold, "strict");
    assert!(policy.compact_id().contains("p77_calibrated_router_v1"));
}
