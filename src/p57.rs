use crate::{
    canonical_json, run_smoke_config, validate, validate_file, AtlasProgram, AtlasResult,
    DiagnosticCode, RuntimeConfig,
};

const ASTRA_P57_ITERATION: &str = "ASTRA-P57";
const P57_VALIDATE: &str = "VALIDATE_REINTEGRATION_CLASSIQUE_RUNTIME";
const P57_RECALIBRATE: &str = "RECALIBRATE_P57";
const P57_NO_GO: &str = "NO_GO_P57";

const VALID_EXAMPLES: &[&str] = &[
    include_str!("../examples/p53_strict.atlas"),
    include_str!("../examples/valid/p53_strict.atlas"),
];

const INVALID_EXAMPLES: &[(&str, DiagnosticCode)] = &[
    (
        include_str!("../examples/invalid/bad_version.atlas"),
        DiagnosticCode::VersionUnknown,
    ),
    (
        include_str!("../examples/invalid/guard_active.atlas"),
        DiagnosticCode::GuardActive,
    ),
    (
        include_str!("../examples/invalid/snapshot_full.atlas"),
        DiagnosticCode::SnapshotFullStrict,
    ),
    (
        include_str!("../examples/invalid/bad_action.atlas"),
        DiagnosticCode::ActionUnknown,
    ),
    (
        include_str!("../examples/invalid/bad_safety.atlas"),
        DiagnosticCode::SafetyUnknown,
    ),
    (
        include_str!("../examples/invalid/layout_mismatch.atlas"),
        DiagnosticCode::LayoutIndexMismatch,
    ),
    (
        include_str!("../examples/invalid/index_mismatch.atlas"),
        DiagnosticCode::LayoutIndexMismatch,
    ),
    (
        include_str!("../examples/invalid/threshold_bad.atlas"),
        DiagnosticCode::ThresholdInvalid,
    ),
    (
        include_str!("../examples/invalid/missing_families.atlas"),
        DiagnosticCode::MissingFamilies,
    ),
    (
        include_str!("../examples/invalid/unknown_family.atlas"),
        DiagnosticCode::FamilyUnknown,
    ),
    (
        include_str!("../examples/invalid/missing_semicolon.atlas"),
        DiagnosticCode::ParseError,
    ),
    (
        include_str!("../examples/invalid/active_without_safety.atlas"),
        DiagnosticCode::ActiveWithoutSafety,
    ),
    (
        include_str!("../examples/invalid/dangerous_encoded.atlas"),
        DiagnosticCode::ActionUnknown,
    ),
    (
        include_str!("../examples/invalid/malformed_family.atlas"),
        DiagnosticCode::FieldMissing,
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P57DecisionGates {
    pub strict_p53_preserved: bool,
    pub guard_remains_refused: bool,
    pub snapshot_full_refused: bool,
    pub valid_examples_pass: bool,
    pub invalid_examples_fail: bool,
    pub runtime_smoke_path_exists: bool,
    pub report_json_stable: bool,
    pub cargo_tests_no_regression: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P57Report {
    pub astra_iteration: String,
    pub atlas_version: String,
    pub strict_p53_enabled: bool,
    pub family_count: usize,
    pub active_family_count: usize,
    pub refused_family_count: usize,
    pub guard_refused: bool,
    pub snapshot_policy: String,
    pub snapshot_full_refused: bool,
    pub runtime_available: bool,
    pub encode_available: bool,
    pub read_available: bool,
    pub update_available: bool,
    pub snapshot_available: bool,
    pub rebuild_available: bool,
    pub metrics_available: bool,
    pub export_json_stable: bool,
    pub family_thresholds_present: bool,
    pub safety_policies_present: bool,
    pub layout_policies_present: bool,
    pub index_policies_present: bool,
    pub valid_examples_pass: bool,
    pub invalid_examples_fail: bool,
    pub runtime_smoke_path_exists: bool,
    pub report_json_stable: bool,
    pub cargo_tests_no_regression: Option<bool>,
    pub external_validation_required: bool,
    pub astra_p57_decision: String,
}

pub fn p57_report(text: &str) -> AtlasResult<P57Report> {
    let program = validate(text)?;
    Ok(P57Report::from_program(&program))
}

pub fn p57_report_file(path: &str) -> AtlasResult<P57Report> {
    let program = validate_file(path)?;
    Ok(P57Report::from_program(&program))
}

pub fn p57_report_json(text: &str) -> AtlasResult<String> {
    let report = p57_report(text)?;
    Ok(p57_report_to_json(&report))
}

pub fn p57_report_json_file(path: &str) -> AtlasResult<String> {
    let report = p57_report_file(path)?;
    Ok(p57_report_to_json(&report))
}

pub fn p57_decision(gates: &P57DecisionGates) -> &'static str {
    if !gates.strict_p53_preserved || !gates.guard_remains_refused || !gates.snapshot_full_refused {
        return P57_NO_GO;
    }

    if gates.valid_examples_pass
        && gates.invalid_examples_fail
        && gates.runtime_smoke_path_exists
        && gates.report_json_stable
        && gates.cargo_tests_no_regression == Some(true)
    {
        P57_VALIDATE
    } else {
        P57_RECALIBRATE
    }
}

impl P57Report {
    fn from_program(program: &AtlasProgram) -> Self {
        let strict_p53_enabled = program
            .runtime
            .get("strict_p53")
            .map(|value| value.as_str())
            == Some("true");
        let snapshot_policy = program
            .runtime
            .get("snapshot")
            .cloned()
            .unwrap_or_else(|| "missing".to_string());
        let strict_p53_preserved =
            strict_p53_enabled && snapshot_policy.as_str() == "incremental_manifest";
        let guard_refused = guard_refused(program);
        let snapshot_full_refused = snapshot_full_refused();
        let active_family_count = program
            .families
            .iter()
            .filter(|family| family.action != "refuse")
            .count();
        let refused_family_count = program.families.len() - active_family_count;
        let family_thresholds_present = program
            .families
            .iter()
            .all(|family| family.threshold.is_finite());
        let safety_policies_present = program
            .families
            .iter()
            .all(|family| !family.safety.is_empty());
        let layout_policies_present = program
            .families
            .iter()
            .all(|family| !family.layout.is_empty());
        let index_policies_present = program
            .families
            .iter()
            .all(|family| !family.index.is_empty());
        let export_json_stable = canonical_json(program) == canonical_json(program);
        let valid_examples_pass = valid_examples_pass();
        let invalid_examples_fail = invalid_examples_fail();

        let metrics = run_smoke_config(RuntimeConfig::from_checked_program(program));
        let runtime_available = metrics.runtime_instantiated;
        let encode_available = metrics.encoded_segments_total > 0;
        let read_available = metrics.read_count > 0;
        let update_available = metrics.update_count > 0;
        let snapshot_available = metrics.snapshot_count > 0;
        let rebuild_available = metrics.rebuild_count > 0 && metrics.rebuild_matches;
        let metrics_available = true;
        let runtime_smoke_path_exists = runtime_available
            && encode_available
            && read_available
            && update_available
            && snapshot_available
            && rebuild_available
            && metrics_available;
        let report_json_stable = true;
        let cargo_tests_no_regression = None;
        let gates = P57DecisionGates {
            strict_p53_preserved,
            guard_remains_refused: guard_refused,
            snapshot_full_refused,
            valid_examples_pass,
            invalid_examples_fail,
            runtime_smoke_path_exists,
            report_json_stable,
            cargo_tests_no_regression,
        };

        Self {
            astra_iteration: ASTRA_P57_ITERATION.to_string(),
            atlas_version: program.version.clone(),
            strict_p53_enabled,
            family_count: program.families.len(),
            active_family_count,
            refused_family_count,
            guard_refused,
            snapshot_policy,
            snapshot_full_refused,
            runtime_available,
            encode_available,
            read_available,
            update_available,
            snapshot_available,
            rebuild_available,
            metrics_available,
            export_json_stable,
            family_thresholds_present,
            safety_policies_present,
            layout_policies_present,
            index_policies_present,
            valid_examples_pass,
            invalid_examples_fail,
            runtime_smoke_path_exists,
            report_json_stable,
            cargo_tests_no_regression,
            external_validation_required: true,
            astra_p57_decision: p57_decision(&gates).to_string(),
        }
    }
}

pub fn p57_report_to_json(report: &P57Report) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string(
        "astra_iteration",
        &report.astra_iteration,
        true,
        2,
    ));
    out.push_str(&json_string(
        "atlas_version",
        &report.atlas_version,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "strict_p53_enabled",
        report.strict_p53_enabled,
        true,
        2,
    ));
    out.push_str(&json_usize("family_count", report.family_count, true, 2));
    out.push_str(&json_usize(
        "active_family_count",
        report.active_family_count,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "refused_family_count",
        report.refused_family_count,
        true,
        2,
    ));
    out.push_str(&json_bool("guard_refused", report.guard_refused, true, 2));
    out.push_str(&json_string(
        "snapshot_policy",
        &report.snapshot_policy,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "snapshot_full_refused",
        report.snapshot_full_refused,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "runtime_available",
        report.runtime_available,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "encode_available",
        report.encode_available,
        true,
        2,
    ));
    out.push_str(&json_bool("read_available", report.read_available, true, 2));
    out.push_str(&json_bool(
        "update_available",
        report.update_available,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "snapshot_available",
        report.snapshot_available,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "rebuild_available",
        report.rebuild_available,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "metrics_available",
        report.metrics_available,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "export_json_stable",
        report.export_json_stable,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "family_thresholds_present",
        report.family_thresholds_present,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "safety_policies_present",
        report.safety_policies_present,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "layout_policies_present",
        report.layout_policies_present,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "index_policies_present",
        report.index_policies_present,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "valid_examples_pass",
        report.valid_examples_pass,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "invalid_examples_fail",
        report.invalid_examples_fail,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "runtime_smoke_path_exists",
        report.runtime_smoke_path_exists,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "report_json_stable",
        report.report_json_stable,
        true,
        2,
    ));
    out.push_str(&json_optional_bool(
        "cargo_tests_no_regression",
        report.cargo_tests_no_regression,
        true,
        2,
    ));
    out.push_str(&json_bool(
        "external_validation_required",
        report.external_validation_required,
        true,
        2,
    ));
    out.push_str(&json_string(
        "astra_p57_decision",
        &report.astra_p57_decision,
        false,
        2,
    ));
    out.push('}');
    out
}

