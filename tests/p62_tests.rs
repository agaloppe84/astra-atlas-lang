use astra_atlas_lang::{
    p61_virtual_ratio_report_json_file, p62_real_ratio_report_file,
    p62_real_ratio_report_file_with_runs, p62_real_ratio_report_json_file, WorkloadMode,
};
use std::process::Command;

#[test]
fn p62_ratio_real_smoke_report_has_measured_schema() {
    let report = p62_real_ratio_report_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("p62 smoke report");

    assert_eq!(report.astra_iteration, "ASTRA-P62");
    assert_eq!(report.cost_model, "measured_real_v1");
    assert_eq!(report.measurement_kind, "real_wall_clock_and_filesystem");
    assert_eq!(report.iteration_count, 100);
    assert_eq!(report.repeat_count, 1);
    assert_eq!(report.warmup_count, 10);
    assert_eq!(report.operation_count, 100);
    assert_eq!(report.summary.run_count, 1);
    assert_eq!(report.runs.len(), 1);
    assert_eq!(
        report.decision.as_str(),
        "RECALIBRATE_P62_MEASUREMENT_MODEL"
    );
}

#[test]
fn p62_ratio_real_smoke_supports_repeated_runs() {
    let report =
        p62_real_ratio_report_file_with_runs("examples/p53_strict.atlas", WorkloadMode::Smoke, 3)
            .expect("p62 repeated smoke report");

    assert_eq!(report.repeat_count, 3);
    assert_eq!(report.summary.run_count, 3);
    assert_eq!(report.runs.len(), 3);
    assert!(report.summary.all_runs_passed);
    assert_eq!(
        report.decision.as_str(),
        "RECALIBRATE_P62_MEASUREMENT_MODEL"
    );
    for (idx, run) in report.runs.iter().enumerate() {
        assert_eq!(run.run_index, idx);
        assert!(run.run_id.contains(&format!(":{}", idx)));
        assert!(run.total_persisted_bytes > 0);
        assert!(run.ratio_effective_per_byte > 0.0);
    }
}

#[test]
fn p62_repeated_summary_orders_key_fields() {
    let report =
        p62_real_ratio_report_file_with_runs("examples/p53_strict.atlas", WorkloadMode::Smoke, 3)
            .expect("p62 repeated smoke report");
    let summary = report.summary;

    assert!(summary.total_persisted_bytes_min <= summary.total_persisted_bytes_median);
    assert!(summary.total_persisted_bytes_median <= summary.total_persisted_bytes_max);
    assert!(summary.ratio_effective_per_byte_min <= summary.ratio_effective_per_byte_median);
    assert!(summary.ratio_effective_per_byte_median <= summary.ratio_effective_per_byte_max);
    assert!(summary.read_p99_us_min <= summary.read_p99_us_median);
    assert!(summary.read_p99_us_median <= summary.read_p99_us_max);
    assert!(summary.update_p99_us_min <= summary.update_p99_us_median);
    assert!(summary.update_p99_us_median <= summary.update_p99_us_max);
    assert!(summary.snapshot_p99_us_min <= summary.snapshot_p99_us_median);
    assert!(summary.snapshot_p99_us_median <= summary.snapshot_p99_us_max);
    assert!(summary.rebuild_p99_us_min <= summary.rebuild_p99_us_median);
    assert!(summary.rebuild_p99_us_median <= summary.rebuild_p99_us_max);
    assert!(summary.audit_p99_us_min <= summary.audit_p99_us_median);
    assert!(summary.audit_p99_us_median <= summary.audit_p99_us_max);
}

#[test]
fn p62_timing_fields_are_measured_and_non_zero_for_smoke() {
    let report =
        p62_real_ratio_report_file_with_runs("examples/p53_strict.atlas", WorkloadMode::Smoke, 3)
            .expect("p62 smoke report");

    for run in &report.runs {
        assert!(run.create_timing.p50_us > 0);
        assert!(run.read_timing.p50_us > 0);
        assert!(run.update_timing.p50_us > 0);
        assert!(run.delete_timing.p50_us > 0);
        assert!(run.snapshot_timing.p50_us > 0);
        assert!(run.rebuild_timing.p50_us > 0);
        assert!(run.audit_timing.p50_us > 0);
    }
}

#[test]
fn p62_persisted_bytes_are_real_and_sum_correctly() {
    let report = p62_real_ratio_report_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("p62 smoke report");
    let bytes = report.persisted_bytes;

    assert!(bytes.snapshot_file_bytes > 0);
    assert!(bytes.manifest_file_bytes > 0);
    assert!(bytes.journal_file_bytes > 0);
    assert!(bytes.index_file_bytes > 0);
    assert!(bytes.payload_file_bytes > 0);
    assert!(bytes.audit_file_bytes > 0);
    assert_eq!(
        bytes.total(),
        bytes.snapshot_file_bytes
            + bytes.manifest_file_bytes
            + bytes.journal_file_bytes
            + bytes.index_file_bytes
            + bytes.payload_file_bytes
            + bytes.audit_file_bytes
    );
}

