use astra_atlas_lang::{
    p71_all_corpora, p78_all_level1_topologies, p78_level1_space_bench, p78_level1_space_estimate,
    p78_parse_level1_contract_file, FileTypeClassifier, Level1TopologyKind,
    Level1VirtualSpaceEstimateOptions, P74CompactionPolicy, P78Decision, P78Level1SpaceOptions,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID: &str = "examples/valid/p78_level1_virtual_space.atlas";

#[test]
fn p78_level1_address_space_exists_and_topologies_are_complete() {
    let topologies = p78_all_level1_topologies();
    assert_eq!(topologies.len(), 8);
    assert!(topologies.contains(&Level1TopologyKind::Grid2D));
    assert!(topologies.contains(&Level1TopologyKind::Grid3D));
    assert!(topologies.contains(&Level1TopologyKind::HierarchicalTree));
    assert!(topologies.contains(&Level1TopologyKind::PathTrie));
    assert!(topologies.contains(&Level1TopologyKind::ContentAddressedDag));
    assert!(topologies.contains(&Level1TopologyKind::GraphAddressSpace));
    assert!(topologies.contains(&Level1TopologyKind::ProductTypedSpace));
    assert!(topologies.contains(&Level1TopologyKind::HybridMultiIndexSpace));
}

#[test]
fn p78_virtual_space_estimator_marks_virtual_bytes_as_equivalent() {
    let metrics = p78_level1_space_estimate(Level1VirtualSpaceEstimateOptions {
        topology_kind: Level1TopologyKind::HybridMultiIndexSpace,
        target_source_bytes: 10_485_760,
        address_bits: 64,
        file_type_count: 16,
        object_count: 10_000,
        chunk_count: 40_000,
        version_count: 4,
        fibers_per_object: 4,
    });

    assert_eq!(metrics.virtual_cell_count, 10_000);
    assert_eq!(metrics.virtual_fiber_count, 40_000);
    assert_eq!(metrics.virtual_chunk_count, 40_000);
    assert!(metrics.virtual_effective_bytes_equivalent > 10_485_760);
    assert!(metrics.bytes_are_equivalent_not_stored);
    assert_eq!(metrics.limiting_factor, "index_size");
}

#[test]
fn p78_universal_file_classifier_uses_raw_fallback_for_unknown_extension() {
    let classifier = FileTypeClassifier {
        classifier_version: "test".to_string(),
    };
    let unknown = classifier.classify("opaque.unknown_ext", "unknown");
    let guard = classifier.classify("guard.bin", "guard");

    assert_eq!(unknown.recommended_codec, "raw_fallback");
    assert!(unknown.raw_fallback);
    assert_eq!(guard.recommended_codec, "refused_guard");
    assert!(guard.guard);
}

#[test]
fn p78_valid_contract_parses_and_invalid_contracts_are_refused() {
    let contract = p78_parse_level1_contract_file(VALID).expect("valid P78 parses");
    assert_eq!(
        contract.space.topology_kind,
        Level1TopologyKind::HybridMultiIndexSpace
    );
    assert!(contract.space.local_on_address);
    assert_eq!(contract.virtual_bytes_claim, "equivalent");

    let invalids = [
        "examples/invalid/p78_global_materialization_allowed.atlas",
        "examples/invalid/p78_virtual_bytes_claim_stored.atlas",
        "examples/invalid/p78_missing_local_on_address.atlas",
        "examples/invalid/p78_missing_virtual_space_metrics.atlas",
        "examples/invalid/p78_raw_fallback_false_gain.atlas",
        "examples/invalid/p78_unknown_level1_topology.atlas",
        "examples/invalid/p78_unbounded_address_lookup.atlas",
        "examples/invalid/p78_missing_guard_gate.atlas",
    ];
    for invalid in invalids {
        assert!(
            p78_parse_level1_contract_file(invalid).is_err(),
            "{} should be refused",
            invalid
        );
    }
}

#[test]
fn p78_level1_space_bench_produces_living_ratio_and_metrics() {
    let export_dir = temp_dir("bench");
    let report = p78_report(&export_dir);

    assert_eq!(report.astra_step, "P78");
    assert_eq!(report.actual_source_bytes, 1_048_576);
    assert_eq!(
        report.best_result.topology_kind,
        Level1TopologyKind::HybridMultiIndexSpace
    );
    assert!(report.best_result.ratio_living > 5.0);
    assert!(report.best_result.reopen_equivalence);
    assert_eq!(report.best_result.drift_status, "NO_DRIFT");
    assert_eq!(
        report.best_result.guard_decision,
        "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED"
    );
    assert_eq!(report.best_result.crud_metrics.crud_success_rate, 1.0);
    assert_eq!(
        report
            .best_result
            .addressing_metrics
            .address_lookup_success_rate,
        1.0
    );
    assert!(report.universal_codec_report.raw_fallback_bytes > 0);
    assert!(report.universal_codec_report.guard_no_false_gain);
    assert_eq!(report.decision, P78Decision::RecalibrateLevel1Topology);
}

#[test]
fn p78_exports_expected_artifacts() {
    let export_dir = temp_dir("exports");
    let report = p78_report(&export_dir);

    assert!(export_dir.join("p78_level1_space_report.json").exists());
    assert!(export_dir
        .join("p78_level1_topology_results.jsonl")
        .exists());
    assert!(export_dir.join("p78_virtual_space_metrics.json").exists());
    assert!(export_dir.join("p78_universal_codec_report.json").exists());
    assert!(export_dir.join("p78_addressing_metrics.csv").exists());
    assert!(export_dir.join("p78_crud_metrics.csv").exists());
    assert!(export_dir.join("p78_summary.md").exists());
    assert!(export_dir
        .join("stores")
        .join("hybrid_multi_index")
        .join("cold")
        .exists());
    assert!(report.comparison.green_count > 0);
    assert!(report.comparison.red_count > 0);
}

#[test]
fn p78_level1_space_estimate_cli_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "level1-space-estimate",
            "--level1-topology",
            "hybrid-multi-index",
            "--target-source-bytes",
            "10485760",
            "--address-bits",
            "64",
            "--file-type-count",
            "16",
            "--object-count",
            "10000",
            "--chunk-count",
            "40000",
            "--version-count",
            "4",
            "--fibers-per-object",
            "4",
            "--format",
            "json",
        ])
        .output()
        .expect("level1 estimate cli");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"virtual_effective_bytes_equivalent\""));
    assert!(stdout.contains("\"bytes_are_equivalent_not_stored\": true"));
}

