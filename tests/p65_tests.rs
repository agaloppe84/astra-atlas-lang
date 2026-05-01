use astra_atlas_lang::{
    p64_ratio_realish_report_file, p65_actor_calibration_report_file, p65_ratio_actors_report_file,
    p65_report_json, write_p65_actor_calibration_exports, write_p65_actor_campaign_exports,
    P64GenerationPolicy, P64RatioRealishOptions, P64WorkloadKind, P65ActorCalibrationOptions,
    P65ActorStrategy, P65CalibrationDecision, P65Decision, P65JournalPolicy, P65QueryLocality,
    P65RatioActorsOptions, WorkloadMode,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn base_options(
    workload: Option<P64WorkloadKind>,
    actor_strategy: Option<P65ActorStrategy>,
) -> P65RatioActorsOptions {
    P65RatioActorsOptions {
        workload,
        actor_strategy,
        mode: WorkloadMode::Smoke,
        runs: 2,
        queries: 64,
        neighborhood_radius: 3,
        budget_bytes: 1_048_576,
        cache_enabled: true,
    }
}

fn calibration_options() -> P65ActorCalibrationOptions {
    P65ActorCalibrationOptions {
        workload: Some(P64WorkloadKind::RealishLogEvents),
        mode: WorkloadMode::Smoke,
        runs: 1,
        queries: 32,
        radius_grid: vec![1, 2],
        budget_grid: vec![262_144, 1_048_576],
        cache_grid: vec![false, true],
        journal_grid: vec![P65JournalPolicy::Lazy, P65JournalPolicy::Compact],
        query_locality_grid: vec![P65QueryLocality::Clustered, P65QueryLocality::Random],
    }
}

#[test]
fn p65_actor_strategies_exist() {
    assert_eq!(P65ActorStrategy::all().len(), 4);
    assert_eq!(
        P65ActorStrategy::from_str("no-actor"),
        Some(P65ActorStrategy::NoActorAddressLocal)
    );
    assert_eq!(
        P65ActorStrategy::from_str("single-local"),
        Some(P65ActorStrategy::SingleLocalActor)
    );
    assert_eq!(
        P65ActorStrategy::from_str("specialized-crud"),
        Some(P65ActorStrategy::SpecializedCrudActors)
    );
    assert_eq!(
        P65ActorStrategy::from_str("over-agentic"),
        Some(P65ActorStrategy::OverAgenticStress)
    );
}

#[test]
fn p65_no_actor_is_address_local_baseline() {
    let report = p65_ratio_actors_report_file(
        "examples/p53_strict.atlas",
        base_options(
            Some(P64WorkloadKind::RealishLogEvents),
            Some(P65ActorStrategy::NoActorAddressLocal),
        ),
    )
    .expect("p65 no actor report");
    let entry = report.entries.first().expect("entry");

    assert_eq!(entry.actor_strategy, "no_actor_address_local");
    assert_eq!(entry.actor_count, 0);
    assert_eq!(entry.total_actor_overhead_bytes, 0);
    assert_eq!(
        entry.total_persisted_bytes,
        entry.baseline_no_actor_persisted_bytes
    );
    assert!(entry.local_actor.is_none());
}

#[test]
fn p65_single_local_actor_exposes_budget_and_overhead() {
    let report = p65_ratio_actors_report_file(
        "examples/p53_strict.atlas",
        base_options(
            Some(P64WorkloadKind::RealishSparseCsv),
            Some(P65ActorStrategy::SingleLocalActor),
        ),
    )
    .expect("p65 single actor report");
    let entry = report.entries.first().expect("entry");
    let actor = entry.local_actor.as_ref().expect("local actor");

    assert!(entry.actor_count > 0);
    assert!(entry.total_actor_overhead_bytes > 0);
    assert!(entry.actor_overhead_ratio > 0.0);
    assert_eq!(actor.budget_bytes, 1_048_576);
    assert!(actor.cache_enabled);
    assert!(actor.journal_enabled);
    assert!(actor.audit_enabled);
    assert!(actor.queue_bytes > 0);
    assert!(entry.cache_hit_rate.unwrap_or(0.0) > 0.0);
}

#[test]
fn p65_specialized_crud_actors_count_coordination() {
    let report = p65_ratio_actors_report_file(
        "examples/p53_strict.atlas",
        base_options(
            Some(P64WorkloadKind::RealishJsonRecords),
            Some(P65ActorStrategy::SpecializedCrudActors),
        ),
    )
    .expect("p65 specialized report");
    let entry = report.entries.first().expect("entry");

    assert!(entry.actor_count >= 4);
    assert!(entry.coordination_events > 0);
    assert!(entry.actor_coordination_bytes > 0);
    assert!(entry.conflict_count <= entry.actor_count);
    assert!(entry.stale_read_count <= entry.actor_count);
}

#[test]
fn p65_over_agentic_stress_is_more_expensive_or_warns() {
    let baseline = p65_ratio_actors_report_file(
        "examples/p53_strict.atlas",
        base_options(
            Some(P64WorkloadKind::RealishHybridFieldFixture),
            Some(P65ActorStrategy::NoActorAddressLocal),
        ),
    )
    .expect("p65 baseline");
    let stress = p65_ratio_actors_report_file(
        "examples/p53_strict.atlas",
        base_options(
            Some(P64WorkloadKind::RealishHybridFieldFixture),
            Some(P65ActorStrategy::OverAgenticStress),
        ),
    )
    .expect("p65 stress");

    let baseline_entry = baseline.entries.first().expect("baseline entry");
    let stress_entry = stress.entries.first().expect("stress entry");
    assert!(stress_entry.total_actor_overhead_bytes > baseline_entry.total_actor_overhead_bytes);
    assert!(stress_entry.total_persisted_bytes > baseline_entry.total_persisted_bytes);
    assert!(stress_entry.conflict_count > 0 || stress_entry.stale_read_count > 0);
}

#[test]
fn p65_actor_net_gain_and_deltas_are_calculated() {
    let report =
        p65_ratio_actors_report_file("examples/p53_strict.atlas", base_options(None, None))
            .expect("p65 report");

    assert_eq!(report.comparisons.len(), 4);
    assert!(matches!(
        report.decision,
        P65Decision::RecalibrateActorOverhead | P65Decision::NoGoLocalActors
    ));
    assert_ne!(report.decision.as_str(), "PROMOTE_P65_LOCAL_ACTORS");
    assert!(report.entries.iter().any(|entry| {
        entry.actor_strategy == "single_local_actor"
            && entry.actor_net_gain > 0.0
            && entry.actor_ratio_delta != 0.0
            && entry.actor_bytes_delta != 0
    }));
}

#[test]
fn p65_json_contains_required_schema_fields() {
    let report = p65_ratio_actors_report_file(
        "examples/p53_strict.atlas",
        base_options(
            Some(P64WorkloadKind::RealishLogEvents),
            Some(P65ActorStrategy::SingleLocalActor),
        ),
    )
    .expect("p65 report");
    let json = p65_report_json(&report);

    assert!(json.contains("\"astra_step\": \"P65\""));
    assert!(json.contains("\"actor_strategy_metrics\":"));
    assert!(json.contains("\"local_actor\":"));
    assert!(json.contains("\"total_actor_overhead_bytes\":"));
    assert!(json.contains("\"actor_overhead_ratio\":"));
    assert!(json.contains("\"actor_net_gain\":"));
    assert!(json.contains("\"conflict_count\":"));
    assert!(json.contains("\"stale_read_count\":"));
    assert!(json.contains("\"strategy_comparison\":"));
}

#[test]
fn p65_exports_are_written() {
    let export_dir = unique_export_dir();
    let report = p65_ratio_actors_report_file(
        "examples/p53_strict.atlas",
        base_options(
            Some(P64WorkloadKind::RealishLogEvents),
            Some(P65ActorStrategy::SingleLocalActor),
        ),
    )
    .expect("p65 report");

    write_p65_actor_campaign_exports(&report, &export_dir).expect("write p65 exports");

    assert!(export_dir.join("p65_actor_campaign_report.json").exists());
    assert!(export_dir.join("p65_actor_runs.jsonl").exists());
    assert!(export_dir.join("p65_actor_summary.md").exists());
    assert!(export_dir.join("p65_actor_metrics.csv").exists());
    assert_eq!(
        fs::read_to_string(export_dir.join("p65_actor_runs.jsonl"))
            .expect("runs")
            .lines()
            .count(),
        2
    );

    let _ = fs::remove_dir_all(export_dir);
}

#[test]
fn p65_ratio_actors_cli_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-actors",
            "examples/p53_strict.atlas",
            "--workload",
            "realish_log_events",
            "--actor-strategy",
            "single-local",
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
            "--format",
            "json",
        ])
        .output()
        .expect("run ratio-actors");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_step\": \"P65\""));
    assert!(stdout.contains("\"actor_strategy\": \"single-local\""));
}

