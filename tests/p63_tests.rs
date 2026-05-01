use astra_atlas_lang::{
    p63_campaign_compare_json_files, p63_campaign_report_file_with_runs,
    write_p63_campaign_exports, P63Decision, P63StabilityStatus, P63ThresholdProfile, WorkloadMode,
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
    assert_eq!(report.decision.as_str(), "RECALIBRATE_P63_THRESHOLDS");
    assert!(matches!(
        report.campaign_stability_status,
        P63StabilityStatus::Stable | P63StabilityStatus::Warn
    ));
    assert!(report
        .decision_reasons
        .iter()
        .any(|reason| reason.contains("threshold calibration")));
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
    assert!(comparison.contains("\"compatibility_status\": \"DIFFERENT_MODES\""));
    assert!(comparison.contains("\"comparison_decision\":"));
    assert!(comparison.contains("\"threshold_profile_a\": \"p63_conservative_v1\""));

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
