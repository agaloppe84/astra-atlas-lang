use astra_atlas_lang::{
    p69_contract_report_file, p71_all_corpora, p71_fiber_store_bench,
    p72_lifecycle_file_looks_like, p72_living_store_bench, p72_living_store_json,
    p72_parse_lifecycle_file, P71FiberStoreOptions, P72CompactionPolicy, P72LivingStoreDecision,
    P72LivingStoreOptions,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID_LIFECYCLE: &str = "examples/valid/p72_living_fiber_store.atlas";

#[test]
fn p72_lifecycle_contract_parses_and_typechecks() {
    let contract = p72_parse_lifecycle_file(VALID_LIFECYCLE).expect("P72 lifecycle parses");

    assert!(p72_lifecycle_file_looks_like(VALID_LIFECYCLE));
    assert_eq!(contract.lifecycle_id, "p72_living_fiber_store");
    assert_eq!(contract.architecture_id, "address_fiber_actor_managed_v1");
    assert_eq!(contract.persistence, "cold_manifest");
    assert_eq!(contract.runtime, "materialize_on_read");
    assert_eq!(contract.reopen, "replay_journal");
    assert_eq!(contract.cache, "runtime_only");
    assert!(contract.reopen_equivalence);
    assert!(contract.all_persistent_storage_counted);
    assert!(contract.runtime_cache_not_required_for_correctness);
    assert!(contract.journal_replay_bounded);
    assert!(contract.guard_no_false_gain);
}

#[test]
fn p72_invalid_lifecycle_contracts_are_refused() {
    let cases = [
        "examples/invalid/p72_missing_reopen_gate.atlas",
        "examples/invalid/p72_cache_required_for_correctness.atlas",
        "examples/invalid/p72_unaccounted_checkpoint.atlas",
        "examples/invalid/p72_missing_journal_replay.atlas",
        "examples/invalid/p72_guard_false_gain.atlas",
        "examples/invalid/p72_unbounded_replay.atlas",
    ];

    for case in cases {
        assert!(
            p72_parse_lifecycle_file(case).is_err(),
            "{} should be refused",
            case
        );
    }
}

#[test]
fn p72_living_store_creates_cold_runtime_and_reports() {
    let export_dir = temp_dir("layout");
    let report = p72_report(&export_dir);

    assert_eq!(report.astra_step, "P72");
    assert_eq!(
        report.living_store_version,
        "p72_living_procedural_fiber_store_v1"
    );
    assert!(export_dir.join("living_store/cold/manifest.json").exists());
    assert!(export_dir.join("living_store/cold/contract.json").exists());
    assert!(export_dir
        .join("living_store/cold/journal/live.journal")
        .exists());
    assert!(export_dir
        .join("living_store/cold/checkpoints/checkpoint.json")
        .exists());
    assert!(export_dir
        .join("living_store/runtime/hot_cache/cache.tmp")
        .exists());
    assert!(export_dir.join("p72_living_report.json").exists());
    assert!(export_dir.join("p72_summary.md").exists());
    assert!(export_dir.join("p72_cost_breakdown.csv").exists());
}

#[test]
fn p72_reopen_equivalence_survives_close_reopen_replay() {
    let report = p72_report(&temp_dir("reopen"));

    assert!(report.reopen_equivalence.reopen_equivalence);
    assert!(report.reopen_equivalence.journal_replay_success);
    assert_eq!(
        report.reopen_equivalence.logical_state_hash_before_close,
        report.reopen_equivalence.logical_state_hash_after_reopen
    );
    assert_eq!(report.reopen_equivalence.reopened_read_success_rate, 1.0);
    assert_eq!(
        report.reopen_equivalence.reopened_roundtrip_success_rate,
        1.0
    );
    assert!(report.journal_replay.journal_replay_steps > report.runtime_working_set.update_count);
}

#[test]
fn p72_update_delete_audit_and_runtime_cache_are_accounted() {
    let report = p72_report(&temp_dir("actions"));

    assert_eq!(report.runtime_working_set.update_count, 5);
    assert_eq!(report.runtime_working_set.delete_count, 2);
    assert!(report.runtime_working_set.audit_count > 0);
    assert_eq!(report.runtime_working_set.close_count, 1);
    assert_eq!(report.runtime_working_set.reopen_count, 1);
    assert!(!report.runtime_state.runtime_cache_required_for_correctness);
    assert!(report.runtime_state.runtime_cache_bytes > 0);
    assert!(report.runtime_state.runtime_actor_state_bytes > 0);
    assert!(report.runtime_state.runtime_peak_bytes > 0);
}

#[test]
fn p72_compaction_preserves_logical_state_and_reports_savings() {
    let report = p72_report(&temp_dir("compaction"));

    assert_eq!(report.compaction.compact_policy, "threshold");
    assert!(report.compaction.logical_state_hash_preserved);
    assert!(report.compaction.bytes_before_compaction > 0);
    assert!(report.compaction.bytes_after_compaction > 0);
    assert!(report.compaction.compaction_savings_percent >= 0.0);
}

#[test]
fn p72_adaptive_living_fiber_preserves_exactness_and_guard() {
    let report = p72_report(&temp_dir("adaptive"));

    assert!(report.adaptive_encoding.adaptive_enabled);
    assert!(report.adaptive_encoding.adaptive_rewrite_count > 0);
    assert_eq!(
        report.adaptive_encoding.policy_after,
        "adaptive_living_fiber"
    );
    assert!(report.adaptive_encoding.exactness_preserved);
    assert!(report.adaptive_encoding.reopen_equivalence_preserved);
    assert!(report.adaptive_encoding.guard_no_false_gain);
}

#[test]
fn p72_guard_incompressible_remains_refused_or_no_go_explicit() {
    let report = p72_report(&temp_dir("guard"));

    assert_eq!(report.guard_decision, "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED");
    assert!(report.guard_no_false_gain);
}

#[test]
fn p72_cost_breakdown_ratios_and_drift_are_calculated() {
    let report = p72_report(&temp_dir("cost"));

    assert_eq!(report.budget_bytes, 10_485_760);
    assert!(report.source_dataset_bytes > 0);
    assert!(report.cold_persisted_bytes > 0);
    assert!(report.runtime_peak_bytes > 0);
    assert!(report.exact_recoverable_bytes > 0);
    assert!(report.ratio_persistent > 0.0);
    assert!(report.ratio_runtime > 0.0);
    assert!(report.ratio_living > 0.0);
    assert!(report.living_cost_breakdown.declared_persistent_bytes > 0);
    assert!(matches!(
        report.living_cost_breakdown.drift_status.as_str(),
        "NO_DRIFT" | "WARN_DRIFT" | "HARD_DRIFT"
    ));
    assert_eq!(
        report.decision,
        P72LivingStoreDecision::RecalibrateLivingCostModel
    );
}

#[test]
fn p72_json_contains_living_sections_without_timing_goldens() {
    let json = p72_living_store_json(&p72_report(&temp_dir("json")));

    assert!(json.contains("\"astra_step\": \"P72\""));
    assert!(json.contains("\"cold_state\""));
    assert!(json.contains("\"runtime_state\""));
    assert!(json.contains("\"reopen_equivalence\""));
    assert!(json.contains("\"ratio_living\""));
    assert!(json.contains("RECALIBRATE_P72_LIVING_COST_MODEL"));
    assert!(!json.contains("duration_ns"));
}

#[test]
fn p72_cli_living_store_bench_succeeds_and_writes_exports() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "living-store-bench",
            "--corpus",
            "all",
            "--budget-bytes",
            "10485760",
            "--runs",
            "30",
            "--queries",
            "1000",
            "--updates",
            "5",
            "--deletes",
            "2",
            "--compact",
            "threshold",
            "--adaptive",
            "on",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("living-store-bench");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"astra_step\": \"P72\""));
    assert!(export_dir.join("p72_living_report.json").exists());
    assert!(export_dir
        .join("living_store/reports/p72_living_report.json")
        .exists());
}

