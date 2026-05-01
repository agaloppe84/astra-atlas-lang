use astra_atlas_lang::{
    p66_ratio_fibers_report_file, p67_fiber_calibration_report_file,
    write_p67_fiber_calibration_exports, P64WorkloadKind, P66JournalPolicy, P66RatioFibersOptions,
    P67AuditPolicy, P67CachePolicy, P67CompactionPolicy, P67ConfigDecision, P67Decision,
    P67FiberCalibrationOptions, P67FiberProjectionDepth, P67QueryLocality, WorkloadMode,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn calibration_options() -> P67FiberCalibrationOptions {
    P67FiberCalibrationOptions {
        workload: Some(P64WorkloadKind::RealishHybridFieldFixture),
        mode: WorkloadMode::Standard,
        runs: 3,
        queries: 120,
        radius_grid: vec![1, 3, 5],
        budget_grid: vec![524_288, 2_097_152],
        cache_grid: vec![P67CachePolicy::On, P67CachePolicy::Compact],
        journal_grid: vec![P66JournalPolicy::Lazy, P66JournalPolicy::Compact],
        audit_grid: vec![P67AuditPolicy::Minimal, P67AuditPolicy::Sampled],
        compaction_grid: vec![
            P67CompactionPolicy::Threshold,
            P67CompactionPolicy::Aggressive,
        ],
        query_locality_grid: vec![P67QueryLocality::Clustered, P67QueryLocality::Mixed],
        fiber_projection_grid: vec![
            P67FiberProjectionDepth::Shallow,
            P67FiberProjectionDepth::Medium,
        ],
    }
}

#[test]
fn p67_calibration_grid_builds() {
    let report =
        p67_fiber_calibration_report_file("examples/p53_strict.atlas", calibration_options())
            .expect("p67 calibration report");

    assert_eq!(report.astra_step, "P67");
    assert_eq!(
        report.calibration_version,
        "p67_address_fiber_overhead_calibration_v1"
    );
    assert_eq!(report.configuration_count, 3 * 2 * 2 * 2 * 2 * 2 * 2 * 2);
    assert!(report.best_by_ratio.is_some());
    assert!(report.best_by_overhead.is_some());
    assert!(report.best_by_net_gain.is_some());
    assert!(report.best_balanced.is_some());
}

#[test]
fn p67_config_contains_net_gain_and_overhead() {
    let report =
        p67_fiber_calibration_report_file("examples/p53_strict.atlas", calibration_options())
            .expect("p67 calibration report");
    let best = report.best_balanced.as_ref().expect("best balanced");

    assert!(best.address_fiber_net_gain > 0.0);
    assert!(best.avg_actor_overhead_ratio >= 0.0);
    assert!(best.fiber_ratio_effective_per_byte > 0.0);
    assert!(best.cache_hit_rate > 0.0);
    assert!(best.update_count > 0);
    assert!(best.audit_count > 0);
    assert!(best.compaction_count > 0);
}

#[test]
fn p67_pareto_front_is_nonempty() {
    let report =
        p67_fiber_calibration_report_file("examples/p53_strict.atlas", calibration_options())
            .expect("p67 calibration report");

    assert!(!report.pareto_front.is_empty());
    assert!(report
        .pareto_front
        .iter()
        .all(|config| config.conflicts == 0 && config.stale_reads == 0));
}

#[test]
fn p67_promotion_candidate_remains_conditional() {
    let report =
        p67_fiber_calibration_report_file("examples/p53_strict.atlas", calibration_options())
            .expect("p67 calibration report");

    assert!(matches!(
        report.decision,
        P67Decision::RecalibrateFiberOverhead | P67Decision::NoGoAddressFiberOverhead
    ));
    assert_ne!(
        report.decision.as_str(),
        "PROMOTE_P67_ADDRESS_FIBER_ARCHITECTURE"
    );
}

#[test]
fn p67_conflicts_or_stale_reads_zero_safety_score() {
    let mut options = calibration_options();
    options.cache_grid = vec![P67CachePolicy::Off];
    options.journal_grid = vec![P66JournalPolicy::Lazy];
    options.query_locality_grid = vec![P67QueryLocality::Random];
    options.radius_grid = vec![1];
    options.budget_grid = vec![1];

    let report = p67_fiber_calibration_report_file("examples/p53_strict.atlas", options)
        .expect("p67 calibration report");
    assert!(report.configurations.iter().any(|config| {
        config.decision == P67ConfigDecision::NoGoFiberSafety
            && config.balanced_score == 0.0
            && (config.conflicts > 0 || config.stale_reads > 0)
    }));
}

#[test]
fn p67_overhead_or_budget_refusals_block_promotion() {
    let mut options = calibration_options();
    options.budget_grid = vec![1];
    options.radius_grid = vec![5];
    options.cache_grid = vec![P67CachePolicy::On];
    options.journal_grid = vec![P66JournalPolicy::Eager];

    let report = p67_fiber_calibration_report_file("examples/p53_strict.atlas", options)
        .expect("p67 calibration report");
    assert!(report
        .configurations
        .iter()
        .all(|config| { !config.promotion_candidate || config.budget_refusal_rate < 0.02 }));
}

#[test]
fn p67_exports_are_written() {
    let report =
        p67_fiber_calibration_report_file("examples/p53_strict.atlas", calibration_options())
            .expect("p67 calibration report");
    let export_dir = temp_dir("exports");
    write_p67_fiber_calibration_exports(&report, &export_dir)
        .expect("write p67 calibration exports");

    assert!(export_dir
        .join("p67_fiber_calibration_report.json")
        .exists());
    assert!(export_dir.join("p67_fiber_calibration_runs.jsonl").exists());
    assert!(export_dir.join("p67_fiber_calibration_grid.csv").exists());
    assert!(export_dir.join("p67_fiber_calibration_summary.md").exists());

    let jsonl =
        fs::read_to_string(export_dir.join("p67_fiber_calibration_runs.jsonl")).expect("jsonl");
    assert_eq!(jsonl.lines().count(), report.configuration_count);
}

#[test]
fn p67_ratio_fibers_calibrate_cli_succeeds() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-fibers-calibrate",
            "examples/p53_strict.atlas",
            "--workload",
            "realish_hybrid_field_fixture",
            "--mode",
            "standard",
            "--runs",
            "3",
            "--queries",
            "120",
            "--radius-grid",
            "1,3",
            "--budget-grid",
            "524288,2097152",
            "--cache-grid",
            "on,compact",
            "--journal-grid",
            "lazy,compact",
            "--audit-grid",
            "minimal,sampled",
            "--compaction-grid",
            "threshold,aggressive",
            "--query-locality-grid",
            "clustered,mixed",
            "--fiber-projection-grid",
            "shallow,medium",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("run ratio-fibers-calibrate");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_step\": \"P67\""));
    assert!(stdout.contains("\"best_balanced\""));
    assert!(export_dir
        .join("p67_fiber_calibration_report.json")
        .exists());
}

