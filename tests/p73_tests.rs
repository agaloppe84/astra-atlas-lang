use astra_atlas_lang::{
    p71_all_corpora, p73_cubical_store_bench, p73_cubical_store_json, p73_parse_cubical_file,
    CubicalDecision, CubicalFaceDirection, P72CompactionPolicy, P73CompareP72,
    P73CubicalStoreOptions,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID_CUBICAL: &str = "examples/valid/p73_cubical_living_store.atlas";

#[test]
fn p73_cubical_contract_parses_and_typechecks() {
    let contract = p73_parse_cubical_file(VALID_CUBICAL).expect("P73 cubical contract parses");

    assert_eq!(contract.topology_id, "cubical_3d");
    assert_eq!(contract.cell, "cube");
    assert_eq!(contract.faces, 6);
    assert_eq!(contract.adjacency, "von_neumann_6");
    assert_eq!(contract.boundary_policy, "shared_faces");
    assert_eq!(contract.gluing_rule, "checksum_and_delta");
    assert!(contract.face_gluing_consistency);
    assert!(!contract.hidden_face_storage);
    assert!(contract.cubical_reopen_equivalence);
    assert!(contract.guard_no_false_gain);
}

#[test]
fn p73_face_directions_are_exactly_six() {
    let directions = CubicalFaceDirection::all();

    assert_eq!(directions.len(), 6);
    assert_eq!(
        CubicalFaceDirection::from_str("plus_x"),
        Some(CubicalFaceDirection::PlusX)
    );
    assert_eq!(
        CubicalFaceDirection::from_str("minus_z"),
        Some(CubicalFaceDirection::MinusZ)
    );
    assert_eq!(CubicalFaceDirection::from_str("diagonal_w"), None);
}

#[test]
fn p73_invalid_cubical_contracts_are_refused() {
    let cases = [
        "examples/invalid/p73_unknown_face_direction.atlas",
        "examples/invalid/p73_missing_face_checksum.atlas",
        "examples/invalid/p73_bad_gluing_rule.atlas",
        "examples/invalid/p73_hidden_face_storage.atlas",
        "examples/invalid/p73_unbounded_face_update.atlas",
        "examples/invalid/p73_invalid_cube_adjacency.atlas",
        "examples/invalid/p73_cache_required_for_correctness.atlas",
        "examples/invalid/p73_missing_cubical_reopen_gate.atlas",
    ];

    for case in cases {
        assert!(
            p73_parse_cubical_file(case).is_err(),
            "{} should be refused",
            case
        );
    }
}

#[test]
fn p73_cubical_cells_expose_six_faces_and_gluing() {
    let report = p73_report(&temp_dir("cells"));

    assert!(report.cell_count > 0);
    assert_eq!(report.face_count, report.cell_count * 6);
    let cell = &report.cells[0];
    assert_eq!(cell.faces.len(), 6);
    assert!(cell
        .gluing_constraints
        .contains(&"checksum_match".to_string()));
    assert!(cell
        .faces
        .iter()
        .all(|face| face.gluing_status == "CONSISTENT"));
}

#[test]
fn p73_crud_face_update_and_read_metrics_are_bounded() {
    let report = p73_report(&temp_dir("crud"));

    assert_eq!(report.crud.cell_read_success_rate, 1.0);
    assert_eq!(report.crud.face_read_success_rate, 1.0);
    assert_eq!(report.crud.update_interior_count, 20);
    assert_eq!(report.crud.update_face_count, 40);
    assert_eq!(report.crud.delete_cell_count, 3);
    assert_eq!(report.crud.tombstone_face_count, 18);
    assert_eq!(report.crud.gluing_failure_count, 0);
    assert_eq!(report.crud.stale_face_read_count, 0);
    assert_eq!(report.crud.hidden_face_storage_risk, "low");
}

#[test]
fn p73_compaction_reopen_and_corruption_recovery_pass() {
    let report = p73_report(&temp_dir("lifecycle"));

    assert!(report.reopen.cubical_reopen_equivalence);
    assert!(report.reopen.face_gluing_consistency);
    assert_eq!(
        report.reopen.logical_state_hash_before_close,
        report.reopen.logical_state_hash_after_reopen
    );
    assert!(report.reopen.journal_replay_steps > 0);
    assert!(report.reopen.face_journal_replay_steps > 0);
    assert!(report.compaction.logical_state_preserved);
    assert!(report.compaction.compaction_savings_bytes > 0);
    assert_eq!(report.corruption_recovery.corruptions_injected, 2);
    assert_eq!(report.corruption_recovery.corruption_detected_count, 2);
    assert_eq!(report.corruption_recovery.recovery_success_count, 2);
    assert_eq!(report.corruption_recovery.unrecovered_corruption_count, 0);
}

#[test]
fn p73_guard_metrics_ratios_and_p72_comparison_are_present() {
    let report = p73_report(&temp_dir("ratios"));

    assert_eq!(report.guard_decision, "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED");
    assert!(report.guard_no_false_gain);
    assert!(report.ratio_persistent > 0.0);
    assert!(report.ratio_runtime > 0.0);
    assert!(report.ratio_living > 0.0);
    assert_eq!(report.p72_baseline_ratio_living, 2.366879);
    assert!(report.cubical_gain_vs_p72 > 1.0);
    assert!(report.face_factorization_gain >= 0.0);
    assert!(report.gluing_overhead_ratio >= 0.0);
    assert!(report.topology_overhead_ratio >= 0.0);
    assert_eq!(
        report.decision,
        CubicalDecision::RecalibrateCubicalFiberTopology
    );
}

#[test]
fn p73_exports_are_written() {
    let export_dir = temp_dir("exports");
    let report = p73_report(&export_dir);

    assert_eq!(report.astra_step, "P73");
    assert!(export_dir.join("p73_cubical_store_report.json").exists());
    assert!(export_dir.join("p73_cubical_cells.jsonl").exists());
    assert!(export_dir.join("p73_cubical_faces.jsonl").exists());
    assert!(export_dir.join("p73_gluing_audit.csv").exists());
    assert!(export_dir.join("p73_cost_breakdown.csv").exists());
    assert!(export_dir.join("p73_corruption_recovery.json").exists());
    assert!(export_dir
        .join("cubical_store/cold/topology/cubical.topo")
        .exists());
    assert!(export_dir
        .join("cubical_store/runtime/materialized_faces/faces.tmp")
        .exists());
}

#[test]
fn p73_json_contains_required_sections_without_timing_goldens() {
    let json = p73_cubical_store_json(&p73_report(&temp_dir("json")));

    assert!(json.contains("\"astra_step\": \"P73\""));
    assert!(json.contains("\"cubical_gain_vs_p72\""));
    assert!(json.contains("\"face_gluing_consistency\""));
    assert!(json.contains("\"cost_breakdown\""));
    assert!(json.contains("RECALIBRATE_P73_CUBICAL_FIBER_TOPOLOGY"));
    assert!(!json.contains("duration_ns"));
}

#[test]
fn p73_cli_cubical_store_bench_succeeds() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "cubical-store-bench",
            "--corpus",
            "all",
            "--budget-bytes",
            "10485760",
            "--cycles",
            "2",
            "--queries",
            "200",
            "--updates",
            "20",
            "--deletes",
            "3",
            "--corruptions",
            "2",
            "--compact",
            "threshold",
            "--adaptive",
            "on",
            "--compare-p72",
            "baseline",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("cubical-store-bench");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"astra_step\": \"P73\""));
    assert!(export_dir.join("p73_cubical_store_report.json").exists());
}

#[test]
fn p73_keeps_p72_living_store_path_available() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["check", "examples/valid/p72_living_fiber_store.atlas"])
        .output()
        .expect("p72 check");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("p72_lifecycle"));
}

#[test]
fn p73_test_stack_audit_report_is_versioned() {
    assert!(fs::metadata("docs/analysis/ASTRA-P73-test-stack-audit.md").is_ok());
}

fn p73_report(export_dir: &std::path::Path) -> astra_atlas_lang::P73CubicalStoreReport {
    p73_cubical_store_bench(
        P73CubicalStoreOptions {
            corpora: p71_all_corpora(),
            budget_bytes: 10_485_760,
            cycles: 2,
            queries: 200,
            updates: 20,
            deletes: 3,
            corruptions: 2,
            compact: P72CompactionPolicy::Threshold,
            adaptive: true,
            compare_p72: P73CompareP72::Baseline,
        },
        export_dir,
    )
    .expect("P73 cubical store report")
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "astra-p73-test-{}-{}-{}",
        std::process::id(),
        label,
        nanos
    ))
}