#[test]
fn p72_keeps_p71_and_p69_compatibility() {
    let p71 = p71_fiber_store_bench(
        P71FiberStoreOptions {
            corpora: p71_all_corpora(),
            budget_bytes: 10_485_760,
            runs: 30,
            queries: 1000,
        },
        temp_dir("p71-compat"),
    )
    .expect("P71 still works");
    let p69 = p69_contract_report_file("examples/valid/p69_address_fiber_contract.atlas")
        .expect("P69 still works");

    assert_eq!(p71.astra_step, "P71");
    assert_eq!(p69.astra_step, "P69");
    assert!(p69.all_storage_counted);
}

#[test]
fn p72_test_stack_audit_report_is_versioned() {
    assert!(fs::metadata("docs/analysis/ASTRA-P72-test-stack-audit.md").is_ok());
}

fn p72_report(export_dir: &std::path::Path) -> astra_atlas_lang::P72LivingStoreReport {
    p72_living_store_bench(
        P72LivingStoreOptions {
            corpora: p71_all_corpora(),
            budget_bytes: 10_485_760,
            runs: 30,
            queries: 1000,
            updates: 5,
            deletes: 2,
            compact: P72CompactionPolicy::Threshold,
            adaptive: true,
            reopen_check: true,
        },
        export_dir,
    )
    .expect("P72 living store report")
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "astra-p72-test-{}-{}-{}",
        std::process::id(),
        label,
        nanos
    ))
}
