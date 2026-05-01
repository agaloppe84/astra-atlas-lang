use astra_atlas_lang::{
    p63_campaign_compare_json_files, p63_campaign_register_json_file,
    p63_campaign_report_file_with_runs, p63_campaign_report_to_json,
    p63_campaign_set_summary_json_file, p63_campaign_summary_json_file, write_p63_campaign_exports,
    P63Decision, P63StabilityStatus, P63ThresholdProfile, WorkloadMode,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn p63_campaign_report_has_required_schema_fields() {
    let report = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Smoke,
        2,
        P63ThresholdProfile::P63,
    )
    .expect("p63 campaign report");

    assert_eq!(report.astra_step, "P63");
    assert_eq!(report.cost_model, "measured_real_v1");
    assert_eq!(report.measurement_kind, "real_wall_clock_and_filesystem");
    assert_eq!(report.threshold_profile, "p63");
    assert_eq!(report.threshold_profile_resolved, "p63_conservative_v1");
    assert_eq!(
        report.threshold_profile_config.profile_id,
        "p63_conservative_v1"
    );
    assert_eq!(report.repeat_count, 2);
    assert_eq!(report.runs.len(), 2);
    assert!(report.machine_metadata.cpu_count.unwrap_or(0) > 0);
    assert!(!report.machine_metadata.os.is_empty());
    assert!(!report.machine_metadata.arch.is_empty());
    assert!(!report.machine_metadata.rustc_version.is_empty());
    assert!(!report.machine_metadata.cargo_version.is_empty());
    assert!(!report.machine_metadata.git_commit.is_empty());
    assert!(!report.machine_metadata.timestamp_utc.is_empty());
}

#[test]
fn p63_threshold_profile_alias_resolves_to_conservative_v1() {
    let profile = P63ThresholdProfile::from_str("p63").expect("p63 profile");
    let spec = profile.spec();

    assert_eq!(spec.profile_id, "p63_conservative_v1");
    assert_eq!(spec.alias, Some("p63"));
    assert_eq!(spec.min_runs_required, 10);
    assert!(!spec.allow_validate);
    assert!(spec.require_machine_metadata);
    assert!(spec.require_campaign_exports);
    assert!(!spec.require_realish_workloads);
    assert_eq!(spec.candidate_min_runs_for_future_validation, 30);
    assert_eq!(spec.candidate_min_campaigns_for_future_validation, 3);
    assert_eq!(spec.candidate_max_ratio_cv, 0.03);
    assert_eq!(spec.candidate_max_bytes_cv, 0.03);
    assert_eq!(spec.candidate_max_intra_mode_ratio_shift_percent, 5.0);
    assert_eq!(spec.candidate_max_intra_mode_bytes_shift_percent, 5.0);
    assert!(spec.candidate_requires_multi_machine);
}

#[test]
fn p63_robust_summary_contains_expected_statistics() {
    let report = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Smoke,
        3,
        P63ThresholdProfile::P63,
    )
    .expect("p63 campaign report");

    let ratio = &report.summary.ratio_effective_per_byte;
    assert!(ratio.min <= ratio.median);
    assert!(ratio.median <= ratio.max);
    assert!(ratio.mean > 0.0);
    assert!(ratio.stddev >= 0.0);
    assert!(ratio.coefficient_of_variation >= 0.0);

    let bytes = &report.summary.total_persisted_bytes;
    assert!(bytes.min > 0.0);
    assert!(bytes.min <= bytes.median);
    assert!(bytes.median <= bytes.max);

    let read_p99 = &report.summary.read_p99_us;
    assert!(read_p99.min > 0.0);
    assert!(read_p99.min <= read_p99.median);
    assert!(read_p99.median <= read_p99.max);
}

