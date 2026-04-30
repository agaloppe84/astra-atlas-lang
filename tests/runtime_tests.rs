use astra_atlas_lang::{
    metrics_json, metrics_json_file, run_smoke, validate, DiagnosticCode, MemoryRuntime,
    RuntimeConfig, SmokeWorkload,
};

const VALID: &str = include_str!("../examples/p53_strict.atlas");
const SNAPSHOT_FULL: &str = include_str!("../examples/invalid/snapshot_full.atlas");
const BAD_VERSION: &str = include_str!("../examples/invalid/bad_version.atlas");

#[test]
fn runtime_instantiates_from_p53_strict() {
    let program = validate(VALID).expect("p53 strict should validate");
    let config = RuntimeConfig::from_checked_program(&program);

    assert_eq!(config.version, "0.1");
    assert!(config.strict_p53());
    assert_eq!(config.families.len(), 12);
    assert!(config.family("stream_processing").is_some());
    assert!(config.family("sparse_index").is_some());
    assert!(config.family("columnar_table").is_some());
}

#[test]
fn smoke_run_succeeds() {
    let metrics = run_smoke(VALID).expect("runtime smoke should succeed");

    assert_eq!(metrics.mode, "smoke");
    assert_eq!(
        metrics.workload_families,
        vec![
            "stream_processing".to_string(),
            "sparse_index".to_string(),
            "columnar_table".to_string(),
        ]
    );
    assert_eq!(metrics.p56_status, "SMOKE_OK_CI_REQUIRED");
    assert_eq!(metrics.atlas_file, "<memory>");
    assert_eq!(metrics.families_total, 12);
    assert!(metrics.runtime_instantiated);
    assert_eq!(metrics.encoded_segments_total, 12);
    assert_eq!(metrics.read_count, 12);
    assert_eq!(metrics.update_count, 6);
    assert_eq!(metrics.snapshot_count, 1);
    assert_eq!(metrics.rebuild_count, 1);
    assert_eq!(metrics.query_success_rate, 1.0);
    assert_eq!(metrics.dangerous_encoded_count, 0);
    assert_eq!(metrics.guard_encoded_count, 0);
    assert!(metrics.strict_p53_preserved);
    assert!(metrics.invalid_regression_checked);
    assert!(metrics.rebuild_matches);
    assert!(metrics.gates.p56_g1_runtime_instantiates);
    assert!(metrics.gates.p56_g2_encode_read_update);
    assert!(metrics.gates.p56_g3_snapshot_incremental);
    assert!(metrics.gates.p56_g4_rebuild);
    assert!(metrics.gates.p56_g5_metrics_export);
    assert!(metrics.gates.p56_g6_p99_under_budget_smoke);
    assert!(metrics.gates.p56_g7_invalid_still_refused);
}

#[test]
fn metrics_json_is_deterministic() {
    let first = metrics_json(VALID).expect("first metrics export");
    let second = metrics_json(VALID).expect("second metrics export");

    assert_eq!(first, second);
    assert!(first.contains("\"p56_status\": \"SMOKE_OK_CI_REQUIRED\""));
    assert!(first.contains("\"atlas_file\": \"<memory>\""));
    assert!(first.contains("\"mode\": \"smoke\""));
    assert!(first.contains("\"families_total\": 12"));
    assert!(first.contains("\"runtime_instantiated\": true"));
    assert!(first.contains("\"encoded_segments_total\": 12"));
    assert!(first.contains("\"read_count\": 12"));
    assert!(first.contains("\"update_count\": 6"));
    assert!(first.contains("\"snapshot_count\": 1"));
    assert!(first.contains("\"rebuild_count\": 1"));
    assert!(first.contains("\"query_success_rate\": 1.000"));
    assert!(first.contains("\"memory_amplification_proxy\": 1.250"));
    assert!(first.contains("\"guard_encoded_count\": 0"));
    assert!(first.contains("\"dangerous_encoded_count\": 0"));
    assert!(first.contains("\"strict_p53_preserved\": true"));
    assert!(first.contains("\"invalid_regression_checked\": true"));
    assert!(first.contains("\"rebuild_matches\": true"));
    assert!(first.contains("\"latency_metric_kind\": \"smoke_proxy\""));
    assert!(first.contains("\"P56_G0_build_test_ci\": \"external_required\""));
    assert!(first.contains("\"P56_G1_runtime_instantiates\": true"));
    assert!(first.contains("\"P56_G2_encode_read_update\": true"));
    assert!(first.contains("\"P56_G3_snapshot_incremental\": true"));
    assert!(first.contains("\"P56_G4_rebuild\": true"));
    assert!(first.contains("\"P56_G5_metrics_export\": true"));
    assert!(first.contains("\"P56_G6_p99_under_budget_smoke\": true"));
    assert!(first.contains("\"P56_G7_invalid_still_refused\": true"));
    assert!(first.contains("\"P56_G8_ci_source_of_truth\": \"external_required\""));
}

#[test]
fn metrics_json_file_reports_atlas_path() {
    let json = metrics_json_file("examples/p53_strict.atlas").expect("file metrics export");

    assert!(json.contains("\"atlas_file\": \"examples/p53_strict.atlas\""));
    assert!(json.contains("\"encoded_segments_total\": 12"));
}

#[test]
fn snapshot_rebuild_roundtrip_preserves_state_summary() {
    let program = validate(VALID).expect("p53 strict should validate");
    let config = RuntimeConfig::from_checked_program(&program);
    let workload = SmokeWorkload::deterministic(&config);
    let mut runtime = MemoryRuntime::new(config.clone());

    runtime.encode_all(&workload.records);
    for read in &workload.reads {
        assert!(runtime.read(&read.family, &read.key).is_some());
    }
    for update in &workload.updates {
        assert!(runtime.update(&update.family, &update.key, &update.value));
    }

    let snapshot = runtime.snapshot();
    let before = runtime.state_summary();
    let rebuilt = MemoryRuntime::rebuild(config, &snapshot);

    assert_eq!(rebuilt.state_summary(), before);
}

#[test]
fn invalid_atlas_files_still_fail_runtime() {
    let err = run_smoke(BAD_VERSION).expect_err("bad version must fail runtime");
    assert_eq!(err.code, DiagnosticCode::VersionUnknown);
}

#[test]
fn snapshot_full_remains_refused_under_strict_p53_runtime() {
    let err = run_smoke(SNAPSHOT_FULL).expect_err("snapshot_full must fail runtime");
    assert_eq!(err.code, DiagnosticCode::SnapshotFullStrict);
    assert_eq!(err.field.as_deref(), Some("snapshot"));
}
