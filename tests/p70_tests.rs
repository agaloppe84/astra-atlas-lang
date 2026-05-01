use astra_atlas_lang::{
    p69_parse_contract_file, p70_all_fixture_kinds, p70_contract_replay_json,
    p70_contract_replay_report_file, p70_detect_fixture_drift, write_p70_contract_replay_exports,
    P70ContractReplayOptions, P70Decision, P70DriftStatus, P70ReplayFixtureKind, WorkloadMode,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID: &str = "examples/valid/p69_address_fiber_contract.atlas";

#[test]
fn p70_contract_replay_runs_on_valid_p69_contract() {
    let report = replay_report();

    assert_eq!(report.astra_step, "P70");
    assert_eq!(report.replay_version, "p70_contract_replay_v1");
    assert_eq!(report.fixtures.len(), 4);
    assert_eq!(report.drift_summary.status, P70DriftStatus::NoDrift);
    assert_eq!(report.decision, P70Decision::RecalibrateContractDrift);
}

#[test]
fn p70_declared_vs_measured_bytes_are_calculated() {
    let report = replay_report();
    let summary = report.declared_vs_measured_summary;

    assert_eq!(summary.declared_contract_bytes, 176128);
    assert!(summary.measured_runtime_bytes_min >= summary.declared_contract_bytes);
    assert!(summary.measured_runtime_bytes_median >= summary.measured_runtime_bytes_min);
    assert!(summary.measured_runtime_bytes_max >= summary.measured_runtime_bytes_median);
    assert!(summary.max_byte_delta_percent > 0.0);
    assert!(summary.max_byte_delta_percent < 5.0);
}

#[test]
fn p70_replay_exposes_fixture_cost_fields() {
    let report = replay_report();

    for fixture in &report.fixtures {
        assert!(fixture.declared_contract_bytes > 0);
        assert!(fixture.measured_runtime_bytes > 0);
        assert_eq!(fixture.accounted_storage_ratio, 1.0);
        assert_eq!(fixture.hidden_storage_risk, "low");
        assert!(fixture.contract_ratio_effective_per_byte > 0.0);
        assert_eq!(fixture.fiber_effective_units, 8_448_000);
        assert!(fixture.cache_bytes > 0);
        assert!(fixture.journal_bytes > 0);
        assert!(fixture.actor_state_bytes > 0);
        assert!(fixture.audit_metadata_bytes > 0);
        assert!(fixture.index_bytes > 0);
        assert!(fixture.residual_bytes > 0);
    }
}

#[test]
fn p70_valid_contract_has_no_drift_but_decision_remains_conservative() {
    let report = replay_report();

    assert_eq!(report.drift_summary.no_drift_count, 4);
    assert_eq!(report.drift_summary.warn_drift_count, 0);
    assert_eq!(report.drift_summary.hard_drift_count, 0);
    assert_eq!(report.drift_summary.invalid_contract_count, 0);
    assert_eq!(report.decision, P70Decision::RecalibrateContractDrift);
}

#[test]
fn p70_drift_detector_warns_or_blocks_large_byte_delta() {
    let report = replay_report();
    let mut fixture = report.fixtures[0].clone();

    fixture.measured_runtime_bytes = fixture.declared_contract_bytes + 30_000;
    fixture.byte_delta = 30_000;
    fixture.byte_delta_percent = 30_000.0 * 100.0 / fixture.declared_contract_bytes as f64;
    assert_eq!(
        p70_detect_fixture_drift(&fixture, 5.0),
        P70DriftStatus::HardDrift
    );

    fixture.measured_runtime_bytes = fixture.declared_contract_bytes + 12_000;
    fixture.byte_delta = 12_000;
    fixture.byte_delta_percent = 12_000.0 * 100.0 / fixture.declared_contract_bytes as f64;
    assert_eq!(
        p70_detect_fixture_drift(&fixture, 5.0),
        P70DriftStatus::WarnDrift
    );
}

#[test]
fn p70_unaccounted_storage_is_invalid_contract_drift() {
    let report = replay_report();
    let mut fixture = report.fixtures[0].clone();

    fixture.accounted_storage_ratio = 0.92;
    assert_eq!(
        p70_detect_fixture_drift(&fixture, 5.0),
        P70DriftStatus::InvalidContract
    );

    fixture.accounted_storage_ratio = 1.0;
    fixture.hidden_storage_risk = "high".to_string();
    assert_eq!(
        p70_detect_fixture_drift(&fixture, 5.0),
        P70DriftStatus::InvalidContract
    );
}

#[test]
fn p70_invalid_contracts_are_refused() {
    let cases = [
        "examples/invalid/p70_cache_unaccounted.atlas",
        "examples/invalid/p70_journal_unaccounted.atlas",
        "examples/invalid/p70_actor_state_unaccounted.atlas",
        "examples/invalid/p70_missing_audit_metadata.atlas",
        "examples/invalid/p70_hidden_storage_risk_high.atlas",
    ];

    for case in cases {
        assert!(
            p69_parse_contract_file(case).is_err(),
            "{} should be refused",
            case
        );
    }
}

#[test]
fn p70_test_stack_audit_report_is_versioned() {
    assert!(fs::metadata("docs/analysis/ASTRA-P70-test-stack-audit.md").is_ok());
}

#[test]
fn p70_exports_are_written() {
    let report = replay_report();
    let export_dir = temp_dir("exports");
    write_p70_contract_replay_exports(&report, &export_dir).expect("write P70 exports");

    assert!(export_dir.join("p70_contract_replay_report.json").exists());
    assert!(export_dir
        .join("p70_contract_replay_fixtures.jsonl")
        .exists());
    assert!(export_dir.join("p70_contract_replay_drift.csv").exists());
    assert!(export_dir.join("p70_contract_replay_summary.md").exists());
}

#[test]
fn p70_contract_replay_json_contains_required_sections() {
    let json = p70_contract_replay_json(&replay_report());

    assert!(json.contains("\"astra_step\": \"P70\""));
    assert!(json.contains("\"declared_vs_measured_summary\""));
    assert!(json.contains("\"drift_summary\""));
    assert!(json.contains("\"test_stack_audit_summary\""));
    assert!(json.contains("\"fixtures\""));
    assert!(json.contains("RECALIBRATE_P70_CONTRACT_DRIFT"));
}

#[test]
fn p70_cli_contract_replay_succeeds_and_writes_exports() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "contract-replay",
            VALID,
            "--fixtures",
            "all",
            "--mode",
            "standard",
            "--runs",
            "30",
            "--queries",
            "1000",
            "--tolerance-percent",
            "5.0",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("contract-replay");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_step\": \"P70\""));
    assert!(export_dir.join("p70_contract_replay_report.json").exists());
}

#[test]
fn p70_keeps_p69_contract_check_working() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["contract-check", VALID, "--format", "json"])
        .output()
        .expect("contract-check");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"astra_step\": \"P69\""));
}

#[test]
fn p70_fixture_selection_supports_all_and_named_fixtures() {
    assert_eq!(p70_all_fixture_kinds().len(), 4);
    assert_eq!(
        P70ReplayFixtureKind::from_str("log"),
        Some(P70ReplayFixtureKind::LogEventFiberReplay)
    );
    assert_eq!(
        P70ReplayFixtureKind::from_str("hybrid"),
        Some(P70ReplayFixtureKind::HybridFieldTileFiberReplay)
    );
    assert_eq!(P70ReplayFixtureKind::from_str("unknown"), None);
}

fn replay_report() -> astra_atlas_lang::ContractReplayReport {
    p70_contract_replay_report_file(
        VALID,
        P70ContractReplayOptions {
            fixtures: p70_all_fixture_kinds(),
            mode: WorkloadMode::Standard,
            runs: 30,
            queries: 1000,
            tolerance_percent: 5.0,
        },
    )
    .expect("P70 replay report")
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "astra-p70-test-{}-{}-{}",
        std::process::id(),
        label,
        nanos
    ))
}
