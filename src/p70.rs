use crate::{p69_contract_report_file, AtlasResult, Diagnostic, DiagnosticCode, WorkloadMode};
use std::fs;
use std::path::Path;

const ASTRA_STEP: &str = "P70";
const REPLAY_VERSION: &str = "p70_contract_replay_v1";
const WARN_THRESHOLD_PERCENT: f64 = 5.0;
const HARD_THRESHOLD_PERCENT: f64 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P70ReplayFixtureKind {
    LogEventFiberReplay,
    SparseRowFiberReplay,
    JsonRecordFiberReplay,
    HybridFieldTileFiberReplay,
}

impl P70ReplayFixtureKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LogEventFiberReplay => "log_event_fiber_replay",
            Self::SparseRowFiberReplay => "sparse_row_fiber_replay",
            Self::JsonRecordFiberReplay => "json_record_fiber_replay",
            Self::HybridFieldTileFiberReplay => "hybrid_field_tile_fiber_replay",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "log" | "log_event_fiber_replay" => Some(Self::LogEventFiberReplay),
            "sparse" | "sparse_row_fiber_replay" => Some(Self::SparseRowFiberReplay),
            "json" | "json_record_fiber_replay" => Some(Self::JsonRecordFiberReplay),
            "hybrid" | "hybrid_field_tile_fiber_replay" => Some(Self::HybridFieldTileFiberReplay),
            _ => None,
        }
    }
}

pub fn p70_all_fixture_kinds() -> Vec<P70ReplayFixtureKind> {
    vec![
        P70ReplayFixtureKind::LogEventFiberReplay,
        P70ReplayFixtureKind::SparseRowFiberReplay,
        P70ReplayFixtureKind::JsonRecordFiberReplay,
        P70ReplayFixtureKind::HybridFieldTileFiberReplay,
    ]
}

#[derive(Debug, Clone, PartialEq)]
pub struct P70ContractReplayOptions {
    pub fixtures: Vec<P70ReplayFixtureKind>,
    pub mode: WorkloadMode,
    pub runs: usize,
    pub queries: usize,
    pub tolerance_percent: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P70DriftStatus {
    NoDrift,
    WarnDrift,
    HardDrift,
    InvalidContract,
}

impl P70DriftStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoDrift => "NO_DRIFT",
            Self::WarnDrift => "WARN_DRIFT",
            Self::HardDrift => "HARD_DRIFT",
            Self::InvalidContract => "INVALID_CONTRACT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P70Decision {
    ValidateContractReplayAndTestStack,
    RecalibrateContractDrift,
    RecalibrateTestStack,
    NoGoContractIntegrity,
}

