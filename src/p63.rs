use crate::{
    p62_real_ratio_report_file_with_runs, AtlasResult, Diagnostic, DiagnosticCode, P62MeasuredRun,
    P62RealRatioReport, WorkloadMode,
};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P63ThresholdProfile {
    P63,
}

impl P63ThresholdProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            P63ThresholdProfile::P63 => "p63",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "p63" => Some(P63ThresholdProfile::P63),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P63Decision {
    ValidateMeasuredRatioCalibration,
    RecalibrateThresholds,
    RecalibrateWorkloads,
    NoGoMeasuredRatioStability,
}

impl P63Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            P63Decision::ValidateMeasuredRatioCalibration => {
                "VALIDATE_P63_MEASURED_RATIO_CALIBRATION"
            }
            P63Decision::RecalibrateThresholds => "RECALIBRATE_P63_THRESHOLDS",
            P63Decision::RecalibrateWorkloads => "RECALIBRATE_P63_WORKLOADS",
            P63Decision::NoGoMeasuredRatioStability => "NO_GO_P63_MEASURED_RATIO_STABILITY",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct P63RobustMetric {
    pub min: f64,
    pub median: f64,
    pub max: f64,
    pub mean: f64,
    pub stddev: f64,
    pub coefficient_of_variation: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P63CampaignSummary {
    pub ratio_effective_per_byte: P63RobustMetric,
    pub total_persisted_bytes: P63RobustMetric,
    pub operation_count: P63RobustMetric,
    pub read_p99_us: P63RobustMetric,
    pub update_p99_us: P63RobustMetric,
    pub snapshot_p99_us: P63RobustMetric,
    pub rebuild_p99_us: P63RobustMetric,
    pub audit_p99_us: P63RobustMetric,
    pub all_runs_passed: bool,
    pub run_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P63MachineMetadata {
    pub os: String,
    pub arch: String,
    pub cpu_count: Option<usize>,
    pub rustc_version: String,
    pub cargo_version: String,
    pub git_commit: String,
    pub git_dirty: Option<bool>,
    pub timestamp_utc: String,
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P63CampaignReport {
    pub campaign_id: String,
    pub measurement_id: String,
    pub astra_step: String,
    pub mode: String,
    pub program_path: String,
    pub cost_model: String,
    pub measurement_kind: String,
    pub threshold_profile: String,
    pub repeat_count: usize,
    pub operation_count: usize,
    pub machine_metadata: P63MachineMetadata,
    pub summary: P63CampaignSummary,
    pub runs: Vec<P62MeasuredRun>,
    pub decision: P63Decision,
    pub decision_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn p63_campaign_report_file_with_runs(
    path: &str,
    mode: WorkloadMode,
    repeat_count: usize,
    threshold_profile: P63ThresholdProfile,
) -> AtlasResult<P63CampaignReport> {
    let measured = p62_real_ratio_report_file_with_runs(path, mode, repeat_count)?;
    Ok(P63CampaignReport::from_p62_report(
        measured,
        threshold_profile,
    ))
}

impl P63CampaignReport {
    pub fn from_p62_report(
        measured: P62RealRatioReport,
        threshold_profile: P63ThresholdProfile,
    ) -> Self {
        let summary = P63CampaignSummary::from_runs(&measured.runs, measured.operation_count);
        let decision = p63_decision(&summary, &measured);
        let decision_reasons = p63_decision_reasons(decision, &summary);
        Self {
            campaign_id: format!("p63-{}", measured.measurement_id),
            measurement_id: measured.measurement_id,
            astra_step: "P63".to_string(),
            mode: measured.mode,
            program_path: measured.program_path,
            cost_model: measured.cost_model,
            measurement_kind: measured.measurement_kind,
            threshold_profile: threshold_profile.as_str().to_string(),
            repeat_count: measured.repeat_count,
            operation_count: measured.operation_count,
            machine_metadata: P63MachineMetadata::collect(),
            summary,
            runs: measured.runs,
            decision,
            decision_reasons,
            warnings: vec![
                "P63 campaign export uses measured_real_v1 inherited from ratio-real".to_string(),
                "timings are machine-dependent and must not be goldenized".to_string(),
                "threshold_profile p63 is conservative and not scientifically calibrated yet"
                    .to_string(),
                "no external dataset is included in this campaign export".to_string(),
            ],
        }
    }
}

impl P63CampaignSummary {
    fn from_runs(runs: &[P62MeasuredRun], operation_count: usize) -> Self {
        Self {
            ratio_effective_per_byte: robust_metric(
                runs.iter()
                    .map(|run| run.ratio_effective_per_byte)
                    .collect(),
            ),
            total_persisted_bytes: robust_metric(
                runs.iter()
                    .map(|run| run.total_persisted_bytes as f64)
                    .collect(),
            ),
            operation_count: robust_metric(runs.iter().map(|_| operation_count as f64).collect()),
            read_p99_us: robust_metric(
                runs.iter()
                    .map(|run| run.read_timing.p99_us as f64)
                    .collect(),
            ),
            update_p99_us: robust_metric(
                runs.iter()
                    .map(|run| run.update_timing.p99_us as f64)
                    .collect(),
            ),
            snapshot_p99_us: robust_metric(
                runs.iter()
                    .map(|run| run.snapshot_timing.p99_us as f64)
                    .collect(),
            ),
            rebuild_p99_us: robust_metric(
                runs.iter()
                    .map(|run| run.rebuild_timing.p99_us as f64)
                    .collect(),
            ),
            audit_p99_us: robust_metric(
                runs.iter()
                    .map(|run| run.audit_timing.p99_us as f64)
                    .collect(),
            ),
            all_runs_passed: runs.iter().all(p63_run_passed),
            run_count: runs.len(),
        }
    }
}

impl P63MachineMetadata {
    fn collect() -> Self {
        Self {
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
            cpu_count: thread::available_parallelism()
                .ok()
                .map(|count| count.get()),
            rustc_version: command_first_line("rustc", &["--version"])
                .unwrap_or_else(|| "unknown".to_string()),
            cargo_version: command_first_line("cargo", &["--version"])
                .unwrap_or_else(|| "unknown".to_string()),
            git_commit: command_first_line("git", &["rev-parse", "HEAD"])
                .unwrap_or_else(|| "unknown".to_string()),
            git_dirty: git_dirty(),
            timestamp_utc: timestamp_utc(),
            profile: if cfg!(debug_assertions) {
                "debug".to_string()
            } else {
                "release".to_string()
            },
        }
    }
}

pub fn write_p63_campaign_exports(
    report: &P63CampaignReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir)
        .map_err(|err| io_diagnostic(format!("create export dir {:?}: {}", export_dir, err)))?;
    write_string(
        &export_dir.join("campaign_report.json"),
        &p63_campaign_report_to_json(report),
    )?;
    write_string(&export_dir.join("runs.jsonl"), &p63_runs_jsonl(report))?;
    write_string(&export_dir.join("runs.csv"), &p63_runs_csv(report))?;
    write_string(
        &export_dir.join("summary.md"),
        &p63_summary_markdown(report),
    )?;
    Ok(())
}

pub fn p63_campaign_report_to_json(report: &P63CampaignReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("campaign_id", &report.campaign_id, true, 2));
    out.push_str(&json_string(
        "measurement_id",
        &report.measurement_id,
        true,
        2,
    ));
    out.push_str(&json_string("astra_step", &report.astra_step, true, 2));
    out.push_str(&json_string("mode", &report.mode, true, 2));
    out.push_str(&json_string("program_path", &report.program_path, true, 2));
    out.push_str(&json_string("cost_model", &report.cost_model, true, 2));
    out.push_str(&json_string(
        "measurement_kind",
        &report.measurement_kind,
        true,
        2,
    ));
    out.push_str(&json_string(
        "threshold_profile",
        &report.threshold_profile,
        true,
        2,
    ));
    out.push_str(&json_usize("repeat_count", report.repeat_count, true, 2));
    out.push_str(&json_usize(
        "operation_count",
        report.operation_count,
        true,
        2,
    ));
    machine_metadata_json(&mut out, &report.machine_metadata);
    campaign_summary_json(&mut out, &report.summary);
    runs_json(&mut out, &report.runs);
    out.push_str(&json_string("decision", report.decision.as_str(), true, 2));
    string_array_json(
        &mut out,
        "decision_reasons",
        &report.decision_reasons,
        true,
        2,
    );
    string_array_json(&mut out, "warnings", &report.warnings, false, 2);
    out.push_str("}\n");
    out
}

fn p63_runs_jsonl(report: &P63CampaignReport) -> String {
    let mut out = String::new();
    for run in &report.runs {
        out.push_str(&run_jsonl(run));
        out.push('\n');
    }
    out
}

fn p63_runs_csv(report: &P63CampaignReport) -> String {
    let mut out = String::new();
    out.push_str("run_index,run_id,total_persisted_bytes,ratio_effective_per_byte,operation_count,read_p99_us,update_p99_us,snapshot_p99_us,rebuild_p99_us,audit_p99_us,all_gates_passed\n");
    for run in &report.runs {
        out.push_str(&format!(
            "{},{},{},{:.6},{},{},{},{},{},{},{}\n",
            run.run_index,
            run.run_id,
            run.total_persisted_bytes,
            run.ratio_effective_per_byte,
            report.operation_count,
            run.read_timing.p99_us,
            run.update_timing.p99_us,
            run.snapshot_timing.p99_us,
            run.rebuild_timing.p99_us,
            run.audit_timing.p99_us,
            p63_run_passed(run)
        ));
    }
    out
}

fn p63_summary_markdown(report: &P63CampaignReport) -> String {
    format!(
        "# ASTRA-P63 Campaign Summary\n\n\
         - Command: `cargo run -p atlas-cli -- ratio-real {} --mode {} --format json --runs {} --threshold-profile {}`\n\
         - Mode: `{}`\n\
         - Runs: `{}`\n\
         - Decision: `{}`\n\
         - Median ratio_effective_per_byte: `{:.6}`\n\
         - Median total_persisted_bytes: `{:.0}`\n\
         - Cost model: `{}`\n\
         - Measurement kind: `{}`\n\n\
         ## Limits\n\n\
         - Timings are machine-dependent and not goldenized.\n\
         - Threshold profile `p63` is conservative and not scientifically calibrated yet.\n\
         - No external dataset is included yet.\n",
        report.program_path,
        report.mode,
        report.repeat_count,
        report.threshold_profile,
        report.mode,
        report.repeat_count,
        report.decision.as_str(),
        report.summary.ratio_effective_per_byte.median,
        report.summary.total_persisted_bytes.median,
        report.cost_model,
        report.measurement_kind
    )
}

fn p63_decision(summary: &P63CampaignSummary, measured: &P62RealRatioReport) -> P63Decision {
    if measured.runs.is_empty() || !summary.all_runs_passed {
        return P63Decision::NoGoMeasuredRatioStability;
    }
    if measured.workloads.is_empty() {
        return P63Decision::RecalibrateWorkloads;
    }
    P63Decision::RecalibrateThresholds
}

fn p63_decision_reasons(decision: P63Decision, summary: &P63CampaignSummary) -> Vec<String> {
    let mut reasons = vec![
        "campaign exports available".to_string(),
        "robust summary available".to_string(),
        "measured_real_v1 still needs threshold calibration".to_string(),
        "no external dataset yet".to_string(),
        "timing values are machine-dependent".to_string(),
        format!("measured run count: {}", summary.run_count),
    ];
    if summary.all_runs_passed {
        reasons.push("all measured runs passed inherited P62 safety gates".to_string());
    }
    if decision == P63Decision::RecalibrateThresholds {
        reasons.push(
            "decision remains recalibrate because P63 thresholds are not scientifically calibrated"
                .to_string(),
        );
    }
    reasons
}

fn p63_run_passed(run: &P62MeasuredRun) -> bool {
    run.total_persisted_bytes > 0
        && run.ratio_effective_per_byte > 0.0
        && run.read_timing.p99_us > 0
        && run.update_timing.p99_us > 0
        && run.snapshot_timing.p99_us > 0
        && run.rebuild_timing.p99_us > 0
        && run.audit_timing.p99_us > 0
        && run.guard_refused
        && run.dangerous_or_adversarial_refused
        && run.audit_passed
        && run.rebuild_passed
        && run.snapshot_roundtrip_passed
}

fn robust_metric(mut samples: Vec<f64>) -> P63RobustMetric {
    samples.retain(|sample| sample.is_finite());
    samples.sort_by(|a, b| a.partial_cmp(b).expect("finite samples sort"));
    if samples.is_empty() {
        return P63RobustMetric {
            min: 0.0,
            median: 0.0,
            max: 0.0,
            mean: 0.0,
            stddev: 0.0,
            coefficient_of_variation: 0.0,
        };
    }
    let min = samples[0];
    let max = samples[samples.len() - 1];
    let median = samples[(samples.len() - 1) / 2];
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    let variance = samples
        .iter()
        .map(|sample| {
            let delta = sample - mean;
            delta * delta
        })
        .sum::<f64>()
        / samples.len() as f64;
    let stddev = variance.sqrt();
    let coefficient_of_variation = if mean == 0.0 { 0.0 } else { stddev / mean };
    P63RobustMetric {
        min,
        median,
        max,
        mean,
        stddev,
        coefficient_of_variation,
    }
}

fn command_first_line(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().next().map(str::trim).and_then(|line| {
        if line.is_empty() {
            None
        } else {
            Some(line.to_string())
        }
    })
}

fn git_dirty() -> Option<bool> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

fn timestamp_utc() -> String {
    command_first_line("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]).unwrap_or_else(|| {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        format!("unix_seconds:{}", seconds)
    })
}

fn machine_metadata_json(out: &mut String, metadata: &P63MachineMetadata) {
    out.push_str("  \"machine_metadata\": {\n");
    out.push_str(&json_string("os", &metadata.os, true, 4));
    out.push_str(&json_string("arch", &metadata.arch, true, 4));
    match metadata.cpu_count {
        Some(count) => out.push_str(&json_usize("cpu_count", count, true, 4)),
        None => out.push_str("    \"cpu_count\": null,\n"),
    }
    out.push_str(&json_string(
        "rustc_version",
        &metadata.rustc_version,
        true,
        4,
    ));
    out.push_str(&json_string(
        "cargo_version",
        &metadata.cargo_version,
        true,
        4,
    ));
    out.push_str(&json_string("git_commit", &metadata.git_commit, true, 4));
    match metadata.git_dirty {
        Some(dirty) => out.push_str(&json_bool("git_dirty", dirty, true, 4)),
        None => out.push_str("    \"git_dirty\": null,\n"),
    }
    out.push_str(&json_string(
        "timestamp_utc",
        &metadata.timestamp_utc,
        true,
        4,
    ));
    out.push_str(&json_string("profile", &metadata.profile, false, 4));
    out.push_str("  },\n");
}

fn campaign_summary_json(out: &mut String, summary: &P63CampaignSummary) {
    out.push_str("  \"summary\": {\n");
    robust_metric_json(
        out,
        "ratio_effective_per_byte",
        &summary.ratio_effective_per_byte,
        true,
        4,
    );
    robust_metric_json(
        out,
        "total_persisted_bytes",
        &summary.total_persisted_bytes,
        true,
        4,
    );
    robust_metric_json(out, "operation_count", &summary.operation_count, true, 4);
    robust_metric_json(out, "read_p99_us", &summary.read_p99_us, true, 4);
    robust_metric_json(out, "update_p99_us", &summary.update_p99_us, true, 4);
    robust_metric_json(out, "snapshot_p99_us", &summary.snapshot_p99_us, true, 4);
    robust_metric_json(out, "rebuild_p99_us", &summary.rebuild_p99_us, true, 4);
    robust_metric_json(out, "audit_p99_us", &summary.audit_p99_us, true, 4);
    out.push_str(&json_bool(
        "all_runs_passed",
        summary.all_runs_passed,
        true,
        4,
    ));
    out.push_str(&json_usize("run_count", summary.run_count, false, 4));
    out.push_str("  },\n");
}

fn robust_metric_json(
    out: &mut String,
    name: &str,
    metric: &P63RobustMetric,
    trailing_comma: bool,
    indent: usize,
) {
    let spaces = " ".repeat(indent);
    out.push_str(&format!("{}\"{}\": {{\n", spaces, name));
    out.push_str(&json_f64("min", metric.min, true, indent + 2));
    out.push_str(&json_f64("median", metric.median, true, indent + 2));
    out.push_str(&json_f64("max", metric.max, true, indent + 2));
    out.push_str(&json_f64("mean", metric.mean, true, indent + 2));
    out.push_str(&json_f64("stddev", metric.stddev, true, indent + 2));
    out.push_str(&json_f64(
        "coefficient_of_variation",
        metric.coefficient_of_variation,
        false,
        indent + 2,
    ));
    out.push_str(&format!(
        "{}}}{}\n",
        spaces,
        if trailing_comma { "," } else { "" }
    ));
}

fn runs_json(out: &mut String, runs: &[P62MeasuredRun]) {
    out.push_str("  \"runs\": [\n");
    for (idx, run) in runs.iter().enumerate() {
        out.push_str(&indent_json(&run_json(run), 4));
        out.push_str(&format!("{}\n", comma(idx, runs.len())));
    }
    out.push_str("  ],\n");
}

fn run_json(run: &P62MeasuredRun) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_usize("run_index", run.run_index, true, 2));
    out.push_str(&json_string("run_id", &run.run_id, true, 2));
    out.push_str(&json_u64(
        "total_persisted_bytes",
        run.total_persisted_bytes,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_byte",
        run.ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_usize("create_count", run.create_count, true, 2));
    out.push_str(&json_usize("read_count", run.read_count, true, 2));
    out.push_str(&json_usize("update_count", run.update_count, true, 2));
    out.push_str(&json_usize("delete_count", run.delete_count, true, 2));
    out.push_str(&json_usize("snapshot_count", run.snapshot_count, true, 2));
    out.push_str(&json_usize("rebuild_count", run.rebuild_count, true, 2));
    out.push_str(&json_usize("audit_count", run.audit_count, true, 2));
    out.push_str(&json_u64("read_p99_us", run.read_timing.p99_us, true, 2));
    out.push_str(&json_u64(
        "update_p99_us",
        run.update_timing.p99_us,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "snapshot_p99_us",
        run.snapshot_timing.p99_us,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "rebuild_p99_us",
        run.rebuild_timing.p99_us,
        true,
        2,
    ));
    out.push_str(&json_u64("audit_p99_us", run.audit_timing.p99_us, true, 2));
    out.push_str(&json_bool("guard_refused", run.guard_refused, true, 2));
    out.push_str(&json_bool(
        "dangerous_or_adversarial_refused",
        run.dangerous_or_adversarial_refused,
        true,
        2,
    ));
    out.push_str(&json_bool("audit_passed", run.audit_passed, true, 2));
    out.push_str(&json_bool("rebuild_passed", run.rebuild_passed, true, 2));
    out.push_str(&json_bool(
        "snapshot_roundtrip_passed",
        run.snapshot_roundtrip_passed,
        false,
        2,
    ));
    out.push('}');
    out
}

