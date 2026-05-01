use astra_atlas_lang::{
    p65_ratio_actors_report_file, p66_ratio_fibers_report_file, write_p66_fiber_campaign_exports,
    FiberGenerationStrategy, P64WorkloadKind, P65ActorStrategy, P65RatioActorsOptions, P66Decision,
    P66FiberKind, P66FiberWorkloadDecision, P66JournalPolicy, P66RatioFibersOptions, WorkloadMode,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn p66_options(
    workload: Option<P64WorkloadKind>,
    strategy: Option<FiberGenerationStrategy>,
) -> P66RatioFibersOptions {
    P66RatioFibersOptions {
        workload,
        fiber_strategy: strategy,
        mode: WorkloadMode::Smoke,
        runs: 1,
        queries: 32,
        neighborhood_radius: 3,
        budget_bytes: 2_097_152,
        cache_enabled: true,
        journal_policy: P66JournalPolicy::Compact,
        update_rate: None,
        audit_rate: None,
    }
}

#[test]
fn p66_fiber_kinds_and_strategies_exist() {
    assert_eq!(P66FiberKind::all().len(), 4);
    assert_eq!(FiberGenerationStrategy::all().len(), 4);
    assert_eq!(
        FiberGenerationStrategy::from_str("point-fiber"),
        Some(FiberGenerationStrategy::PointFiberOnly)
    );
    assert_eq!(
        FiberGenerationStrategy::from_str("neighborhood-fiber"),
        Some(FiberGenerationStrategy::NeighborhoodFiber)
    );
    assert_eq!(
        FiberGenerationStrategy::from_str("actor-fiber"),
        Some(FiberGenerationStrategy::ActorManagedFiber)
    );
    assert_eq!(
        FiberGenerationStrategy::from_str("actor-neighborhood-fiber"),
        Some(FiberGenerationStrategy::ActorManagedNeighborhoodFiber)
    );
}

#[test]
fn p66_address_fiber_exposes_declared_generated_and_effective_units() {
    let report = p66_ratio_fibers_report_file(
        "examples/p53_strict.atlas",
        p66_options(
            Some(P64WorkloadKind::RealishLogEvents),
            Some(FiberGenerationStrategy::PointFiberOnly),
        ),
    )
    .expect("p66 report");
    let entry = report.entries.first().expect("entry");

    assert_eq!(entry.address_fiber.fiber_kind, "log_event_fiber");
    assert!(entry.address_fiber.declared_units > 0);
    assert!(entry.address_fiber.generated_units > 0);
    assert!(entry.address_fiber.effective_units > 0);
    assert!(entry.fiber_declared_units >= entry.fiber_generated_units);
    assert!(entry.fiber_effective_units <= entry.fiber_declared_units);
}

#[test]
fn p66_point_fiber_generates_no_more_than_neighborhood_fiber() {
    let report = p66_ratio_fibers_report_file(
        "examples/p53_strict.atlas",
        p66_options(Some(P64WorkloadKind::RealishSparseCsv), None),
    )
    .expect("p66 report");
    let point = report
        .entries
        .iter()
        .find(|entry| entry.fiber_strategy == "point_fiber_only")
        .expect("point fiber");
    let neighborhood = report
        .entries
        .iter()
        .find(|entry| entry.fiber_strategy == "neighborhood_fiber")
        .expect("neighborhood fiber");

    assert!(point.fiber_generated_units <= neighborhood.fiber_generated_units);
    assert!(point.fiber_selectivity <= neighborhood.fiber_selectivity);
}

#[test]
fn p66_actor_managed_fiber_counts_actor_overhead() {
    let report = p66_ratio_fibers_report_file(
        "examples/p53_strict.atlas",
        p66_options(
            Some(P64WorkloadKind::RealishJsonRecords),
            Some(FiberGenerationStrategy::ActorManagedFiber),
        ),
    )
    .expect("p66 report");
    let entry = report.entries.first().expect("entry");
    let actor = entry
        .address_fiber
        .actor_binding
        .as_ref()
        .expect("actor binding");

    assert!(entry.fiber_actor_bytes > 0);
    assert!(entry.actor_overhead_ratio > 0.0);
    assert!(actor.total_actor_overhead_bytes > 0);
    assert_eq!(actor.conflict_count, entry.conflicts);
    assert_eq!(actor.stale_read_count, entry.stale_reads);
}

#[test]
fn p66_ratios_gains_update_audit_and_compaction_are_present() {
    let report = p66_ratio_fibers_report_file(
        "examples/p53_strict.atlas",
        p66_options(
            Some(P64WorkloadKind::RealishHybridFieldFixture),
            Some(FiberGenerationStrategy::ActorManagedNeighborhoodFiber),
        ),
    )
    .expect("p66 report");
    let entry = report.entries.first().expect("entry");

    assert!(entry.fiber_ratio_effective_per_byte > 0.0);
    assert!(entry.fiber_gain_vs_materialized > 0.0);
    assert!(entry.address_fiber_net_gain.unwrap_or(0.0) > 0.0);
    assert!(entry.update_count > 0);
    assert!(entry.audit_count > 0);
    assert!(entry.compaction_count > 0);
    assert_eq!(entry.fiber_update_success_rate, 1.0);
    assert_eq!(entry.fiber_audit_success_rate, 1.0);
}

#[test]
fn p66_unsafe_or_underbudgeted_fiber_is_refused_or_no_go() {
    let report = p66_ratio_fibers_report_file(
        "examples/p53_strict.atlas",
        P66RatioFibersOptions {
            workload: Some(P64WorkloadKind::RealishHybridFieldFixture),
            fiber_strategy: Some(FiberGenerationStrategy::ActorManagedNeighborhoodFiber),
            mode: WorkloadMode::Smoke,
            runs: 1,
            queries: 64,
            neighborhood_radius: 5,
            budget_bytes: 1,
            cache_enabled: false,
            journal_policy: P66JournalPolicy::Lazy,
            update_rate: None,
            audit_rate: None,
        },
    )
    .expect("p66 report");
    let entry = report.entries.first().expect("entry");

    assert!(
        entry.decision == P66FiberWorkloadDecision::NoGoFiberUnsafe
            || entry.budget_refusals > 2
            || entry.stale_reads > 0
    );
}

#[test]
fn p66_policy_comparison_is_structured_and_conservative() {
    let report = p66_ratio_fibers_report_file("examples/p53_strict.atlas", p66_options(None, None))
        .expect("p66 report");

    assert_eq!(report.comparisons.len(), 4);
    assert!(matches!(
        report.decision,
        P66Decision::RecalibrateAddressFiberModel | P66Decision::NoGoAddressFiber
    ));
    assert_ne!(
        report.decision.as_str(),
        "PROMOTE_P66_ADDRESS_FIBER_ARCHITECTURE"
    );
    assert!(report
        .comparisons
        .iter()
        .all(|comparison| !comparison.best_fiber_strategy.is_empty()));
}

#[test]
fn p66_exports_are_written() {
    let export_dir = unique_export_dir();
    let report = p66_ratio_fibers_report_file(
        "examples/p53_strict.atlas",
        p66_options(
            Some(P64WorkloadKind::RealishLogEvents),
            Some(FiberGenerationStrategy::ActorManagedFiber),
        ),
    )
    .expect("p66 report");

    write_p66_fiber_campaign_exports(&report, &export_dir).expect("write p66 exports");

    assert!(export_dir.join("p66_fiber_campaign_report.json").exists());
    assert!(export_dir.join("p66_fiber_runs.jsonl").exists());
    assert!(export_dir.join("p66_fiber_summary.md").exists());
    assert!(export_dir.join("p66_fiber_metrics.csv").exists());
    assert_eq!(
        fs::read_to_string(export_dir.join("p66_fiber_runs.jsonl"))
            .expect("runs")
            .lines()
            .count(),
        1
    );

    let _ = fs::remove_dir_all(export_dir);
}

#[test]
fn p66_ratio_fibers_cli_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-fibers",
            "examples/p53_strict.atlas",
            "--workload",
            "realish_log_events",
            "--fiber-strategy",
            "actor-fiber",
            "--mode",
            "smoke",
            "--runs",
            "1",
            "--queries",
            "16",
            "--neighborhood-radius",
            "2",
            "--budget-bytes",
            "1048576",
            "--cache",
            "on",
            "--journal",
            "compact",
            "--format",
            "json",
        ])
        .output()
        .expect("run ratio-fibers");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_step\": \"P66\""));
    assert!(stdout.contains("\"fiber_strategy\": \"actor-fiber\""));
}

