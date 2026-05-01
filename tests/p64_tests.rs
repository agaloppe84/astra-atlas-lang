use astra_atlas_lang::{
    p63_campaign_report_file_with_runs, p64_ratio_realish_report_file, p64_report_json,
    write_p64_campaign_exports, P63ThresholdProfile, P64Decision, P64GenerationPolicy,
    P64PolicyDecision, P64RatioRealishOptions, P64WorkloadKind, WorkloadMode,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn base_options(
    workload: Option<P64WorkloadKind>,
    policy: Option<P64GenerationPolicy>,
) -> P64RatioRealishOptions {
    P64RatioRealishOptions {
        workload,
        policy,
        mode: WorkloadMode::Smoke,
        runs: 2,
        queries: 64,
        neighborhood_radius: 3,
    }
}

#[test]
fn p64_workloads_and_policies_exist() {
    assert_eq!(P64WorkloadKind::all().len(), 4);
    assert_eq!(P64GenerationPolicy::all().len(), 3);
    assert_eq!(
        P64WorkloadKind::from_str("realish_log_events"),
        Some(P64WorkloadKind::RealishLogEvents)
    );
    assert_eq!(
        P64GenerationPolicy::from_str("address-local"),
        Some(P64GenerationPolicy::AddressLocalGeneration)
    );
}

#[test]
fn p64_address_local_does_not_materialize_full_virtual_space() {
    let report = p64_ratio_realish_report_file(
        "examples/p53_strict.atlas",
        base_options(None, Some(P64GenerationPolicy::AddressLocalGeneration)),
    )
    .expect("p64 address-local report");

    assert_eq!(report.entries.len(), 4);
    for entry in &report.entries {
        assert_eq!(entry.generation_policy, "address_local_generation");
        assert!(entry.virtual_generated_units < entry.virtual_declared_units);
        assert!(entry.locality_selectivity < 1.0);
        assert!(entry.local_generated_units_per_query > 0);
        assert!(entry.cache_enabled);
        assert!(entry.cache_hit_rate.unwrap_or(0.0) > 0.0);
    }
}

#[test]
fn p64_full_materialization_is_baseline_full_generation() {
    let report = p64_ratio_realish_report_file(
        "examples/p53_strict.atlas",
        base_options(None, Some(P64GenerationPolicy::FullMaterialization)),
    )
    .expect("p64 full report");

    for entry in &report.entries {
        assert_eq!(entry.generation_policy, "full_materialization");
        assert_eq!(entry.virtual_generated_units, entry.virtual_declared_units);
        assert_eq!(entry.locality_selectivity, 1.0);
        assert!(entry.total_persisted_bytes > 0);
    }
}

#[test]
fn p64_ratios_gains_and_local_success_metrics_are_present() {
    let report = p64_ratio_realish_report_file(
        "examples/p53_strict.atlas",
        base_options(
            Some(P64WorkloadKind::RealishSparseCsv),
            Some(P64GenerationPolicy::AddressLocalGeneration),
        ),
    )
    .expect("p64 report");
    let entry = report.entries.first().expect("entry");

    assert!(entry.ratio_effective_per_byte > 0.0);
    assert!(entry.effective_gain_vs_materialized > 0.0);
    assert!(entry.generated_gain_vs_materialized > 0.0);
    assert!(entry.local_generation_gain_vs_full_materialization > 0.0);
    assert!(entry.local_read_success_rate > 0.0);
    assert!(entry.local_update_success_rate > 0.0);
    assert!(entry.audit_success_rate > 0.0);
    assert!(entry.runtime_observed_ns_median > 0);
    assert_eq!(entry.unsafe_local_generation_count, 0);
    assert!(entry.guard_refused_count > 0);
}

#[test]
fn p64_policy_comparison_is_structured_and_conservative() {
    let report =
        p64_ratio_realish_report_file("examples/p53_strict.atlas", base_options(None, None))
            .expect("p64 comparison report");

    assert_eq!(report.comparisons.len(), 4);
    assert!(matches!(
        report.decision,
        P64Decision::RecalibrateAddressLocalRatioModel | P64Decision::NoGoAddressLocality
    ));
    assert_ne!(report.decision.as_str(), "VALIDATE_P64");
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.best_policy == "address_local_generation"
            && matches!(
                comparison.decision,
                P64PolicyDecision::AddressLocalPromising | P64PolicyDecision::AddressLocalStrong
            )
    }));
}

