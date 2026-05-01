use astra_atlas_lang::{
    p69_contract_check_json_file, p69_contract_report_file, p69_contract_run_report_file,
    p69_parse_contract_file, validate_file, write_p69_contract_exports, P69ContractRunOptions,
    P69Decision, WorkloadMode,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID: &str = "examples/valid/p69_address_fiber_contract.atlas";

#[test]
fn p69_contract_parses_and_typechecks() {
    let contract = p69_parse_contract_file(VALID).expect("p69 contract parses");

    assert_eq!(contract.code_form_id, "address_fiber_contract");
    assert_eq!(contract.architecture_id, "address_fiber_actor_managed_v1");
    assert_eq!(contract.fiber_schema.projection, "shallow");
    assert_eq!(contract.actor_policy.cache, "compact");
    assert!(contract.gates.all_storage_counted);
}

#[test]
fn p69_contract_check_report_contains_cost_breakdown() {
    let report = p69_contract_report_file(VALID).expect("p69 report");

    assert_eq!(report.astra_step, "P69");
    assert!(report.parse_ok);
    assert!(report.typecheck_ok);
    assert!(report.all_storage_counted);
    assert_eq!(report.hidden_storage_risk, "low");
    assert_eq!(report.cost_breakdown.generator_code_bytes, 4096);
    assert_eq!(report.cost_breakdown.parameter_bytes, 16384);
    assert_eq!(report.cost_breakdown.index_bytes, 24576);
    assert_eq!(report.cost_breakdown.journal_bytes, 4096);
    assert_eq!(report.cost_breakdown.actor_state_bytes, 12288);
    assert_eq!(report.cost_breakdown.audit_metadata_bytes, 4096);
    assert!(report.cost_breakdown.total_contract_bytes > 0);
    assert!(report.contract_ratio_effective_per_byte > 0.0);
    assert_eq!(
        report.decision,
        P69Decision::PromoteAddressFiberContractRuntime
    );
}

#[test]
fn p69_contract_check_json_is_stable_shape() {
    let json = p69_contract_check_json_file(VALID).expect("p69 json");

    assert!(json.contains("\"astra_step\": \"P69\""));
    assert!(json.contains("\"contract_id\": \"address_fiber_contract\""));
    assert!(json.contains("\"all_storage_counted\": true"));
    assert!(json.contains("\"cost_breakdown\""));
    assert!(json.contains("PROMOTE_P69_ADDRESS_FIBER_CONTRACT_RUNTIME"));
}

#[test]
fn p69_contract_run_writes_exports() {
    let report = p69_contract_run_report_file(
        VALID,
        P69ContractRunOptions {
            mode: WorkloadMode::Standard,
            runs: 30,
            queries: 1000,
        },
    )
    .expect("p69 contract run");
    let export_dir = temp_dir("exports");
    write_p69_contract_exports(&report, &export_dir).expect("write p69 exports");

    assert!(export_dir.join("p69_contract_report.json").exists());
    assert!(export_dir.join("p69_contract_cost_breakdown.csv").exists());
    assert!(export_dir.join("p69_contract_summary.md").exists());
    let csv = fs::read_to_string(export_dir.join("p69_contract_cost_breakdown.csv")).expect("csv");
    assert!(csv.contains("generator_code_bytes"));
    assert!(csv.contains("actor_state_bytes"));
    assert!(csv.contains("audit_metadata_bytes"));
}

#[test]
fn p69_invalid_contracts_are_refused() {
    let cases = [
        "examples/invalid/p69_missing_fiber_schema.atlas",
        "examples/invalid/p69_unknown_generator.atlas",
        "examples/invalid/p69_unaccounted_actor_state.atlas",
        "examples/invalid/p69_missing_all_storage_counted_gate.atlas",
        "examples/invalid/p69_zero_budget_actor.atlas",
        "examples/invalid/p69_unknown_projection.atlas",
        "examples/invalid/p69_contract_unknown_reference.atlas",
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
fn p69_cli_contract_check_and_run_succeed() {
    let export_dir = temp_dir("cli");
    let check = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["contract-check", VALID, "--format", "json"])
        .output()
        .expect("contract-check");
    assert!(
        check.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(String::from_utf8_lossy(&check.stdout).contains("\"astra_step\": \"P69\""));

    let run = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "contract-run",
            VALID,
            "--mode",
            "standard",
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
        .expect("contract-run");
    assert!(
        run.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(export_dir.join("p69_contract_report.json").exists());
}

#[test]
fn p69_cli_check_accepts_contract_and_keeps_p53_working() {
    let contract = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["check", VALID])
        .output()
        .expect("check p69");
    assert!(
        contract.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&contract.stderr)
    );
    assert!(String::from_utf8_lossy(&contract.stdout).contains("p69_contract"));

    let program = validate_file("examples/p53_strict.atlas").expect("p53 strict still passes");
    assert_eq!(program.families.len(), 12);
}

#[test]
fn p69_cli_check_rejects_invalid_contracts() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "check",
            "examples/invalid/p69_unaccounted_actor_state.atlas",
        ])
        .output()
        .expect("check invalid p69");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E_PARSE"));
    assert!(stderr.contains("actor_state"));
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "astra-p69-test-{}-{}-{}",
        std::process::id(),
        label,
        nanos
    ))
}