fn run_jsonl(run: &P62MeasuredRun) -> String {
    format!(
        "{{\"run_index\":{},\"run_id\":\"{}\",\"total_persisted_bytes\":{},\"ratio_effective_per_byte\":{:.6},\"create_count\":{},\"read_count\":{},\"update_count\":{},\"delete_count\":{},\"snapshot_count\":{},\"rebuild_count\":{},\"audit_count\":{},\"read_p99_us\":{},\"update_p99_us\":{},\"snapshot_p99_us\":{},\"rebuild_p99_us\":{},\"audit_p99_us\":{},\"guard_refused\":{},\"dangerous_or_adversarial_refused\":{},\"audit_passed\":{},\"rebuild_passed\":{},\"snapshot_roundtrip_passed\":{}}}",
        run.run_index,
        escape_json(&run.run_id),
        run.total_persisted_bytes,
        run.ratio_effective_per_byte,
        run.create_count,
        run.read_count,
        run.update_count,
        run.delete_count,
        run.snapshot_count,
        run.rebuild_count,
        run.audit_count,
        run.read_timing.p99_us,
        run.update_timing.p99_us,
        run.snapshot_timing.p99_us,
        run.rebuild_timing.p99_us,
        run.audit_timing.p99_us,
        run.guard_refused,
        run.dangerous_or_adversarial_refused,
        run.audit_passed,
        run.rebuild_passed,
        run.snapshot_roundtrip_passed
    )
}