#[test]
fn p63_campaign_report_exposes_core_virtual_real_metrics() {
    let report = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Smoke,
        2,
        P63ThresholdProfile::P63,
    )
    .expect("p63 campaign report");

    let metrics = &report.core_metrics;
    assert!(metrics.virtual_declared_units >= metrics.virtual_reachable_units);
    assert!(metrics.virtual_reachable_units >= metrics.virtual_readable_units);
    assert!(metrics.virtual_readable_units >= metrics.virtual_updatable_units);
    assert!(metrics.virtual_updatable_units >= metrics.virtual_safe_units);
    assert!(metrics.virtual_safe_units >= metrics.virtual_effective_units);
    assert!(metrics.virtual_effective_units > 0);
    assert!(metrics.total_persisted_bytes > 0);
    assert!(metrics.payload_file_bytes > 0);
    assert!(metrics.index_file_bytes > 0);
    assert!(metrics.journal_file_bytes > 0);
    assert!(metrics.manifest_file_bytes > 0);
    assert!(metrics.checksum_or_audit_bytes.unwrap_or(0) > 0);
    assert_eq!(metrics.metadata_bytes, None);
    assert_eq!(metrics.assumed_materialized_value_bytes, 8);
    assert_eq!(
        metrics.estimated_materialized_bytes,
        metrics.virtual_declared_units * metrics.assumed_materialized_value_bytes
    );
    assert!(metrics.ratio_effective_per_byte > 0.0);
    assert!(metrics.gain_vs_materialized > 0.0);
    assert!(metrics.effective_gain_vs_materialized > 0.0);

    let expected_ratio =
        metrics.virtual_effective_units as f64 / metrics.total_persisted_bytes as f64;
    assert!((metrics.ratio_effective_per_byte - expected_ratio).abs() < 0.000001);

    let json = p63_campaign_report_to_json(&report);
    assert!(json.contains("\"core_ratio_metrics\": {"));
    assert!(json.contains("\"virtual_declared_units\":"));
    assert!(json.contains("\"virtual_effective_units\":"));
    assert!(json.contains("\"ratio_effective_per_byte\":"));
    assert!(json.contains("\"gain_vs_materialized\":"));
    assert!(json.contains("\"metadata_bytes\": null"));
}

#[test]
fn p63_campaign_report_contains_stability_statuses() {
    let report = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Smoke,
        3,
        P63ThresholdProfile::P63,
    )
    .expect("p63 campaign report");

    assert_eq!(
        report.ratio_stability_status,
        P63StabilityStatus::NotEnoughRuns
    );
    assert_eq!(
        report.bytes_stability_status,
        P63StabilityStatus::NotEnoughRuns
    );
    assert_eq!(
        report.timing_stability_status,
        P63StabilityStatus::NotEnoughRuns
    );
    assert_eq!(
        report.campaign_stability_status,
        P63StabilityStatus::NotEnoughRuns
    );
    assert!(report
        .stability_reasons
        .iter()
        .any(|reason| reason.contains("min_runs_required")));
}

#[test]
fn p63_decision_is_known_and_conservative_for_profile_p63() {
    let report = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Smoke,
        10,
        P63ThresholdProfile::P63,
    )
    .expect("p63 campaign report");

    assert!(matches!(
        report.decision,
        P63Decision::RecalibrateThresholds
            | P63Decision::RecalibrateWorkloads
            | P63Decision::NoGoMeasuredRatioStability
    ));
    assert_ne!(
        report.decision.as_str(),
        "VALIDATE_P63_MEASURED_RATIO_CALIBRATION"
    );
    assert!(matches!(
        report.campaign_stability_status,
        P63StabilityStatus::Stable | P63StabilityStatus::Warn | P63StabilityStatus::Unstable
    ));
}

