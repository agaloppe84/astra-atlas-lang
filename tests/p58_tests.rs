use astra_atlas_lang::{
    p57_report_json, p58_metrics_json, p58_report, p58_report_json, p58_report_json_file,
    p58_report_markdown, run_workload, runtime_config, RuntimeWorkload, SmokeWorkload,
    WorkloadExpectation, WorkloadMode,
};
use std::process::Command;

const VALID: &str = include_str!("../examples/p53_strict.atlas");

#[test]
fn workload_mode_parsing_accepts_expected_modes() {
    assert_eq!(WorkloadMode::from_str("smoke"), Some(WorkloadMode::Smoke));
    assert_eq!(
        WorkloadMode::from_str("standard"),
        Some(WorkloadMode::Standard)
    );
    assert_eq!(
        WorkloadMode::from_str("ambitious"),
        Some(WorkloadMode::Ambitious)
    );
}

#[test]
fn workload_mode_parsing_rejects_unknown_modes() {
    assert_eq!(WorkloadMode::from_str("full"), None);
    assert_eq!(WorkloadMode::from_str(""), None);
}

#[test]
fn smoke_workload_still_exists_and_is_deterministic() {
    let config = runtime_config(VALID).expect("valid runtime config");
    let first = SmokeWorkload::deterministic(&config);
    let second = SmokeWorkload::deterministic(&config);

    assert_eq!(first, second);
    assert_eq!(first.mode, WorkloadMode::Smoke);
    assert_eq!(first.families.len(), 3);
    assert_eq!(first.records.len(), 12);
    assert_eq!(first.reads.len(), 12);
    assert_eq!(first.updates.len(), 6);
    assert_eq!(first.snapshot_count, 1);
    assert_eq!(first.rebuild_count, 1);
}

#[test]
fn standard_workload_covers_active_non_guard_families() {
    let config = runtime_config(VALID).expect("valid runtime config");
    let workload = RuntimeWorkload::for_mode(&config, WorkloadMode::Standard);
    let active_families: Vec<String> = config
        .families
        .iter()
        .filter(|family| family.name != "guard" && family.action != "refuse")
        .map(|family| family.name.clone())
        .collect();

    assert_eq!(workload.mode, WorkloadMode::Standard);
    assert_eq!(workload.families, active_families);
    assert_eq!(workload.families.len(), 11);
    assert!(!workload.families.iter().any(|family| family == "guard"));
    assert_eq!(workload.specs.len(), workload.families.len());
    assert!(workload
        .specs
        .iter()
        .all(|spec| spec.expected_category != WorkloadExpectation::Refuse));
}

#[test]
fn ambitious_workload_is_deterministic_and_larger_than_standard() {
    let config = runtime_config(VALID).expect("valid runtime config");
    let standard = RuntimeWorkload::for_mode(&config, WorkloadMode::Standard);
    let first = RuntimeWorkload::for_mode(&config, WorkloadMode::Ambitious);
    let second = RuntimeWorkload::for_mode(&config, WorkloadMode::Ambitious);

    assert_eq!(first, second);
    assert_eq!(first.mode, WorkloadMode::Ambitious);
    assert!(first.records.len() > standard.records.len());
    assert!(first.reads.len() > standard.reads.len());
    assert!(first.updates.len() > standard.updates.len());
}

#[test]
fn guard_family_is_not_encoded_as_normal_workload() {
    let config = runtime_config(VALID).expect("valid runtime config");
    let workload = RuntimeWorkload::for_mode(&config, WorkloadMode::Standard);

    assert!(!workload
        .records
        .iter()
        .any(|record| record.family == "guard"));
    assert!(!workload.reads.iter().any(|read| read.family == "guard"));
    assert!(!workload
        .updates
        .iter()
        .any(|update| update.family == "guard"));

    let metrics = run_workload(VALID, WorkloadMode::Standard).expect("standard runtime");
    assert_eq!(metrics.guard_encoded_count, 0);
    assert!(metrics.no_guard_encoded);
}

