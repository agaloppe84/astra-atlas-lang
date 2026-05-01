use astra_atlas_lang::{
    p67_fiber_calibration_report_file, p68_promotion_report_file, write_p68_promotion_exports,
    P64WorkloadKind, P66JournalPolicy, P67AuditPolicy, P67CachePolicy, P67CompactionPolicy,
    P67FiberCalibrationOptions, P67FiberProjectionDepth, P67QueryLocality, P68Candidate,
    P68Decision, P68GateStatus, P68PairingStatus, P68PhaseStatus, P68PromotionOptions,
    P68PromotionThresholds, P68StressStatus, PromotionEvaluator, WorkloadMode,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn p68_options() -> P68PromotionOptions {
    P68PromotionOptions {
        run_ablations: true,
        run_stress: true,
        phase_map: true,
        strict: true,
    }
}

fn synthetic_candidate(mode: &str) -> P68Candidate {
    P68Candidate {
        config_id: format!("realish_hybrid_field_fixture:{}:synthetic", mode),
        workload: "realish_hybrid_field_fixture".to_string(),
        mode: mode.to_string(),
        radius: if mode == "standard" { 1 } else { 2 },
        budget_bytes: 4_194_304,
        cache_policy: "compact".to_string(),
        journal_policy: "compact".to_string(),
        audit_policy: "minimal".to_string(),
        compaction_policy: "threshold".to_string(),
        query_locality: "clustered".to_string(),
        fiber_projection_depth: "shallow".to_string(),
        metadata_policy: "standard".to_string(),
        promotion_candidate: true,
        address_fiber_net_gain: if mode == "standard" { 17.0 } else { 13.0 },
        avg_actor_overhead_ratio: if mode == "standard" { 0.12 } else { 0.13 },
        fiber_ratio_effective_per_byte: if mode == "standard" { 30.0 } else { 3.2 },
        cache_hit_rate: 0.72,
        update_count: 10,
        audit_count: 5,
        compaction_count: 2,
        conflicts: 0,
        stale_reads: 0,
        budget_refusals: 0,
        budget_refusal_rate: 0.0,
        bytes_per_query: 64.0,
    }
}

#[test]
fn p68_promotion_report_builds_from_p67_candidates() {
    let report = p68_promotion_report_file("examples/p53_strict.atlas", p68_options())
        .expect("p68 promotion report");

    assert_eq!(report.astra_step, "P68");
    assert_eq!(
        report.promotion_evaluator_version,
        "p68_promotion_evaluator_v1"
    );
    assert_eq!(
        report.paired_gate_result.standard_gate_status,
        P68GateStatus::Pass
    );
    assert_eq!(
        report.paired_gate_result.ambitious_gate_status,
        P68GateStatus::Pass
    );
    assert_eq!(
        report.paired_gate_result.pairing_status,
        P68PairingStatus::Compatible
    );
}

#[test]
fn p68_promotes_only_when_standard_and_ambitious_pass() {
    let evaluator = PromotionEvaluator::new(P68PromotionThresholds::default());
    let standard = synthetic_candidate("standard");
    let ambitious = synthetic_candidate("ambitious");
    let result = evaluator.evaluate(&standard, &ambitious);

    assert_eq!(
        result.promotion_decision,
        P68Decision::PromoteAddressFiberArchitecture
    );

    let mut weak_ambitious = ambitious.clone();
    weak_ambitious.address_fiber_net_gain = 1.0;
    weak_ambitious.promotion_candidate = false;
    let weak_result = evaluator.evaluate(&standard, &weak_ambitious);
    assert_eq!(
        weak_result.promotion_decision,
        P68Decision::RecalibratePromotionGate
    );
}

#[test]
fn p68_refuses_overhead_conflicts_stale_reads_and_budget_refusals() {
    let evaluator = PromotionEvaluator::new(P68PromotionThresholds::default());
    let standard = synthetic_candidate("standard");
    let ambitious = synthetic_candidate("ambitious");

    let mut high_overhead = standard.clone();
    high_overhead.avg_actor_overhead_ratio = 0.31;
    high_overhead.promotion_candidate = false;
    assert_eq!(
        evaluator
            .evaluate(&high_overhead, &ambitious)
            .promotion_decision,
        P68Decision::RecalibratePromotionGate
    );

    let mut conflict = standard.clone();
    conflict.conflicts = 1;
    assert_eq!(
        evaluator.evaluate(&conflict, &ambitious).promotion_decision,
        P68Decision::NoGoAddressFiberArchitecture
    );

    let mut stale = standard.clone();
    stale.stale_reads = 1;
    assert_eq!(
        evaluator.evaluate(&stale, &ambitious).promotion_decision,
        P68Decision::NoGoAddressFiberArchitecture
    );

    let mut budget = standard;
    budget.budget_refusal_rate = 0.20;
    budget.budget_refusals = 200;
    assert_eq!(
        evaluator.evaluate(&budget, &ambitious).promotion_decision,
        P68Decision::RecalibratePromotionGate
    );
}

#[test]
fn p68_ablations_stress_phase_map_and_manifest_are_structured() {
    let report = p68_promotion_report_file("examples/p53_strict.atlas", p68_options())
        .expect("p68 promotion report");

    assert!(!report.ablations.is_empty());
    assert!(report.ablation_summary.cache_contribution > 0.0);
    assert!(report
        .stress_scenarios
        .iter()
        .any(|stress| stress.stress_status == P68StressStatus::Robust));
    assert!(report
        .stress_scenarios
        .iter()
        .any(|stress| stress.stress_status == P68StressStatus::Warn));
    assert!(report
        .stress_scenarios
        .iter()
        .any(|stress| stress.stress_status == P68StressStatus::Unstable));
    assert!(report
        .stress_scenarios
        .iter()
        .any(|stress| stress.stress_status == P68StressStatus::NoGo));
    assert!(report
        .phase_map_cells
        .iter()
        .any(|cell| cell.phase_status == P68PhaseStatus::GreenPromotable));
    assert!(report
        .phase_map_cells
        .iter()
        .any(|cell| cell.phase_status == P68PhaseStatus::YellowRecalibrate));
    assert!(report
        .phase_map_cells
        .iter()
        .any(|cell| cell.phase_status == P68PhaseStatus::RedNoGo));
    assert_eq!(
        report.architecture_manifest.architecture_id,
        "address_fiber_actor_managed_v1"
    );
}

#[test]
fn p68_historical_comparison_contains_p64_to_p68() {
    let report = p68_promotion_report_file("examples/p53_strict.atlas", p68_options())
        .expect("p68 promotion report");
    let steps: Vec<_> = report
        .historical_comparison
        .iter()
        .map(|entry| entry.step.as_str())
        .collect();

    for step in ["P64", "P65", "P66", "P67", "P68"] {
        assert!(steps.contains(&step));
    }
}

#[test]
fn p68_exports_are_written() {
    let report = p68_promotion_report_file("examples/p53_strict.atlas", p68_options())
        .expect("p68 promotion report");
    let export_dir = temp_dir("exports");
    write_p68_promotion_exports(&report, &export_dir).expect("write p68 exports");

    assert!(export_dir.join("p68_promotion_report.json").exists());
    assert!(export_dir.join("p68_ablations.jsonl").exists());
    assert!(export_dir.join("p68_stress.jsonl").exists());
    assert!(export_dir.join("p68_phase_map.csv").exists());
    assert!(export_dir.join("p68_summary.md").exists());
    assert!(export_dir
        .join("address_fiber_architecture_manifest.json")
        .exists());

    let ablations = fs::read_to_string(export_dir.join("p68_ablations.jsonl")).expect("ablations");
    let stress = fs::read_to_string(export_dir.join("p68_stress.jsonl")).expect("stress");
    assert_eq!(ablations.lines().count(), report.ablations.len());
    assert_eq!(stress.lines().count(), report.stress_scenarios.len());
}

#[test]
fn p68_ratio_fibers_promote_cli_succeeds() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-fibers-promote",
            "examples/p53_strict.atlas",
            "--run-ablations",
            "--run-stress",
            "--phase-map",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("run ratio-fibers-promote");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_step\": \"P68\""));
    assert!(stdout.contains("PROMOTE_P68_ADDRESS_FIBER_ARCHITECTURE"));
    assert!(export_dir.join("p68_promotion_report.json").exists());
}