#[test]
fn p63_campaign_exports_are_written() {
    let export_dir = unique_export_dir();
    let report = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Smoke,
        3,
        P63ThresholdProfile::P63,
    )
    .expect("p63 campaign report");

    write_p63_campaign_exports(&report, &export_dir).expect("write p63 exports");

    let campaign_report = fs::read_to_string(export_dir.join("campaign_report.json"))
        .expect("campaign_report.json exists");
    let runs_jsonl = fs::read_to_string(export_dir.join("runs.jsonl")).expect("runs.jsonl exists");
    let runs_csv = fs::read_to_string(export_dir.join("runs.csv")).expect("runs.csv exists");
    let summary_md = fs::read_to_string(export_dir.join("summary.md")).expect("summary.md exists");

    assert!(campaign_report.contains("\"astra_step\": \"P63\""));
    assert!(campaign_report.contains("\"threshold_profile\": \"p63\""));
    assert!(campaign_report.contains("\"threshold_profile_resolved\": \"p63_conservative_v1\""));
    assert!(campaign_report.contains("\"campaign_stability_status\":"));
    assert!(campaign_report.contains("\"machine_metadata\": {"));
    assert!(campaign_report.contains("\"ratio_effective_per_byte\": {"));
    assert!(campaign_report.contains("\"stddev\":"));
    assert!(campaign_report.contains("\"coefficient_of_variation\":"));
    assert_eq!(runs_jsonl.lines().count(), 3);
    assert!(runs_csv.starts_with("campaign_id,run_index,mode,threshold_profile"));
    assert!(runs_csv.contains("RECALIBRATE_P63_THRESHOLDS"));
    assert!(summary_md.contains("ASTRA-P63 Campaign Summary"));
    assert!(summary_md.contains("Decision:"));

    let _ = fs::remove_dir_all(export_dir);
}

