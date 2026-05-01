use astra_atlas_lang::{bench_report_json_file, WorkloadMode};
use std::process::Command;

#[test]
fn p60_bench_report_json_is_stable_for_standard() {
    let first = bench_report_json_file("examples/p53_strict.atlas", WorkloadMode::Standard)
        .expect("first p60 bench report");
    let second = bench_report_json_file("examples/p53_strict.atlas", WorkloadMode::Standard)
        .expect("second p60 bench report");

    assert_eq!(first, second);
    assert!(first.contains("\"astra_iteration\": \"ASTRA-SYS-P60\""));
    assert!(first.contains("\"benchmark_kind\": \"deterministic_structural_proxy\""));
    assert!(first.contains("\"program_path\": \"examples/p53_strict.atlas\""));
    assert!(first.contains("\"mode\": \"standard\""));
    assert!(first.contains("\"family_count\": 12"));
    assert!(first.contains("\"workload_count\": 11"));
    assert!(first.contains("\"encoded_segments\": 33"));
    assert!(first.contains("\"reads\": 33"));
    assert!(first.contains("\"updates\": 11"));
    assert!(first.contains("\"elapsed_ms\": null"));
    assert!(first.contains("\"p50_proxy_cost_units\":"));
    assert!(first.contains("\"p95_proxy_cost_units\":"));
    assert!(first.contains("\"p99_proxy_cost_units\":"));
    assert!(first.contains("\"decision\": \"VALIDATE\""));
    assert!(first.contains("not a wall-clock or industrial performance benchmark"));
}

#[test]
fn p60_bench_report_decisions_follow_mode_scope() {
    let smoke = bench_report_json_file("examples/p53_strict.atlas", WorkloadMode::Smoke)
        .expect("smoke p60 bench report");
    let standard = bench_report_json_file("examples/p53_strict.atlas", WorkloadMode::Standard)
        .expect("standard p60 bench report");
    let ambitious = bench_report_json_file("examples/p53_strict.atlas", WorkloadMode::Ambitious)
        .expect("ambitious p60 bench report");

    assert!(smoke.contains("\"decision\": \"RECALIBRATE\""));
    assert!(smoke.contains("\"ci_safe\": true"));
    assert!(standard.contains("\"decision\": \"VALIDATE\""));
    assert!(standard.contains("\"local_manual_only\": false"));
    assert!(ambitious.contains("\"decision\": \"VALIDATE\""));
    assert!(ambitious.contains("\"local_manual_only\": true"));
    assert!(ambitious.contains("ambitious mode is local/manual"));
}

#[test]
fn bench_cli_supports_json_without_changing_text_output() {
    let text = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["bench", "--mode", "standard"])
        .output()
        .expect("run text bench");
    assert!(text.status.success());
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("OK: bench standard"));
    assert!(stdout.contains("encoded_segments=33"));

    let json = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["bench", "--mode", "standard", "--format", "json"])
        .output()
        .expect("run json bench");
    assert!(json.status.success());
    let stdout = String::from_utf8_lossy(&json.stdout);
    assert!(stdout.contains("\"astra_iteration\": \"ASTRA-SYS-P60\""));
    assert!(stdout.contains("\"mode\": \"standard\""));
    assert!(stdout.contains("\"decision\": \"VALIDATE\""));
}