#[test]
fn p65_ratio_actors_rejects_invalid_strategy() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-actors",
            "examples/p53_strict.atlas",
            "--workload",
            "realish_log_events",
            "--actor-strategy",
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
            "--format",
            "json",
        ])
        .output()
        .expect("run invalid ratio-actors");

    assert!(!output.status.success());
}

#[test]
fn p65_keeps_p64_address_local_path_working() {
    let report = p64_ratio_realish_report_file(
        "examples/p53_strict.atlas",
        P64RatioRealishOptions {
            workload: Some(P64WorkloadKind::RealishLogEvents),
            policy: Some(P64GenerationPolicy::AddressLocalGeneration),
            mode: WorkloadMode::Smoke,
            runs: 1,
            queries: 16,
            neighborhood_radius: 2,
        },
    )
    .expect("p64 report");

    assert_eq!(report.astra_step, "P64");
    assert_eq!(
        report.decision.as_str(),
        "RECALIBRATE_P64_ADDRESS_LOCAL_RATIO_MODEL"
    );
}

#[test]
fn p65_calibration_grid_builds_and_reports_best_configs() {
    let report =
        p65_actor_calibration_report_file("examples/p53_strict.atlas", calibration_options())
            .expect("p65 calibration report");

    assert_eq!(report.astra_step, "P65-2");
    assert_eq!(
        report.calibration_version,
        "p65_actor_overhead_calibration_v1"
    );
    assert_eq!(report.configurations_tested, 32);
    assert_eq!(report.configurations.len(), 32);
    assert!(report.best_by_ratio.is_some());
    assert!(report.best_by_overhead.is_some());
    assert!(report.best_balanced.is_some());
    assert!(!report.pareto_front.is_empty());
    assert_eq!(
        report.decision,
        P65CalibrationDecision::RecalibrateP65ActorOverhead
    );
}