#[test]
fn p63_campaign_comparison_reports_deltas_and_status() {
    let root = unique_export_dir();
    let smoke_dir = root.join("smoke");
    let standard_dir = root.join("standard");
    let smoke = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Smoke,
        2,
        P63ThresholdProfile::P63,
    )
    .expect("smoke report");
    let standard = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Standard,
        2,
        P63ThresholdProfile::P63,
    )
    .expect("standard report");
    write_p63_campaign_exports(&smoke, &smoke_dir).expect("write smoke exports");
    write_p63_campaign_exports(&standard, &standard_dir).expect("write standard exports");

    let comparison = p63_campaign_compare_json_files(
        smoke_dir
            .join("campaign_report.json")
            .to_str()
            .expect("smoke path"),
        standard_dir
            .join("campaign_report.json")
            .to_str()
            .expect("standard path"),
    )
    .expect("comparison json");

    assert!(comparison.contains("\"ratio_shift\":"));
    assert!(comparison.contains("\"bytes_shift\":"));
    assert!(comparison.contains("\"compatibility_status\": \"DIFFERENT_MODES_INFORMATIONAL\""));
    assert!(comparison.contains("\"intra_mode_status\": \"INTRA_MODE_NOT_ENOUGH_DATA\""));
    assert!(comparison.contains("\"comparison_decision\":"));
    assert!(comparison.contains("\"threshold_profile_a\": \"p63_conservative_v1\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn p63_campaign_comparison_marks_same_mode_comparable() {
    let root = unique_export_dir();
    let standard_a_dir = root.join("standard_a");
    let standard_b_dir = root.join("standard_b");
    let standard_a = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Standard,
        2,
        P63ThresholdProfile::P63,
    )
    .expect("standard A report");
    let standard_b = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Standard,
        2,
        P63ThresholdProfile::P63,
    )
    .expect("standard B report");
    write_p63_campaign_exports(&standard_a, &standard_a_dir).expect("write standard A exports");
    write_p63_campaign_exports(&standard_b, &standard_b_dir).expect("write standard B exports");

    let comparison = p63_campaign_compare_json_files(
        standard_a_dir
            .join("campaign_report.json")
            .to_str()
            .expect("standard A path"),
        standard_b_dir
            .join("campaign_report.json")
            .to_str()
            .expect("standard B path"),
    )
    .expect("comparison json");

    assert!(comparison.contains("\"compatibility_status\": \"SAME_MODE_COMPARABLE\""));
    assert!(comparison.contains("\"intra_mode_status\": \"INTRA_MODE_STABLE\""));
    assert!(comparison.contains("\"decision_compatibility\": \"SAME_DECISION\""));
    assert!(comparison.contains("\"comparison_decision\": \"COMPARE_P63_SAME_MODE_INFORMATIONAL\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn p63_campaign_registry_adds_campaigns_and_summarizes() {
    let root = unique_export_dir();
    let standard_a_dir = root.join("standard_a");
    let standard_b_dir = root.join("standard_b");
    let registry_path = root.join("registry.json");
    let standard_a = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Standard,
        2,
        P63ThresholdProfile::P63,
    )
    .expect("standard A report");
    let standard_b = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Standard,
        2,
        P63ThresholdProfile::P63,
    )
    .expect("standard B report");
    write_p63_campaign_exports(&standard_a, &standard_a_dir).expect("write standard A exports");
    write_p63_campaign_exports(&standard_b, &standard_b_dir).expect("write standard B exports");

    let registry_one = p63_campaign_register_json_file(
        standard_a_dir
            .join("campaign_report.json")
            .to_str()
            .expect("standard A path"),
        registry_path.to_str().expect("registry path"),
        "standard_local_001",
    )
    .expect("register first campaign");
    assert!(registry_one.contains("\"registry_version\": \"p63_registry_v1\""));
    assert!(registry_one.contains("\"astra_step\": \"P63\""));
    assert!(registry_one.contains("\"campaign_name\": \"standard_local_001\""));
    assert!(registry_one.contains("\"virtual_effective_units\":"));
    assert!(registry_one.contains("\"gain_vs_materialized\":"));

    let registry_two = p63_campaign_register_json_file(
        standard_b_dir
            .join("campaign_report.json")
            .to_str()
            .expect("standard B path"),
        registry_path.to_str().expect("registry path"),
        "standard_local_002",
    )
    .expect("register second campaign");
    assert!(registry_two.contains("\"campaign_name\": \"standard_local_001\""));
    assert!(registry_two.contains("\"campaign_name\": \"standard_local_002\""));

    let summary = p63_campaign_summary_json_file(registry_path.to_str().expect("registry path"))
        .expect("registry summary");
    assert!(summary.contains("\"registry_version\": \"p63_registry_v1\""));
    assert!(summary.contains("\"campaign_count\": 2"));
    assert!(summary.contains("\"modes\": ["));
    assert!(summary.contains("\"standard\""));
    assert!(summary.contains("\"virtual_effective_units\":"));
    assert!(summary.contains("\"gain_vs_materialized\":"));
    assert!(summary.contains("\"recommendation\":"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn p63_campaign_registry_cli_commands_work() {
    let root = unique_export_dir();
    let export_dir = root.join("standard");
    let registry_path = root.join("registry.json");
    let report = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Standard,
        2,
        P63ThresholdProfile::P63,
    )
    .expect("standard report");
    write_p63_campaign_exports(&report, &export_dir).expect("write standard exports");

    let register = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-campaign-register",
            export_dir
                .join("campaign_report.json")
                .to_str()
                .expect("campaign path"),
            "--registry",
            registry_path.to_str().expect("registry path"),
            "--name",
            "standard_local_001",
            "--format",
            "json",
        ])
        .output()
        .expect("run ratio-campaign-register");

    assert!(register.status.success());
    let register_stdout = String::from_utf8_lossy(&register.stdout);
    assert!(register_stdout.contains("\"registry_version\": \"p63_registry_v1\""));
    assert!(registry_path.exists());

    let summary = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-campaign-summary",
            registry_path.to_str().expect("registry path"),
            "--format",
            "json",
        ])
        .output()
        .expect("run ratio-campaign-summary");

    assert!(summary.status.success());
    let summary_stdout = String::from_utf8_lossy(&summary.stdout);
    assert!(summary_stdout.contains("\"campaign_count\": 1"));
    assert!(summary_stdout.contains("\"standard\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn p63_campaign_set_summary_handles_standard_campaigns_conservatively() {
    let root = unique_export_dir();
    let registry_path = root.join("registry.json");
    register_test_campaign(
        &root,
        &registry_path,
        "standard_set_001",
        WorkloadMode::Standard,
        2,
    );
    register_test_campaign(
        &root,
        &registry_path,
        "standard_set_002",
        WorkloadMode::Standard,
        2,
    );

    let summary = p63_campaign_set_summary_json_file(
        registry_path.to_str().expect("registry path"),
        "standard_test_set",
        Some(WorkloadMode::Standard),
        Some(P63ThresholdProfile::P63),
    )
    .expect("campaign set summary");

    assert!(summary.contains("\"campaign_set_version\": \"p63_campaign_set_v1\""));
    assert!(summary.contains("\"astra_step\": \"P63\""));
    assert!(summary.contains("\"set_name\": \"standard_test_set\""));
    assert!(summary.contains("\"mode\": \"standard\""));
    assert!(summary.contains("\"threshold_profile\": \"p63_conservative_v1\""));
    assert!(summary.contains("\"campaign_count\": 2"));
    assert!(summary.contains("\"total_runs\": 4"));
    assert!(summary.contains("\"virtual_declared_units\":"));
    assert!(summary.contains("\"virtual_effective_units\":"));
    assert!(summary.contains("\"total_persisted_bytes\":"));
    assert!(summary.contains("\"ratio_effective_per_byte\":"));
    assert!(summary.contains("\"gain_vs_materialized\":"));
    assert!(summary.contains("\"effective_gain_vs_materialized\":"));
    assert!(summary.contains("\"intra_mode_set_status\": \"CAMPAIGN_SET_NOT_ENOUGH_DATA\""));
    assert!(summary.contains("\"set_decision\": \"RECALIBRATE_P63_THRESHOLDS\""));
    assert!(!summary.contains("VALIDATE_P63_MEASURED_RATIO_CALIBRATION"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn p63_campaign_set_summary_detects_mixed_modes() {
    let root = unique_export_dir();
    let registry_path = root.join("registry.json");
    register_test_campaign(
        &root,
        &registry_path,
        "standard_set_001",
        WorkloadMode::Standard,
        2,
    );
    register_test_campaign(
        &root,
        &registry_path,
        "smoke_set_001",
        WorkloadMode::Smoke,
        2,
    );

    let summary = p63_campaign_set_summary_json_file(
        registry_path.to_str().expect("registry path"),
        "mixed_mode_test_set",
        None,
        None,
    )
    .expect("campaign set summary");

    assert!(summary.contains("\"intra_mode_set_status\": \"CAMPAIGN_SET_MIXED_MODES\""));
    assert!(summary.contains("\"set_decision\": \"RECALIBRATE_P63_WORKLOADS\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn p63_campaign_set_summary_detects_mixed_profiles() {
    let root = unique_export_dir();
    let registry_path = root.join("registry.json");
    register_test_campaign(
        &root,
        &registry_path,
        "standard_set_001",
        WorkloadMode::Standard,
        2,
    );
    register_test_campaign(
        &root,
        &registry_path,
        "standard_set_002",
        WorkloadMode::Standard,
        2,
    );
    let registry = fs::read_to_string(&registry_path).expect("registry json");
    let mixed_profile_registry = registry.replacen(
        "\"threshold_profile\": \"p63_conservative_v1\"",
        "\"threshold_profile\": \"p63_experimental_future\"",
        1,
    );
    fs::write(&registry_path, mixed_profile_registry).expect("write mixed profile registry");

    let summary = p63_campaign_set_summary_json_file(
        registry_path.to_str().expect("registry path"),
        "mixed_profile_test_set",
        None,
        None,
    )
    .expect("campaign set summary");

    assert!(summary.contains("\"intra_mode_set_status\": \"CAMPAIGN_SET_MIXED_PROFILES\""));
    assert!(summary.contains("\"set_decision\": \"RECALIBRATE_P63_WORKLOADS\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn p63_campaign_set_summary_cli_filters_standard_campaigns() {
    let root = unique_export_dir();
    let registry_path = root.join("registry.json");
    register_test_campaign(
        &root,
        &registry_path,
        "standard_set_001",
        WorkloadMode::Standard,
        2,
    );
    register_test_campaign(
        &root,
        &registry_path,
        "smoke_set_001",
        WorkloadMode::Smoke,
        2,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-campaign-set-summary",
            registry_path.to_str().expect("registry path"),
            "--mode",
            "standard",
            "--threshold-profile",
            "p63",
            "--format",
            "json",
            "--set-name",
            "standard_filtered_set",
        ])
        .output()
        .expect("run ratio-campaign-set-summary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"campaign_set_version\": \"p63_campaign_set_v1\""));
    assert!(stdout.contains("\"set_name\": \"standard_filtered_set\""));
    assert!(stdout.contains("\"mode\": \"standard\""));
    assert!(stdout.contains("\"campaign_count\": 1"));
    assert!(stdout.contains("\"set_decision\": \"RECALIBRATE_P63_THRESHOLDS\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn p63_ratio_real_cli_writes_export_dir() {
    let export_dir = unique_export_dir();
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-real",
            "examples/p53_strict.atlas",
            "--mode",
            "smoke",
            "--format",
            "json",
            "--runs",
            "2",
            "--export-dir",
            export_dir.to_str().expect("utf8 export dir"),
            "--threshold-profile",
            "p63",
        ])
        .output()
        .expect("run p63 ratio-real export cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_step\": \"P63\""));
    assert!(stdout.contains("\"threshold_profile\": \"p63\""));
    assert!(stdout.contains("\"decision\": \"RECALIBRATE_P63_THRESHOLDS\""));
    assert!(export_dir.join("campaign_report.json").exists());
    assert!(export_dir.join("runs.jsonl").exists());
    assert!(export_dir.join("runs.csv").exists());
    assert!(export_dir.join("summary.md").exists());

    let lines = fs::read_to_string(export_dir.join("runs.jsonl"))
        .expect("runs jsonl")
        .lines()
        .count();
    assert_eq!(lines, 2);

    let _ = fs::remove_dir_all(export_dir);
}

#[test]
fn p63_ratio_real_requires_threshold_profile_for_export_dir() {
    let export_dir = unique_export_dir();
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-real",
            "examples/p53_strict.atlas",
            "--mode",
            "smoke",
            "--format",
            "json",
            "--runs",
            "1",
            "--export-dir",
            export_dir.to_str().expect("utf8 export dir"),
        ])
        .output()
        .expect("run p63 ratio-real missing profile");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("ratio-real --export-dir requires --threshold-profile p63"));
    assert!(!export_dir.exists());
}

#[test]
fn p63_ratio_real_without_profile_keeps_p62_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-real",
            "examples/p53_strict.atlas",
            "--mode",
            "smoke",
            "--format",
            "json",
            "--runs",
            "1",
        ])
        .output()
        .expect("run p62 compatible ratio-real");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_iteration\": \"ASTRA-P62\""));
    assert!(!stdout.contains("\"astra_step\": \"P63\""));
}

fn unique_export_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("astra-p63-test-{}-{}", std::process::id(), nanos))
}

fn register_test_campaign(
    root: &std::path::Path,
    registry_path: &std::path::Path,
    campaign_name: &str,
    mode: WorkloadMode,
    runs: usize,
) {
    let export_dir = root.join(campaign_name);
    let report = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        mode,
        runs,
        P63ThresholdProfile::P63,
    )
    .expect("campaign report");
    write_p63_campaign_exports(&report, &export_dir).expect("write campaign exports");
    p63_campaign_register_json_file(
        export_dir
            .join("campaign_report.json")
            .to_str()
            .expect("campaign path"),
        registry_path.to_str().expect("registry path"),
        campaign_name,
    )
    .expect("register campaign");
}