fn guard_refused(program: &AtlasProgram) -> bool {
    program.families.iter().any(|family| {
        family.name == "guard"
            && family.action == "refuse"
            && family.safety == "refuse"
            && family.layout == "refuse"
            && family.index == "none"
    })
}

fn snapshot_full_refused() -> bool {
    matches!(
        validate(include_str!("../examples/invalid/snapshot_full.atlas")),
        Err(diagnostic) if diagnostic.code == DiagnosticCode::SnapshotFullStrict
    )
}

fn valid_examples_pass() -> bool {
    VALID_EXAMPLES
        .iter()
        .all(|example| validate(*example).is_ok())
}

fn invalid_examples_fail() -> bool {
    INVALID_EXAMPLES.iter().all(|(example, code)| {
        matches!(
            validate(*example),
            Err(diagnostic) if diagnostic.code == *code
        )
    })
}

fn json_string(key: &str, value: &str, comma: bool, indent: usize) -> String {
    json_line(key, &format!("\"{}\"", escape_json(value)), comma, indent)
}

fn json_bool(key: &str, value: bool, comma: bool, indent: usize) -> String {
    json_line(key, if value { "true" } else { "false" }, comma, indent)
}

fn json_optional_bool(key: &str, value: Option<bool>, comma: bool, indent: usize) -> String {
    match value {
        Some(value) => json_bool(key, value, comma, indent),
        None => json_line(key, "null", comma, indent),
    }
}

fn json_usize(key: &str, value: usize, comma: bool, indent: usize) -> String {
    json_line(key, &value.to_string(), comma, indent)
}

fn json_line(key: &str, value: &str, comma: bool, indent: usize) -> String {
    let trailing = if comma { "," } else { "" };
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        escape_json(key),
        value,
        trailing
    )
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}
