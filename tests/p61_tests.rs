use astra_atlas_lang::{
    p61_virtual_ratio_report_file, p61_virtual_ratio_report_json_file, WorkloadMode,
};
use std::process::Command;

#[test]
fn p61_smoke_report_is_deterministic() {
    let first =
        p61_virtual_ratio_report_json_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
            .expect("first p61 smoke report");
    let second =
        p61_virtual_ratio_report_json_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
            .expect("second p61 smoke report");

    assert_eq!(first, second);
    assert!(first.contains("\"astra_iteration\": \"ASTRA-P61\""));
    assert!(first.contains("\"cost_model\": \"deterministic_proxy_v1\""));
    assert!(first.contains("\"ratio_effective\":"));
    assert!(first.contains("\"ratio_declared\":"));
    assert!(first.contains("\"decision\": \"RECALIBRATE_P61_RATIO_COST_MODEL\""));
    assert!(first.contains("ratio_declared is informational only"));
}

#[test]
fn p61_smoke_report_matches_golden() {
    let json = p61_virtual_ratio_report_json_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("p61 smoke report");
    let expected = include_str!("golden/p61_ratio_smoke.json").trim_end_matches('\n');

    assert_eq!(json, expected);
}

#[test]
fn p61_virtual_ordering_invariant_holds() {
    let report = p61_virtual_ratio_report_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("p61 smoke report");
    let metrics = report.metrics;

    assert!(metrics.virtual_effective <= metrics.virtual_safe);
    assert!(metrics.virtual_safe <= metrics.virtual_updatable);
    assert!(metrics.virtual_updatable <= metrics.virtual_readable);
    assert!(metrics.virtual_readable <= metrics.virtual_reachable);
    assert!(metrics.virtual_reachable <= metrics.virtual_declared);
    assert!(metrics.ordering_invariants_hold);
}

#[test]
fn p61_real_total_cost_and_effective_ratio_are_consistent() {
    let report = p61_virtual_ratio_report_file("examples/p53_strict.atlas", WorkloadMode::Standard)
        .expect("p61 standard report");
    let metrics = report.metrics;

    assert!(metrics.real_total_cost_units() > 0);
    assert!(metrics.virtual_effective > 0);
    let expected = metrics.virtual_effective as f64 / metrics.real_total_cost_units() as f64;
    assert!((metrics.ratio_effective - expected).abs() < f64::EPSILON);
    assert!(metrics.ratio_declared >= metrics.ratio_effective);
}

#[test]
fn p61_top_level_totals_are_derived_from_workloads() {
    let report = p61_virtual_ratio_report_file("examples/p53_strict.atlas", WorkloadMode::Standard)
        .expect("p61 standard report");

    let virtual_declared: u128 = report
        .workloads
        .iter()
        .map(|workload| workload.virtual_declared)
        .sum();
    let virtual_effective: u128 = report
        .workloads
        .iter()
        .map(|workload| workload.virtual_effective)
        .sum();
    let real_total_cost_units: u128 = report
        .workloads
        .iter()
        .map(|workload| workload.real_total_cost_units())
        .sum();

    assert_eq!(report.metrics.virtual_declared, virtual_declared);
    assert_eq!(report.metrics.virtual_effective, virtual_effective);
    assert_eq!(
        report.metrics.real_total_cost_units(),
        real_total_cost_units
    );
    assert_eq!(
        report.metrics.ratio_effective,
        report.metrics.virtual_effective as f64 / report.metrics.real_total_cost_units() as f64
    );
}

#[test]
fn p61_workload_crud_and_refusal_semantics_are_explicit() {
    let report = p61_virtual_ratio_report_file("examples/p53_strict.atlas", WorkloadMode::Standard)
        .expect("p61 standard report");

    for workload in &report.workloads {
        assert_eq!(workload.refused(), !workload.accepted);
        if workload.accepted {
            assert!(workload.create_count > 0);
            assert!(workload.read_count > 0);
            assert!(workload.snapshot_count >= 1);
            assert!(workload.rebuild_count >= 1);
            assert!(workload.audit_count >= 1);
            assert_eq!(workload.refusal_reason, "none");
            assert!(workload.virtual_effective > 0);
        } else {
            assert!(workload.refused());
            assert_ne!(workload.refusal_reason, "none");
            assert_eq!(workload.virtual_effective, 0);
            assert!(workload.real_total_cost_units() > 0);
        }
    }
}

#[test]
fn p61_refuses_guard_and_adversarial_virtual_space() {
    let report = p61_virtual_ratio_report_file("examples/p53_strict.atlas", WorkloadMode::Standard)
        .expect("p61 standard report");

    assert!(report.metrics.guard_refused);
    assert!(report.metrics.dangerous_or_adversarial_refused);
    assert!(report
        .workloads
        .iter()
        .any(|workload| workload.guard_refused && workload.virtual_effective == 0));
    assert!(report.workloads.iter().any(
        |workload| workload.dangerous_or_adversarial_refused && workload.virtual_effective == 0
    ));
    assert!(report
        .workloads
        .iter()
        .any(|workload| workload.mechanism() == "guard_refusal"
            && workload.refusal_reason == "guard_random_space"));
    assert!(report
        .workloads
        .iter()
        .any(|workload| workload.mechanism() == "adversarial_refusal"
            && workload.refusal_reason == "adversarial_or_dangerous_space"));
}

#[test]
fn p61_cli_ratio_smoke_json_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio",
            "examples/p53_strict.atlas",
            "--mode",
            "smoke",
            "--format",
            "json",
        ])
        .output()
        .expect("run p61 ratio cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_iteration\": \"ASTRA-P61\""));
    assert!(stdout.contains("\"mode\": \"smoke\""));
    assert!(stdout.contains("\"ratio_effective\":"));
    assert!(stdout.contains("\"mechanism\": \"guard_refusal\""));
    assert!(stdout.contains("\"refusal_reason\": \"guard_random_space\""));
    assert!(stdout.contains("\"decision\": \"RECALIBRATE_P61_RATIO_COST_MODEL\""));
}

#[test]
fn p61_cli_ratio_rejects_invalid_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio",
            "examples/p53_strict.atlas",
            "--mode",
            "full",
            "--format",
            "json",
        ])
        .output()
        .expect("run p61 ratio cli");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported mode 'full'"));
}

#[test]
fn p61_cli_ratio_rejects_invalid_atlas_program() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio",
            "examples/invalid/snapshot_full.atlas",
            "--mode",
            "smoke",
            "--format",
            "json",
        ])
        .output()
        .expect("run p61 ratio cli on invalid atlas");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E_SNAPSHOT_FULL_STRICT"));
}

#[test]
fn p61_cli_ratio_rejects_markdown_format_explicitly() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio",
            "examples/p53_strict.atlas",
            "--mode",
            "smoke",
            "--format",
            "markdown",
        ])
        .output()
        .expect("run p61 ratio cli with markdown");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ratio requires --format json"));
}