fn string_array_json(
    out: &mut String,
    name: &str,
    values: &[String],
    trailing_comma: bool,
    indent: usize,
) {
    let spaces = " ".repeat(indent);
    out.push_str(&format!("{}\"{}\": [\n", spaces, name));
    for (idx, value) in values.iter().enumerate() {
        out.push_str(&format!(
            "{}  \"{}\"{}\n",
            spaces,
            escape_json(value),
            comma(idx, values.len())
        ));
    }
    out.push_str(&format!(
        "{}]{}\n",
        spaces,
        if trailing_comma { "," } else { "" }
    ));
}

fn indent_json(text: &str, indent: usize) -> String {
    let spaces = " ".repeat(indent);
    text.lines()
        .map(|line| format!("{}{}", spaces, line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn json_string(name: &str, value: &str, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": \"{}\"{}\n",
        " ".repeat(indent),
        name,
        escape_json(value),
        if trailing_comma { "," } else { "" }
    )
}

fn json_bool(name: &str, value: bool, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        name,
        value,
        if trailing_comma { "," } else { "" }
    )
}

fn json_usize(name: &str, value: usize, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        name,
        value,
        if trailing_comma { "," } else { "" }
    )
}

fn json_u64(name: &str, value: u64, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {}{}\n",
        " ".repeat(indent),
        name,
        value,
        if trailing_comma { "," } else { "" }
    )
}

fn json_f64(name: &str, value: f64, trailing_comma: bool, indent: usize) -> String {
    format!(
        "{}\"{}\": {:.6}{}\n",
        " ".repeat(indent),
        name,
        value,
        if trailing_comma { "," } else { "" }
    )
}

fn comma(idx: usize, len: usize) -> &'static str {
    if idx + 1 == len {
        ""
    } else {
        ","
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn write_string(path: &Path, content: &str) -> AtlasResult<()> {
    fs::write(path, content).map_err(|err| io_diagnostic(format!("write {:?}: {}", path, err)))
}

fn io_diagnostic(message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Io, message)
}