#[test]
fn runtime_run_succeeds_for_smoke_and_standard() {
    let smoke = run_workload(VALID, WorkloadMode::Smoke).expect("smoke runtime");
    let standard = run_workload(VALID, WorkloadMode::Standard).expect("standard runtime");

    assert_eq!(smoke.mode, "smoke");
    assert_eq!(smoke.encoded_segments_total, 12);
    assert!(smoke.rebuild_matches);
    assert_eq!(standard.mode, "standard");
    assert_eq!(standard.workload_family_count, 11);
    assert_eq!(standard.encoded_segments_total, 33);
    assert_eq!(standard.read_count, 33);
    assert_eq!(standard.update_count, 11);
    assert!(standard.rebuild_matches);
    assert!(standard.no_guard_encoded);
}

#[test]
fn p58_metrics_json_contains_required_schema_fields() {
    let json = p58_metrics_json(VALID, WorkloadMode::Standard).expect("p58 metrics json");

    assert!(json.contains("\"astra_iteration\": \"ASTRA-SYS-P58\""));
    assert!(json.contains("\"mode\": \"standard\""));
    assert!(json.contains("\"program_path\": \"<memory>\""));
    assert!(json.contains("\"strict_p53_enabled\": true"));
    assert!(json.contains("\"family_count\": 12"));
    assert!(json.contains("\"active_family_count\": 11"));
    assert!(json.contains("\"refused_family_count\": 1"));
    assert!(json.contains("\"workload_count\": 11"));
    assert!(json.contains("\"workload_family_count\": 11"));
    assert!(json.contains("\"encoded_segments\": 33"));
    assert!(json.contains("\"records\": 33"));
    assert!(json.contains("\"reads\": 33"));
    assert!(json.contains("\"updates\": 11"));
    assert!(json.contains("\"snapshots\": 1"));
    assert!(json.contains("\"rebuilds\": 1"));
    assert!(json.contains("\"no_guard_encoded\": true"));
    assert!(json.contains("\"guard_refused\": true"));
    assert!(json.contains("\"snapshot_full_refused\": true"));
    assert!(json.contains("\"runtime_available\": true"));
    assert!(json.contains("\"metrics_available\": true"));
    assert!(json.contains("\"report_available\": true"));
    assert!(json.contains("\"workloads\": ["));
    assert!(json.contains("\"gates\": {"));
    assert!(json.contains("\"decision\": \"VALIDATE\""));
}

#[test]
fn p58_report_json_contains_required_schema_fields() {
    let json = p58_report_json(VALID, WorkloadMode::Smoke).expect("p58 report json");

    assert!(json.contains("\"astra_iteration\": \"ASTRA-SYS-P58\""));
    assert!(json.contains("\"mode\": \"smoke\""));
    assert!(json.contains("\"workload_count\": 3"));
    assert!(json.contains("\"encoded_segments\": 12"));
    assert!(json.contains("\"P58_G0_runtime_mode_available\": true"));
    assert!(json.contains("\"P58_G1_workload_registry_nonempty\": true"));
    assert!(json.contains("\"P58_G2_standard_covers_active_non_guard_families\": false"));
    assert!(json.contains("\"P58_G3_guard_not_encoded\": true"));
    assert!(json.contains("\"P58_G4_query_success_rate_ok\": true"));
    assert!(json.contains("\"P58_G5_snapshot_rebuild_available\": true"));
    assert!(json.contains("\"P58_G6_metrics_json_stable\": true"));
    assert!(json.contains("\"P58_G7_report_generated\": true"));
    assert!(json.contains("\"decision\": \"RECALIBRATE\""));
}

#[test]
fn p58_report_json_is_deterministic_for_smoke_and_standard() {
    let smoke_first = p58_report_json(VALID, WorkloadMode::Smoke).expect("first smoke report");
    let smoke_second = p58_report_json(VALID, WorkloadMode::Smoke).expect("second smoke report");
    let standard_first =
        p58_report_json(VALID, WorkloadMode::Standard).expect("first standard report");
    let standard_second =
        p58_report_json(VALID, WorkloadMode::Standard).expect("second standard report");

    assert_eq!(smoke_first, smoke_second);
    assert_eq!(standard_first, standard_second);
}

