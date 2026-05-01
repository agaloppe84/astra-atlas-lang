use astra_atlas_lang::{
    p69_parse_contract_file, p71_all_corpora, p71_fiber_store_bench, p71_fiber_store_json,
    P71Decision, P71FiberStoreOptions, RealDataCorpusKind,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn p71_fiber_store_creates_manifest_and_reports() {
    let export_dir = temp_dir("manifest");
    let report = p71_report(&export_dir);

    assert_eq!(report.astra_step, "P71");
    assert_eq!(report.store_version, "p71_filesystem_fiber_store_v1");
    assert!(export_dir.join("store/manifest.json").exists());
    assert!(export_dir.join("store/contract.json").exists());
    assert!(export_dir.join("store/address_index.json").exists());
    assert!(export_dir.join("store/dictionaries/common.dict").exists());
    assert!(export_dir.join("store/residuals/residuals.bin").exists());
    assert!(export_dir.join("store/checksums/checksums.txt").exists());
    assert!(export_dir.join("store/audit/audit.json").exists());
    assert!(export_dir.join("p71_fiber_store_report.json").exists());
    assert!(export_dir.join("p71_fiber_store_summary.md").exists());
}

#[test]
fn p71_budget_10_mib_is_measured_against_filesystem_bytes() {
    let report = p71_report(&temp_dir("budget"));

    assert_eq!(report.budget.budget_bytes, 10_485_760);
    assert!(report.cost_breakdown.total_store_bytes > 0);
    assert!(report.budget_used_percent > 0.0);
    assert!(report.budget_pass);
    assert!(!report.refused_due_to_budget);
}

#[test]
fn p71_roundtrip_and_checksums_pass_on_exact_corpora() {
    let report = p71_report(&temp_dir("roundtrip"));

    assert!(report.roundtrip.sample_count > 0);
    assert_eq!(
        report.roundtrip.sample_count,
        report.roundtrip.exact_roundtrip_count
    );
    assert_eq!(report.roundtrip.missing_fiber_count, 0);
    assert_eq!(report.roundtrip.corrupted_fiber_count, 0);
    assert_eq!(report.roundtrip.checksum_pass_rate, 1.0);
    assert_eq!(report.roundtrip.roundtrip_success_rate, 1.0);
    assert!(report.exact_recoverable_bytes > 0);
}

#[test]
fn p71_filesystem_cost_breakdown_contains_required_categories() {
    let report = p71_report(&temp_dir("costs"));
    let cost = report.cost_breakdown;

    assert!(cost.manifest_bytes > 0);
    assert!(cost.contract_bytes > 0);
    assert!(cost.generator_bytes > 0);
    assert!(cost.dictionary_bytes > 0);
    assert!(cost.index_bytes > 0);
    assert!(cost.residual_bytes > 0);
    assert!(cost.journal_bytes > 0);
    assert!(cost.checksum_bytes > 0);
    assert!(cost.audit_metadata_bytes > 0);
    assert!(cost.actor_state_bytes > 0);
    assert!(cost.total_store_bytes > 0);
}

#[test]
fn p71_ratios_and_raw_baseline_are_calculated() {
    let report = p71_report(&temp_dir("ratios"));

    assert!(report.source_dataset_bytes > 0);
    assert_eq!(report.raw_baseline_bytes, report.source_dataset_bytes);
    assert!(report.exact_bytes_per_store_byte > 0.0);
    assert!(report.useful_retrieved_bytes_per_store_byte > 0.0);
    assert!(report.effective_units_per_store_byte > 0.0);
    assert!(report.procedural_store_gain_vs_raw > 0.0);
}

#[test]
fn p71_retrieval_query_returns_known_results() {
    let report = p71_report(&temp_dir("retrieval"));

    assert!(report.retrieval.query_count > 0);
    assert_eq!(
        report.retrieval.query_count,
        report.retrieval.successful_queries
    );
    assert_eq!(report.retrieval.precision, 1.0);
    assert_eq!(report.retrieval.recall, 1.0);
    assert_eq!(report.retrieval.exact_match_rate, 1.0);
    assert_eq!(report.retrieval.false_positive_count, 0);
    assert_eq!(report.retrieval.false_negative_count, 0);
}

#[test]
fn p71_incompressible_guard_is_refused_without_false_gain() {
    let report = p71_report(&temp_dir("guard"));

    assert!(report.guard.guard_source_bytes > 0);
    assert_eq!(report.guard.guard_store_bytes, 0);
    assert!(report.guard.guard_refused);
    assert!(report.guard.guard_no_false_gain);
    assert_eq!(
        report.guard.guard_decision,
        "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED"
    );
    assert_eq!(report.policy_counts.get("refused_fiber").copied(), Some(1));
}

#[test]
fn p71_declared_vs_measured_byte_drift_is_reported() {
    let report = p71_report(&temp_dir("drift"));

    assert_eq!(report.declared_vs_measured.declared_total_bytes, 176_128);
    assert!(report.declared_vs_measured.measured_total_store_bytes > 0);
    assert!(report.declared_vs_measured.delta_percent >= 0.0);
    assert!(matches!(
        report.declared_vs_measured.drift_status.as_str(),
        "NO_DRIFT" | "WARN_DRIFT" | "HARD_DRIFT"
    ));
    if report.declared_vs_measured.drift_status == "HARD_DRIFT" {
        assert_eq!(report.decision, P71Decision::RecalibrateContractCostModel);
    }
}

#[test]
fn p71_keeps_p69_contract_check_compatible() {
    let report = p71_report(&temp_dir("contract"));

    assert!(report.contract_check_pass);
    assert!(report.all_storage_counted);
    assert_eq!(report.hidden_storage_risk, "low");
}

#[test]
fn p71_invalid_contracts_are_refused() {
    let cases = [
        "examples/invalid/p71_hidden_raw_fallback.atlas",
        "examples/invalid/p71_missing_checksum_store.atlas",
        "examples/invalid/p71_unaccounted_dictionary.atlas",
        "examples/invalid/p71_budget_exceeded_contract.atlas",
        "examples/invalid/p71_false_guard_gain.atlas",
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
fn p71_json_contains_fiber_store_sections_without_timing_goldens() {
    let json = p71_fiber_store_json(&p71_report(&temp_dir("json")));

    assert!(json.contains("\"astra_step\": \"P71\""));
    assert!(json.contains("\"filesystem_cost_breakdown\""));
    assert!(json.contains("\"declared_vs_measured\""));
    assert!(json.contains("\"roundtrip\""));
    assert!(json.contains("\"retrieval\""));
    assert!(json.contains("\"guard\""));
    assert!(
        json.contains("RECALIBRATE_P71_ENCODING_MODEL")
            || json.contains("RECALIBRATE_P71_CONTRACT_COST_MODEL")
    );
    assert!(!json.contains("duration_ns"));
}

#[test]
fn p71_cli_fiber_store_bench_succeeds_and_writes_exports() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "fiber-store-bench",
            "--corpus",
            "all",
            "--budget-bytes",
            "10485760",
            "--runs",
            "30",
            "--queries",
            "1000",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("fiber-store-bench");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"astra_step\": \"P71\""));
    assert!(export_dir.join("p71_fiber_store_report.json").exists());
    assert!(export_dir.join("p71_decode_report.json").exists());
    assert!(export_dir.join("p71_query_report.json").exists());
}

#[test]
fn p71_corpus_selection_supports_all_and_named_corpora() {
    assert_eq!(p71_all_corpora().len(), 5);
    assert_eq!(
        RealDataCorpusKind::from_str("code"),
        Some(RealDataCorpusKind::RealCode)
    );
    assert_eq!(
        RealDataCorpusKind::from_str("guard"),
        Some(RealDataCorpusKind::IncompressibleGuardBlob)
    );
    assert_eq!(RealDataCorpusKind::from_str("unknown"), None);
}

#[test]
fn p71_test_stack_audit_report_is_versioned() {
    assert!(fs::metadata("docs/analysis/ASTRA-P71-test-stack-audit.md").is_ok());
}

fn p71_report(export_dir: &std::path::Path) -> astra_atlas_lang::FiberStoreReport {
    p71_fiber_store_bench(
        P71FiberStoreOptions {
            corpora: p71_all_corpora(),
            budget_bytes: 10_485_760,
            runs: 30,
            queries: 1000,
        },
        export_dir,
    )
    .expect("P71 fiber store report")
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "astra-p71-test-{}-{}-{}",
        std::process::id(),
        label,
        nanos
    ))
}