#[test]
fn p64_json_contains_required_schema_fields() {
    let report = p64_ratio_realish_report_file(
        "examples/p53_strict.atlas",
        base_options(
            Some(P64WorkloadKind::RealishJsonRecords),
            Some(P64GenerationPolicy::AddressLocalGeneration),
        ),
    )
    .expect("p64 report");
    let json = p64_report_json(&report);

    assert!(json.contains("\"astra_step\": \"P64\""));
    assert!(json.contains("\"virtual_declared_units\":"));
    assert!(json.contains("\"virtual_generated_units\":"));
    assert!(json.contains("\"virtual_effective_units\":"));
    assert!(json.contains("\"address_local_summary\":"));
    assert!(json.contains("\"campaign_set_version\": \"p64_policy_comparison_set_v1\""));
    assert!(json.contains("\"ratio_effective_per_byte\":"));
    assert!(json.contains("\"effective_gain_vs_materialized\":"));
    assert!(json.contains("\"locality_selectivity\":"));
    assert!(json.contains("\"policy_comparison\":"));
}

#[test]
fn p64_exports_are_written() {
    let export_dir = unique_export_dir();
    let report = p64_ratio_realish_report_file(
        "examples/p53_strict.atlas",
        base_options(
            Some(P64WorkloadKind::RealishLogEvents),
            Some(P64GenerationPolicy::AddressLocalGeneration),
        ),
    )
    .expect("p64 report");

    write_p64_campaign_exports(&report, &export_dir).expect("write p64 exports");

    assert!(export_dir.join("p64_campaign_report.json").exists());
    assert!(export_dir.join("p64_runs.jsonl").exists());
    assert!(export_dir.join("p64_summary.md").exists());
    assert!(export_dir.join("p64_workload_metrics.csv").exists());
    assert_eq!(
        fs::read_to_string(export_dir.join("p64_runs.jsonl"))
            .expect("runs")
            .lines()
            .count(),
        2
    );

    let _ = fs::remove_dir_all(export_dir);
}

#[test]
fn p64_ratio_realish_cli_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-realish",
            "examples/p53_strict.atlas",
            "--workload",
            "realish_log_events",
            "--policy",
            "address-local",
            "--mode",
            "smoke",
            "--runs",
            "1",
            "--queries",
            "16",
            "--neighborhood-radius",
            "2",
            "--format",
            "json",
        ])
        .output()
        .expect("run ratio-realish");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_step\": \"P64\""));
    assert!(stdout.contains("\"generation_policy\": \"address-local\""));
}

#[test]
fn p64_ratio_realish_rejects_invalid_policy() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-realish",
            "examples/p53_strict.atlas",
            "--workload",
            "realish_log_events",
            "--policy",
            "unknown",
            "--mode",
            "smoke",
            "--runs",
            "1",
            "--queries",
            "16",
            "--neighborhood-radius",
            "2",
            "--format",
            "json",
        ])
        .output()
        .expect("run ratio-realish invalid");

    assert!(!output.status.success());
}

#[test]
fn p64_keeps_p63_report_path_working() {
    let report = p63_campaign_report_file_with_runs(
        "examples/p53_strict.atlas",
        WorkloadMode::Smoke,
        1,
        P63ThresholdProfile::P63,
    )
    .expect("p63 report");

    assert_eq!(report.astra_step, "P63");
    assert_eq!(report.decision.as_str(), "RECALIBRATE_P63_THRESHOLDS");
}

fn unique_export_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("astra-p64-test-{}-{}", std::process::id(), nanos))
}