#[test]
fn p58_report_json_matches_stable_golden_outputs() {
    let smoke = p58_report_json_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("p58 smoke report json");
    let standard = p58_report_json_file("examples/p53_strict.atlas", WorkloadMode::Standard)
        .expect("p58 standard report json");
    let expected_smoke = include_str!("golden/p58_report_smoke.json").trim_end_matches('\n');
    let expected_standard = include_str!("golden/p58_report_standard.json").trim_end_matches('\n');

    assert_eq!(smoke, expected_smoke);
    assert_eq!(standard, expected_standard);
}

#[test]
fn p58_report_decisions_are_conservative_by_mode() {
    let smoke = p58_report(VALID, WorkloadMode::Smoke).expect("p58 smoke report");
    let standard = p58_report(VALID, WorkloadMode::Standard).expect("p58 standard report");
    let ambitious = p58_report(VALID, WorkloadMode::Ambitious).expect("p58 ambitious report");

    assert_eq!(smoke.decision, "RECALIBRATE");
    assert_eq!(standard.decision, "VALIDATE");
    assert_eq!(ambitious.decision, "VALIDATE");
    assert!(ambitious
        .warnings
        .iter()
        .any(|warning| warning.contains("local-only")));
}

#[test]
fn p58_standard_report_semantics_are_structural() {
    let report = p58_report(VALID, WorkloadMode::Standard).expect("p58 standard report");

    assert_eq!(report.astra_iteration, "ASTRA-SYS-P58");
    assert_eq!(report.mode, "standard");
    assert!(report.strict_p53_enabled);
    assert_eq!(report.family_count, 12);
    assert_eq!(report.active_family_count, 11);
    assert_eq!(report.refused_family_count, 1);
    assert_eq!(report.workload_count, 11);
    assert_eq!(report.workload_family_count, 11);
    assert_eq!(report.records, 33);
    assert_eq!(report.encoded_segments, 33);
    assert_eq!(report.reads, 33);
    assert_eq!(report.updates, 11);
    assert_eq!(report.query_success_rate, 1.0);
    assert!(report.no_guard_encoded);
    assert!(report.guard_refused);
    assert!(report.snapshot_full_refused);
    assert!(report.gates.p58_g0_runtime_mode_available);
    assert!(report.gates.p58_g1_workload_registry_nonempty);
    assert!(
        report
            .gates
            .p58_g2_standard_covers_active_non_guard_families
    );
    assert!(report.gates.p58_g3_guard_not_encoded);
    assert!(report.gates.p58_g4_query_success_rate_ok);
    assert!(report.gates.p58_g5_snapshot_rebuild_available);
    assert!(report.gates.p58_g6_metrics_json_stable);
    assert!(report.gates.p58_g7_report_generated);
}

#[test]
fn p58_markdown_report_contains_iteration_mode_decision_and_gates() {
    let markdown = p58_report_markdown(VALID, WorkloadMode::Standard).expect("p58 markdown report");

    assert!(markdown.contains("# ASTRA-SYS-P58 runtime report"));
    assert!(markdown.contains("- Mode: `standard`"));
    assert!(markdown.contains("- Decision: `VALIDATE`"));
    assert!(markdown.contains("## Gates summary"));
    assert!(markdown.contains("P58_G0_runtime_mode_available"));
    assert!(markdown.contains("P58_G7_report_generated"));
    assert!(markdown.contains("## Warnings"));
}

#[test]
fn p58_does_not_change_p57_report_golden() {
    let json = p57_report_json(VALID).expect("p57 report json");
    let expected = include_str!("golden/p57_report.json").trim_end_matches('\n');

    assert_eq!(json, expected);
}

#[test]
fn invalid_mode_for_metrics_and_report_is_rejected() {
    for command in ["metrics", "report"] {
        let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
            .args([
                command,
                "examples/p53_strict.atlas",
                "--mode",
                "full",
                "--format",
                "json",
            ])
            .output()
            .expect("run atlas-cli");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("unsupported mode 'full'"));
    }
}