#[test]
fn p62_effective_ratio_per_byte_is_derived_from_measured_bytes() {
    let report = p62_real_ratio_report_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("p62 smoke report");
    let expected = report.virtual_effective as f64 / report.persisted_bytes.total() as f64;

    assert!(report.persisted_bytes.total() > 0);
    assert!((report.ratio_effective_per_byte - expected).abs() < f64::EPSILON);
}

#[test]
fn p62_guard_and_adversarial_workloads_remain_ineffective() {
    let report = p62_real_ratio_report_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("p62 smoke report");

    assert!(report.guard_refused);
    assert!(report.dangerous_or_adversarial_refused);
    assert!(report.workloads.iter().any(|workload| {
        workload.mechanism == "guard_refusal" && workload.refused && workload.virtual_effective == 0
    }));
    assert!(report.workloads.iter().any(|workload| {
        workload.mechanism == "adversarial_refusal"
            && workload.refused
            && workload.virtual_effective == 0
    }));
}

#[test]
fn p62_runtime_safety_roundtrip_gates_pass_for_smoke() {
    let report = p62_real_ratio_report_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("p62 smoke report");

    assert!(report.audit_passed);
    assert!(report.rebuild_passed);
    assert!(report.snapshot_roundtrip_passed);
    assert_eq!(report.create_count, 100);
    assert_eq!(report.read_count, 100);
    assert_eq!(report.update_count, 100);
    assert_eq!(report.delete_count, 10);
    assert_eq!(report.snapshot_count, 1);
    assert_eq!(report.rebuild_count, 1);
    assert_eq!(report.audit_count, 1);
}

#[test]
fn p62_ratio_real_cli_smoke_json_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-real",
            "examples/p53_strict.atlas",
            "--mode",
            "smoke",
            "--format",
            "json",
        ])
        .output()
        .expect("run p62 ratio-real cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_iteration\": \"ASTRA-P62\""));
    assert!(stdout.contains("\"cost_model\": \"measured_real_v1\""));
    assert!(stdout.contains("\"measurement_kind\": \"real_wall_clock_and_filesystem\""));
    assert!(stdout.contains("\"decision\": \"RECALIBRATE_P62_MEASUREMENT_MODEL\""));
}

#[test]
fn p62_ratio_real_cli_smoke_json_with_runs_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-real",
            "examples/p53_strict.atlas",
            "--mode",
            "smoke",
            "--format",
            "json",
            "--runs",
            "3",
        ])
        .output()
        .expect("run p62 ratio-real cli with runs");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"repeat_count\": 3"));
    assert!(stdout.contains("\"run_count\": 3"));
    assert!(stdout.contains("\"runs\": ["));
}

#[test]
fn p62_ratio_real_rejects_invalid_mode_and_format() {
    let bad_mode = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-real",
            "examples/p53_strict.atlas",
            "--mode",
            "full",
            "--format",
            "json",
        ])
        .output()
        .expect("run p62 bad mode");
    assert!(!bad_mode.status.success());
    assert!(String::from_utf8_lossy(&bad_mode.stderr).contains("unsupported mode 'full'"));

    let bad_format = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-real",
            "examples/p53_strict.atlas",
            "--mode",
            "smoke",
            "--format",
            "markdown",
        ])
        .output()
        .expect("run p62 bad format");
    assert!(!bad_format.status.success());
    assert!(
        String::from_utf8_lossy(&bad_format.stderr).contains("ratio-real requires --format json")
    );
}

#[test]
fn p62_ratio_real_rejects_invalid_runs_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-real",
            "examples/p53_strict.atlas",
            "--mode",
            "smoke",
            "--format",
            "json",
            "--runs",
            "0",
        ])
        .output()
        .expect("run p62 bad runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("ratio-real requires --runs greater than zero"));
}

#[test]
fn p62_keeps_p61_smoke_golden_stable() {
    let json = p61_virtual_ratio_report_json_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("p61 smoke report");
    let expected = include_str!("golden/p61_ratio_smoke.json").trim_end_matches('\n');

    assert_eq!(json, expected);
}

#[test]
fn p62_json_contains_required_top_level_fields() {
    let json = p62_real_ratio_report_json_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("p62 smoke json");

    assert!(json.contains("\"create_p50_us\":"));
    assert!(json.contains("\"repeat_count\": 1"));
    assert!(json.contains("\"summary\": {"));
    assert!(json.contains("\"runs\": ["));
    assert!(json.contains("\"read_p99_us\":"));
    assert!(json.contains("\"snapshot_file_bytes\":"));
    assert!(json.contains("\"total_persisted_bytes\":"));
    assert!(json.contains("\"ratio_effective_per_byte\":"));
    assert!(json.contains("\"snapshot_roundtrip_passed\": true"));
    assert!(json.contains("\"workloads\": ["));
}