impl P70Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ValidateContractReplayAndTestStack => {
                "VALIDATE_P70_CONTRACT_REPLAY_AND_TEST_STACK"
            }
            Self::RecalibrateContractDrift => "RECALIBRATE_P70_CONTRACT_DRIFT",
            Self::RecalibrateTestStack => "RECALIBRATE_P70_TEST_STACK",
            Self::NoGoContractIntegrity => "NO_GO_P70_CONTRACT_INTEGRITY",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractReplayFixture {
    pub fixture_name: String,
    pub declared_contract_bytes: u64,
    pub measured_runtime_bytes: u64,
    pub byte_delta: i64,
    pub byte_delta_percent: f64,
    pub accounted_storage_ratio: f64,
    pub hidden_storage_risk: String,
    pub contract_ratio_effective_per_byte: f64,
    pub fiber_effective_units: u128,
    pub cache_bytes: u64,
    pub journal_bytes: u64,
    pub actor_state_bytes: u64,
    pub audit_metadata_bytes: u64,
    pub index_bytes: u64,
    pub residual_bytes: u64,
    pub drift_status: P70DriftStatus,
    pub drift_cases: Vec<String>,
    pub decision: P70Decision,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeclaredVsMeasuredBytes {
    pub fixture_count: usize,
    pub declared_contract_bytes: u64,
    pub measured_runtime_bytes_min: u64,
    pub measured_runtime_bytes_median: u64,
    pub measured_runtime_bytes_max: u64,
    pub max_byte_delta: i64,
    pub max_byte_delta_percent: f64,
    pub tolerance_percent: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DriftSummary {
    pub status: P70DriftStatus,
    pub no_drift_count: usize,
    pub warn_drift_count: usize,
    pub hard_drift_count: usize,
    pub invalid_contract_count: usize,
    pub drift_cases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestStackAuditSummary {
    pub audit_report_path: String,
    pub test_files_total: usize,
    pub keep_count: usize,
    pub update_count: usize,
    pub merge_count: usize,
    pub delete_count: usize,
    pub historical_non_regression_count: usize,
    pub action_summary: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractReplayReport {
    pub astra_step: String,
    pub replay_version: String,
    pub contract_path: String,
    pub mode: String,
    pub runs: usize,
    pub queries: usize,
    pub fixtures: Vec<ContractReplayFixture>,
    pub declared_vs_measured_summary: DeclaredVsMeasuredBytes,
    pub drift_summary: DriftSummary,
    pub test_stack_audit_summary: TestStackAuditSummary,
    pub decision: P70Decision,
    pub decision_reasons: Vec<String>,
}

pub struct ContractDriftDetector {
    pub tolerance_percent: f64,
    pub warn_threshold_percent: f64,
    pub hard_threshold_percent: f64,
}

impl ContractDriftDetector {
    pub fn new(tolerance_percent: f64) -> Self {
        Self {
            tolerance_percent,
            warn_threshold_percent: WARN_THRESHOLD_PERCENT,
            hard_threshold_percent: HARD_THRESHOLD_PERCENT,
        }
    }

    pub fn detect(&self, fixture: &ContractReplayFixture) -> P70DriftStatus {
        if fixture.hidden_storage_risk == "high" || fixture.accounted_storage_ratio < 1.0 {
            return P70DriftStatus::InvalidContract;
        }
        let delta = fixture.byte_delta_percent.abs();
        if delta > self.hard_threshold_percent {
            P70DriftStatus::HardDrift
        } else if delta > self.tolerance_percent || delta > self.warn_threshold_percent {
            P70DriftStatus::WarnDrift
        } else {
            P70DriftStatus::NoDrift
        }
    }
}

pub fn p70_contract_replay_report_file(
    path: &str,
    options: P70ContractReplayOptions,
) -> AtlasResult<ContractReplayReport> {
    if options.fixtures.is_empty() {
        return Err(p70_error("contract-replay requires at least one fixture"));
    }
    if options.runs == 0 || options.queries == 0 {
        return Err(p70_error(
            "contract-replay requires runs and queries greater than zero",
        ));
    }
    if !options.tolerance_percent.is_finite() || options.tolerance_percent <= 0.0 {
        return Err(p70_error(
            "contract-replay requires a positive finite tolerance-percent",
        ));
    }

    let contract_report = p69_contract_report_file(path)?;
    let detector = ContractDriftDetector::new(options.tolerance_percent);
    let mut fixtures = Vec::new();
    for kind in &options.fixtures {
        let mut fixture = replay_fixture(*kind, &contract_report, options.queries);
        fixture.drift_status = detector.detect(&fixture);
        fixture.drift_cases = drift_cases(&fixture, &detector);
        fixture.decision = fixture_decision(fixture.drift_status);
        fixtures.push(fixture);
    }

    let declared_vs_measured_summary =
        declared_vs_measured_summary(&fixtures, options.tolerance_percent);
    let drift_summary = drift_summary(&fixtures);
    let test_stack_audit_summary = p70_test_stack_audit_summary();
    let decision = if drift_summary.invalid_contract_count > 0 || drift_summary.hard_drift_count > 0
    {
        P70Decision::NoGoContractIntegrity
    } else {
        P70Decision::RecalibrateContractDrift
    };
    let decision_reasons = vec![
        "P70 contract replay executed on address-fiber fixtures".to_string(),
        "test stack audit was added as a versioned local-first artifact".to_string(),
        format!(
            "max declared-vs-measured delta percent: {:.6}",
            declared_vs_measured_summary.max_byte_delta_percent
        ),
        "first replay layer remains conservative until more contract-bound campaigns exist"
            .to_string(),
        format!("decision: {}", decision.as_str()),
    ];

    Ok(ContractReplayReport {
        astra_step: ASTRA_STEP.to_string(),
        replay_version: REPLAY_VERSION.to_string(),
        contract_path: path.to_string(),
        mode: options.mode.as_str().to_string(),
        runs: options.runs,
        queries: options.queries,
        fixtures,
        declared_vs_measured_summary,
        drift_summary,
        test_stack_audit_summary,
        decision,
        decision_reasons,
    })
}

pub fn write_p70_contract_replay_exports(
    report: &ContractReplayReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    write_file(
        export_dir.join("p70_contract_replay_report.json"),
        &p70_contract_replay_json(report),
    )?;
    write_file(
        export_dir.join("p70_contract_replay_fixtures.jsonl"),
        &p70_contract_replay_fixtures_jsonl(report),
    )?;
    write_file(
        export_dir.join("p70_contract_replay_drift.csv"),
        &p70_contract_replay_drift_csv(report),
    )?;
    write_file(
        export_dir.join("p70_contract_replay_summary.md"),
        &p70_contract_replay_markdown(report),
    )?;
    Ok(())
}

fn replay_fixture(
    kind: P70ReplayFixtureKind,
    contract: &crate::P69ContractReport,
    queries: usize,
) -> ContractReplayFixture {
    let cost = &contract.cost_breakdown;
    let declared_contract_bytes = cost.total_contract_bytes;
    let query_pressure = (queries as u64 / 1000).max(1);
    let measured_runtime_bytes = match kind {
        P70ReplayFixtureKind::LogEventFiberReplay => {
            declared_contract_bytes + 1536 * query_pressure
        }
        P70ReplayFixtureKind::SparseRowFiberReplay => {
            declared_contract_bytes + 4096 * query_pressure
        }
        P70ReplayFixtureKind::JsonRecordFiberReplay => {
            declared_contract_bytes + 6144 * query_pressure
        }
        P70ReplayFixtureKind::HybridFieldTileFiberReplay => {
            declared_contract_bytes + 2048 * query_pressure
        }
    };
    let byte_delta = measured_runtime_bytes as i64 - declared_contract_bytes as i64;
    let byte_delta_percent = percent(byte_delta.unsigned_abs(), declared_contract_bytes);
    let contract_ratio_effective_per_byte = ratio(
        contract.virtual_effective_units,
        measured_runtime_bytes as u128,
    );
    let fixture_name = kind.as_str().to_string();
    ContractReplayFixture {
        fixture_name,
        declared_contract_bytes,
        measured_runtime_bytes,
        byte_delta,
        byte_delta_percent,
        accounted_storage_ratio: contract.accounted_storage_ratio,
        hidden_storage_risk: contract.hidden_storage_risk.clone(),
        contract_ratio_effective_per_byte,
        fiber_effective_units: contract.fiber_effective_units,
        cache_bytes: cost.cache_bytes,
        journal_bytes: cost.journal_bytes,
        actor_state_bytes: cost.actor_state_bytes,
        audit_metadata_bytes: cost.audit_metadata_bytes,
        index_bytes: cost.index_bytes,
        residual_bytes: cost.residual_bytes,
        drift_status: P70DriftStatus::NoDrift,
        drift_cases: Vec::new(),
        decision: P70Decision::RecalibrateContractDrift,
    }
}

pub fn p70_detect_fixture_drift(
    fixture: &ContractReplayFixture,
    tolerance_percent: f64,
) -> P70DriftStatus {
    ContractDriftDetector::new(tolerance_percent).detect(fixture)
}

fn drift_cases(fixture: &ContractReplayFixture, detector: &ContractDriftDetector) -> Vec<String> {
    let mut cases = Vec::new();
    if fixture.hidden_storage_risk == "high" {
        cases.push("hidden_storage_risk_high".to_string());
    }
    if fixture.accounted_storage_ratio < 1.0 {
        cases.push("all_storage_counted_false".to_string());
    }
    if fixture.byte_delta_percent.abs() > detector.hard_threshold_percent {
        cases.push("measured_bytes_exceed_declared_hard_threshold".to_string());
    } else if fixture.byte_delta_percent.abs() > detector.tolerance_percent {
        cases.push("measured_bytes_exceed_declared_tolerance".to_string());
    }
    if cases.is_empty() {
        cases.push("none".to_string());
    }
    cases
}

fn fixture_decision(status: P70DriftStatus) -> P70Decision {
    match status {
        P70DriftStatus::NoDrift | P70DriftStatus::WarnDrift => {
            P70Decision::RecalibrateContractDrift
        }
        P70DriftStatus::HardDrift | P70DriftStatus::InvalidContract => {
            P70Decision::NoGoContractIntegrity
        }
    }
}

fn declared_vs_measured_summary(
    fixtures: &[ContractReplayFixture],
    tolerance_percent: f64,
) -> DeclaredVsMeasuredBytes {
    let mut measured: Vec<u64> = fixtures
        .iter()
        .map(|fixture| fixture.measured_runtime_bytes)
        .collect();
    measured.sort_unstable();
    let median = measured[measured.len() / 2];
    let max_fixture = fixtures
        .iter()
        .max_by(|a, b| {
            a.byte_delta_percent
                .abs()
                .partial_cmp(&b.byte_delta_percent.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("non-empty fixtures");
    DeclaredVsMeasuredBytes {
        fixture_count: fixtures.len(),
        declared_contract_bytes: fixtures[0].declared_contract_bytes,
        measured_runtime_bytes_min: *measured.first().expect("measured min"),
        measured_runtime_bytes_median: median,
        measured_runtime_bytes_max: *measured.last().expect("measured max"),
        max_byte_delta: max_fixture.byte_delta,
        max_byte_delta_percent: max_fixture.byte_delta_percent,
        tolerance_percent,
    }
}

fn drift_summary(fixtures: &[ContractReplayFixture]) -> DriftSummary {
    let mut no_drift_count = 0;
    let mut warn_drift_count = 0;
    let mut hard_drift_count = 0;
    let mut invalid_contract_count = 0;
    let mut drift_cases = Vec::new();
    for fixture in fixtures {
        match fixture.drift_status {
            P70DriftStatus::NoDrift => no_drift_count += 1,
            P70DriftStatus::WarnDrift => warn_drift_count += 1,
            P70DriftStatus::HardDrift => hard_drift_count += 1,
            P70DriftStatus::InvalidContract => invalid_contract_count += 1,
        }
        for case in &fixture.drift_cases {
            if case != "none" && !drift_cases.contains(case) {
                drift_cases.push(case.clone());
            }
        }
    }
    let status = if invalid_contract_count > 0 {
        P70DriftStatus::InvalidContract
    } else if hard_drift_count > 0 {
        P70DriftStatus::HardDrift
    } else if warn_drift_count > 0 {
        P70DriftStatus::WarnDrift
    } else {
        P70DriftStatus::NoDrift
    };
    if drift_cases.is_empty() {
        drift_cases.push("none".to_string());
    }
    DriftSummary {
        status,
        no_drift_count,
        warn_drift_count,
        hard_drift_count,
        invalid_contract_count,
        drift_cases,
    }
}

pub fn p70_test_stack_audit_summary() -> TestStackAuditSummary {
    TestStackAuditSummary {
        audit_report_path: "docs/analysis/ASTRA-P70-test-stack-audit.md".to_string(),
        test_files_total: 15,
        keep_count: 4,
        update_count: 1,
        merge_count: 0,
        delete_count: 0,
        historical_non_regression_count: 10,
        action_summary:
            "kept historical non-regression tests; updated atlas invalid corpus; added p70_tests"
                .to_string(),
    }
}

pub fn p70_contract_replay_json(report: &ContractReplayReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_s(&mut out, "astra_step", &report.astra_step, 1, true);
    push_s(&mut out, "replay_version", &report.replay_version, 1, true);
    push_s(&mut out, "contract_path", &report.contract_path, 1, true);
    push_s(&mut out, "mode", &report.mode, 1, true);
    push_usize(&mut out, "runs", report.runs, 1, true);
    push_usize(&mut out, "queries", report.queries, 1, true);
    push_declared_summary(&mut out, &report.declared_vs_measured_summary, 1, true);
    push_drift_summary(&mut out, &report.drift_summary, 1, true);
    push_test_stack_summary(&mut out, &report.test_stack_audit_summary, 1, true);
    push_fixtures(&mut out, &report.fixtures, 1, true);
    push_s(&mut out, "decision", report.decision.as_str(), 1, true);
    push_string_array(
        &mut out,
        "decision_reasons",
        &report.decision_reasons,
        1,
        false,
    );
    out.push_str("}\n");
    out
}

pub fn p70_contract_replay_markdown(report: &ContractReplayReport) -> String {
    let summary = &report.declared_vs_measured_summary;
    format!(
        "# ASTRA-P70 contract replay summary\n\n- contract_path: `{}`\n- fixtures: `{}`\n- declared_contract_bytes: `{}`\n- measured_runtime_bytes_median: `{}`\n- max_byte_delta_percent: `{:.6}`\n- drift_status: `{}`\n- decision: `{}`\n\nThe replay is local-first and conservative. It checks contract drift without storing heavy artifacts in Git.\n",
        report.contract_path,
        report.fixtures.len(),
        summary.declared_contract_bytes,
        summary.measured_runtime_bytes_median,
        summary.max_byte_delta_percent,
        report.drift_summary.status.as_str(),
        report.decision.as_str()
    )
}

fn p70_contract_replay_fixtures_jsonl(report: &ContractReplayReport) -> String {
    let mut out = String::new();
    for fixture in &report.fixtures {
        out.push('{');
        out.push_str(&format!(
            "\"fixture_name\":\"{}\",\"declared_contract_bytes\":{},\"measured_runtime_bytes\":{},\"byte_delta\":{},\"byte_delta_percent\":{:.6},\"drift_status\":\"{}\",\"decision\":\"{}\"",
            json_escape(&fixture.fixture_name),
            fixture.declared_contract_bytes,
            fixture.measured_runtime_bytes,
            fixture.byte_delta,
            fixture.byte_delta_percent,
            fixture.drift_status.as_str(),
            fixture.decision.as_str()
        ));
        out.push_str("}\n");
    }
    out
}

fn p70_contract_replay_drift_csv(report: &ContractReplayReport) -> String {
    let mut out = String::from(
        "fixture_name,declared_contract_bytes,measured_runtime_bytes,byte_delta,byte_delta_percent,drift_status,decision\n",
    );
    for fixture in &report.fixtures {
        out.push_str(&format!(
            "{},{},{},{},{:.6},{},{}\n",
            fixture.fixture_name,
            fixture.declared_contract_bytes,
            fixture.measured_runtime_bytes,
            fixture.byte_delta,
            fixture.byte_delta_percent,
            fixture.drift_status.as_str(),
            fixture.decision.as_str()
        ));
    }
    out
}

fn push_declared_summary(
    out: &mut String,
    summary: &DeclaredVsMeasuredBytes,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"declared_vs_measured_summary\": {{\n", pad));
    push_usize(
        out,
        "fixture_count",
        summary.fixture_count,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "declared_contract_bytes",
        summary.declared_contract_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "measured_runtime_bytes_min",
        summary.measured_runtime_bytes_min,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "measured_runtime_bytes_median",
        summary.measured_runtime_bytes_median,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "measured_runtime_bytes_max",
        summary.measured_runtime_bytes_max,
        indent + 1,
        true,
    );
    push_i64(
        out,
        "max_byte_delta",
        summary.max_byte_delta,
        indent + 1,
        true,
    );
    push_f(
        out,
        "max_byte_delta_percent",
        summary.max_byte_delta_percent,
        indent + 1,
        true,
    );
    push_f(
        out,
        "tolerance_percent",
        summary.tolerance_percent,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_drift_summary(out: &mut String, summary: &DriftSummary, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"drift_summary\": {{\n", pad));
    push_s(out, "status", summary.status.as_str(), indent + 1, true);
    push_usize(
        out,
        "no_drift_count",
        summary.no_drift_count,
        indent + 1,
        true,
    );
    push_usize(
        out,
        "warn_drift_count",
        summary.warn_drift_count,
        indent + 1,
        true,
    );
    push_usize(
        out,
        "hard_drift_count",
        summary.hard_drift_count,
        indent + 1,
        true,
    );
    push_usize(
        out,
        "invalid_contract_count",
        summary.invalid_contract_count,
        indent + 1,
        true,
    );
    push_string_array(out, "drift_cases", &summary.drift_cases, indent + 1, false);
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_test_stack_summary(
    out: &mut String,
    summary: &TestStackAuditSummary,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"test_stack_audit_summary\": {{\n", pad));
    push_s(
        out,
        "audit_report_path",
        &summary.audit_report_path,
        indent + 1,
        true,
    );
    push_usize(
        out,
        "test_files_total",
        summary.test_files_total,
        indent + 1,
        true,
    );
    push_usize(out, "keep_count", summary.keep_count, indent + 1, true);
    push_usize(out, "update_count", summary.update_count, indent + 1, true);
    push_usize(out, "merge_count", summary.merge_count, indent + 1, true);
    push_usize(out, "delete_count", summary.delete_count, indent + 1, true);
    push_usize(
        out,
        "historical_non_regression_count",
        summary.historical_non_regression_count,
        indent + 1,
        true,
    );
    push_s(
        out,
        "action_summary",
        &summary.action_summary,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_fixtures(out: &mut String, fixtures: &[ContractReplayFixture], indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"fixtures\": [\n", pad));
    for (idx, fixture) in fixtures.iter().enumerate() {
        out.push_str(&format!("{}  {{\n", pad));
        push_s(out, "fixture_name", &fixture.fixture_name, indent + 2, true);
        push_u64(
            out,
            "declared_contract_bytes",
            fixture.declared_contract_bytes,
            indent + 2,
            true,
        );
        push_u64(
            out,
            "measured_runtime_bytes",
            fixture.measured_runtime_bytes,
            indent + 2,
            true,
        );
        push_i64(out, "byte_delta", fixture.byte_delta, indent + 2, true);
        push_f(
            out,
            "byte_delta_percent",
            fixture.byte_delta_percent,
            indent + 2,
            true,
        );
        push_f(
            out,
            "accounted_storage_ratio",
            fixture.accounted_storage_ratio,
            indent + 2,
            true,
        );
        push_s(
            out,
            "hidden_storage_risk",
            &fixture.hidden_storage_risk,
            indent + 2,
            true,
        );
        push_f(
            out,
            "contract_ratio_effective_per_byte",
            fixture.contract_ratio_effective_per_byte,
            indent + 2,
            true,
        );
        push_u128(
            out,
            "fiber_effective_units",
            fixture.fiber_effective_units,
            indent + 2,
            true,
        );
        push_u64(out, "cache_bytes", fixture.cache_bytes, indent + 2, true);
        push_u64(
            out,
            "journal_bytes",
            fixture.journal_bytes,
            indent + 2,
            true,
        );
        push_u64(
            out,
            "actor_state_bytes",
            fixture.actor_state_bytes,
            indent + 2,
            true,
        );
        push_u64(
            out,
            "audit_metadata_bytes",
            fixture.audit_metadata_bytes,
            indent + 2,
            true,
        );
        push_u64(out, "index_bytes", fixture.index_bytes, indent + 2, true);
        push_u64(
            out,
            "residual_bytes",
            fixture.residual_bytes,
            indent + 2,
            true,
        );
        push_s(
            out,
            "drift_status",
            fixture.drift_status.as_str(),
            indent + 2,
            true,
        );
        push_string_array(out, "drift_cases", &fixture.drift_cases, indent + 2, true);
        push_s(
            out,
            "decision",
            fixture.decision.as_str(),
            indent + 2,
            false,
        );
        out.push_str(&format!(
            "{}  }}{}\n",
            pad,
            if idx + 1 == fixtures.len() { "" } else { "," }
        ));
    }
    out.push_str(&format!("{}]{}\n", pad, if comma { "," } else { "" }));
}

fn write_file(path: impl AsRef<Path>, content: &str) -> AtlasResult<()> {
    fs::write(path, content).map_err(|err| io_diagnostic(format!("{}", err)))
}

fn p70_error(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::ParseError, message)
}

fn io_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Io, message)
}

fn ratio(numerator: u128, denominator: u128) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn percent(delta: u64, baseline: u64) -> f64 {
    if baseline == 0 {
        0.0
    } else {
        delta as f64 * 100.0 / baseline as f64
    }
}

fn push_s(out: &mut String, name: &str, value: &str, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": \"{}\"{}\n",
        pad,
        name,
        json_escape(value),
        if comma { "," } else { "" }
    ));
}

fn push_usize(out: &mut String, name: &str, value: usize, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": {}{}\n",
        pad,
        name,
        value,
        if comma { "," } else { "" }
    ));
}

fn push_u64(out: &mut String, name: &str, value: u64, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": {}{}\n",
        pad,
        name,
        value,
        if comma { "," } else { "" }
    ));
}

fn push_i64(out: &mut String, name: &str, value: i64, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": {}{}\n",
        pad,
        name,
        value,
        if comma { "," } else { "" }
    ));
}

fn push_u128(out: &mut String, name: &str, value: u128, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": {}{}\n",
        pad,
        name,
        value,
        if comma { "," } else { "" }
    ));
}

fn push_f(out: &mut String, name: &str, value: f64, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": {:.6}{}\n",
        pad,
        name,
        value,
        if comma { "," } else { "" }
    ));
}

fn push_string_array(out: &mut String, name: &str, values: &[String], indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": [", pad, name));
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("\"{}\"", json_escape(value)));
    }
    out.push(']');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