#[test]
fn p68_ratio_fibers_promote_rejects_invalid_mode_pair() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-fibers-promote",
            "examples/p53_strict.atlas",
            "--mode-pair",
            "smoke,standard",
            "--format",
            "json",
        ])
        .output()
        .expect("run invalid ratio-fibers-promote");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("standard,ambitious"));
}

#[test]
fn p68_keeps_p67_calibration_working() {
    let report = p67_fiber_calibration_report_file(
        "examples/p53_strict.atlas",
        P67FiberCalibrationOptions {
            workload: Some(P64WorkloadKind::RealishHybridFieldFixture),
            mode: WorkloadMode::Standard,
            runs: 3,
            queries: 120,
            radius_grid: vec![1, 3],
            budget_grid: vec![524_288, 2_097_152],
            cache_grid: vec![P67CachePolicy::On, P67CachePolicy::Compact],
            journal_grid: vec![P66JournalPolicy::Lazy, P66JournalPolicy::Compact],
            audit_grid: vec![P67AuditPolicy::Minimal],
            compaction_grid: vec![P67CompactionPolicy::Threshold],
            query_locality_grid: vec![P67QueryLocality::Clustered],
            fiber_projection_grid: vec![P67FiberProjectionDepth::Shallow],
        },
    )
    .expect("p67 report");

    assert_eq!(report.astra_step, "P67");
    assert!(!report.configurations.is_empty());
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "astra-p68-test-{}-{}-{}",
        std::process::id(),
        label,
        nanos
    ))
}