#[test]
fn p78_level1_space_bench_cli_succeeds() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "level1-space-bench",
            "--corpus",
            "all",
            "--level1-topology",
            "all",
            "--fiber-router",
            "p77-calibrated",
            "--target-source-bytes",
            "1048576",
            "--cycles",
            "2",
            "--queries",
            "500",
            "--updates",
            "50",
            "--deletes",
            "5",
            "--compact",
            "adaptive",
            "--adaptive",
            "on",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("level1 bench cli");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_step\": \"P78\""));
    assert!(stdout.contains("\"ratio_living\""));
    assert!(export_dir.join("p78_level1_space_report.json").exists());
}

#[test]
fn p78_check_command_and_p77_non_regression_remain_available() {
    let p78 = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["check", VALID])
        .output()
        .expect("p78 check");
    assert!(
        p78.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&p78.stderr)
    );
    assert!(String::from_utf8_lossy(&p78.stdout).contains("p78_level1_space"));

    let p77 = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["check", "examples/valid/p77_calibrated_router_policy.atlas"])
        .output()
        .expect("p77 check");
    assert!(
        p77.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&p77.stderr)
    );
    assert!(String::from_utf8_lossy(&p77.stdout).contains("p77_policy"));
}

#[test]
fn p78_test_stack_audit_is_versioned() {
    assert!(fs::metadata("docs/analysis/ASTRA-P78-test-stack-audit.md").is_ok());
}

#[test]
fn p78_has_no_timing_golden() {
    let report = p78_report(&temp_dir("no-timing"));
    let markdown = astra_atlas_lang::p78_level1_space_markdown(&report);
    assert!(!markdown.contains("p99"));
    assert!(!markdown.contains("duration_ns"));
}

fn p78_report(export_dir: &std::path::Path) -> astra_atlas_lang::P78Level1SpaceReport {
    p78_level1_space_bench(
        P78Level1SpaceOptions {
            corpora: p71_all_corpora(),
            level1_topologies: p78_all_level1_topologies(),
            fiber_router: "p77-calibrated".to_string(),
            target_source_bytes: 1_048_576,
            cycles: 2,
            queries: 500,
            updates: 50,
            deletes: 5,
            compact: P74CompactionPolicy::Adaptive,
            adaptive: true,
        },
        export_dir,
    )
    .expect("P78 report")
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    path.push(format!("astra_p78_{}_{}", label, nanos));
    fs::create_dir_all(&path).expect("temp dir");
    path
}