#[test]
fn p65_calibration_config_contains_actor_overhead_and_net_gain() {
    let report =
        p65_actor_calibration_report_file("examples/p53_strict.atlas", calibration_options())
            .expect("p65 calibration report");
    let config = report.best_balanced.as_ref().expect("best balanced");

    assert!(config.actor_net_gain > 0.0);
    assert!(config.actor_overhead_ratio >= 0.0);
    assert!(config.ratio_effective_per_byte > 0.0);
    assert!(config.bytes_per_query > 0.0);
    assert!(config.cache_hit_rate >= 0.0);
    assert!(config.decision != P65CalibrationDecision::PromoteP66LocalActorArchitecture);
}

#[test]
fn p65_calibration_safety_factor_blocks_conflicts_and_stale_reads() {
    let report = p65_actor_calibration_report_file(
        "examples/p53_strict.atlas",
        P65ActorCalibrationOptions {
            workload: Some(P64WorkloadKind::RealishLogEvents),
            mode: WorkloadMode::Smoke,
            runs: 1,
            queries: 32,
            radius_grid: vec![1],
            budget_grid: vec![262_144],
            cache_grid: vec![false],
            journal_grid: vec![P65JournalPolicy::Lazy],
            query_locality_grid: vec![P65QueryLocality::Random],
        },
    )
    .expect("p65 calibration report");
    let config = report.configurations.first().expect("config");

    assert!(config.conflicts > 0 || config.stale_reads > 0);
    assert_eq!(config.balanced_score, 0.0);
    assert_eq!(
        config.decision,
        P65CalibrationDecision::NoGoP65ActorOverhead
    );
    assert!(!config.promotion_candidate);
}

