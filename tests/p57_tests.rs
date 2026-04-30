use astra_atlas_lang::{
    p57_decision, p57_report, p57_report_json, validate, DiagnosticCode, P57DecisionGates,
};

const VALID: &str = include_str!("../examples/p53_strict.atlas");
const GUARD_ACTIVE: &str = include_str!("../examples/invalid/guard_active.atlas");
const SNAPSHOT_FULL: &str = include_str!("../examples/invalid/snapshot_full.atlas");

#[test]
fn p57_report_exists_and_has_json_object_shape() {
    let json = p57_report_json(VALID).expect("p57 report should exist");

    assert!(json.starts_with("{\n"));
    assert!(json.ends_with('}'));
    assert!(json.contains("\"astra_iteration\": \"ASTRA-P57\""));
    assert!(json.contains("\"astra_p57_decision\": \"RECALIBRATE_P57\""));
}

#[test]
fn p57_report_preserves_guard_refusal() {
    let report = p57_report(VALID).expect("p57 report should be built");
    assert!(report.guard_refused);
    assert_eq!(report.refused_family_count, 1);

    let err = validate(GUARD_ACTIVE).expect_err("guard_active must remain invalid");
    assert_eq!(err.code, DiagnosticCode::GuardActive);
}

#[test]
fn p57_report_refuses_snapshot_full_program() {
    let err = p57_report_json(SNAPSHOT_FULL).expect_err("snapshot_full must fail report");

    assert_eq!(err.code, DiagnosticCode::SnapshotFullStrict);
    assert_eq!(err.field.as_deref(), Some("snapshot"));
}

#[test]
fn p57_report_does_not_weaken_strict_p53() {
    let report = p57_report(VALID).expect("p57 report should be built");

    assert!(report.strict_p53_enabled);
    assert_eq!(report.snapshot_policy, "incremental_manifest");
    assert!(report.snapshot_full_refused);
}

#[test]
fn p57_report_json_matches_golden() {
    let json = p57_report_json(VALID).expect("p57 report json should be built");
    let expected = include_str!("golden/p57_report.json").trim_end_matches('\n');

    assert_eq!(json, expected);
}

#[test]
fn p57_report_json_is_deterministic() {
    let first = p57_report_json(VALID).expect("first p57 report");
    let second = p57_report_json(VALID).expect("second p57 report");

    assert_eq!(first, second);
}

#[test]
fn p57_decision_is_not_validate_if_core_gates_are_missing() {
    let missing_strict = P57DecisionGates {
        strict_p53_preserved: false,
        guard_remains_refused: true,
        snapshot_full_refused: true,
        valid_examples_pass: true,
        invalid_examples_fail: true,
        runtime_smoke_path_exists: true,
        report_json_stable: true,
        cargo_tests_no_regression: Some(true),
    };
    assert_eq!(p57_decision(&missing_strict), "NO_GO_P57");

    let missing_external_validation = P57DecisionGates {
        strict_p53_preserved: true,
        guard_remains_refused: true,
        snapshot_full_refused: true,
        valid_examples_pass: true,
        invalid_examples_fail: true,
        runtime_smoke_path_exists: true,
        report_json_stable: true,
        cargo_tests_no_regression: None,
    };
    assert_eq!(
        p57_decision(&missing_external_validation),
        "RECALIBRATE_P57"
    );
}