#[test]
fn p67_ratio_fibers_calibrate_rejects_invalid_grid_value() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-fibers-calibrate",
            "examples/p53_strict.atlas",
            "--workload",
            "all",
            "--mode",
            "standard",
            "--runs",
            "3",
            "--queries",
            "120",
            "--radius-grid",
            "1",
            "--budget-grid",
            "524288",
            "--cache-grid",
            "turbo",
            "--journal-grid",
            "compact",
            "--audit-grid",
            "minimal",
            "--compaction-grid",
            "threshold",
            "--query-locality-grid",
            "clustered",
            "--fiber-projection-grid",
            "shallow",
            "--format",
            "json",
        ])
        .output()
        .expect("run invalid ratio-fibers-calibrate");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported cache policy"));
}

#[test]
fn p67_keeps_p66_ratio_fibers_working() {
    let report = p66_ratio_fibers_report_file(
        "examples/p53_strict.atlas",
        P66RatioFibersOptions {
            workload: Some(P64WorkloadKind::RealishHybridFieldFixture),
            fiber_strategy: None,
            mode: WorkloadMode::Standard,
            runs: 3,
            queries: 120,
            neighborhood_radius: 3,
            budget_bytes: 2_097_152,
            cache_enabled: true,
            journal_policy: P66JournalPolicy::Compact,
            update_rate: None,
            audit_rate: None,
        },
    )
    .expect("p66 report");

    assert_eq!(report.astra_step, "P66");
    assert!(!report.entries.is_empty());
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "astra-p67-test-{}-{}-{}",
        std::process::id(),
        label,
        nanos
    ))
}