#[test]
fn p65_calibration_high_overhead_cannot_promote() {
    let report =
        p65_actor_calibration_report_file("examples/p53_strict.atlas", calibration_options())
            .expect("p65 calibration report");

    for config in &report.configurations {
        if config.actor_overhead_ratio >= 0.15
            || config.conflicts > 0
            || config.stale_reads > 0
            || config.budget_refusal_rate > 0.10
        {
            assert!(!config.promotion_candidate);
        }
    }
    assert_ne!(
        report.decision,
        P65CalibrationDecision::PromoteP66LocalActorArchitecture
    );
}

#[test]
fn p65_calibration_exports_are_written() {
    let export_dir = unique_export_dir();
    let report =
        p65_actor_calibration_report_file("examples/p53_strict.atlas", calibration_options())
            .expect("p65 calibration report");

    write_p65_actor_calibration_exports(&report, &export_dir)
        .expect("write p65 calibration exports");

    assert!(export_dir
        .join("p65_actor_calibration_report.json")
        .exists());
    assert!(export_dir.join("p65_actor_calibration_runs.jsonl").exists());
    assert!(export_dir.join("p65_actor_calibration_summary.md").exists());
    assert!(export_dir.join("p65_actor_calibration_grid.csv").exists());
    assert_eq!(
        fs::read_to_string(export_dir.join("p65_actor_calibration_runs.jsonl"))
            .expect("runs")
            .lines()
            .count(),
        report.configurations_tested
    );

    let _ = fs::remove_dir_all(export_dir);
}

#[test]
fn p65_ratio_actors_calibrate_cli_succeeds() {
    let export_dir = unique_export_dir();
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-actors-calibrate",
            "examples/p53_strict.atlas",
            "--workload",
            "realish_log_events",
            "--mode",
            "smoke",
            "--runs",
            "1",
            "--queries",
            "16",
            "--radius-grid",
            "1,2",
            "--budget-grid",
            "262144,1048576",
            "--cache-grid",
            "off,on",
            "--journal-grid",
            "lazy,compact",
            "--query-locality-grid",
            "clustered,random",
            "--export-dir",
            export_dir.to_str().expect("export path"),
            "--format",
            "json",
        ])
        .output()
        .expect("run ratio-actors-calibrate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_step\": \"P65-2\""));
    assert!(stdout.contains("\"calibration_version\": \"p65_actor_overhead_calibration_v1\""));
    assert!(export_dir
        .join("p65_actor_calibration_report.json")
        .exists());

    let _ = fs::remove_dir_all(export_dir);
}

#[test]
fn p65_ratio_actors_calibrate_rejects_invalid_grid_value() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "ratio-actors-calibrate",
            "examples/p53_strict.atlas",
            "--workload",
            "realish_log_events",
            "--mode",
            "smoke",
            "--runs",
            "1",
            "--queries",
            "16",
            "--radius-grid",
            "0",
            "--budget-grid",
            "262144",
            "--cache-grid",
            "on",
            "--journal-grid",
            "lazy",
            "--query-locality-grid",
            "clustered",
            "--format",
            "json",
        ])
        .output()
        .expect("run invalid ratio-actors-calibrate");

    assert!(!output.status.success());
}

fn unique_export_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("astra-p65-test-{}-{}", std::process::id(), nanos))
}