#[test]
fn p66_ratio_fibers_rejects_invalid_strategy() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-fibers",
            "examples/p53_strict.atlas",
            "--workload",
            "realish_log_events",
            "--fiber-strategy",
            "unknown",
            "--mode",
            "smoke",
            "--runs",
            "1",
            "--queries",
            "16",
            "--neighborhood-radius",
            "2",
            "--budget-bytes",
            "1048576",
            "--journal",
            "compact",
            "--format",
            "json",
        ])
        .output()
        .expect("run invalid ratio-fibers");

    assert!(!output.status.success());
}

#[test]
fn p66_keeps_p65_actor_path_working() {
    let report = p65_ratio_actors_report_file(
        "examples/p53_strict.atlas",
        P65RatioActorsOptions {
            workload: Some(P64WorkloadKind::RealishLogEvents),
            actor_strategy: Some(P65ActorStrategy::SingleLocalActor),
            mode: WorkloadMode::Smoke,
            runs: 1,
            queries: 16,
            neighborhood_radius: 2,
            budget_bytes: 1_048_576,
            cache_enabled: true,
        },
    )
    .expect("p65 report");

    assert_eq!(report.astra_step, "P65");
}

fn unique_export_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("astra-p66-test-{}-{}", std::process::id(), nanos))
}
