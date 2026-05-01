use crate::{
    p62_real_ratio_report_file_with_runs, AtlasResult, Diagnostic, DiagnosticCode, P62MeasuredRun,
    P62RealRatioReport, WorkloadMode,
};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const ASSUMED_MATERIALIZED_VALUE_BYTES: u128 = 8;

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
            "p63" | "p63_conservative_v1" => Some(P63ThresholdProfile::P63),
            _ => None,
        }
    }

    pub fn spec(self) -> P63ThresholdProfileSpec {
        match self {
            P63ThresholdProfile::P63 => P63ThresholdProfileSpec {
                profile_id: "p63_conservative_v1",
                alias: Some("p63"),
                min_runs_required: 10,
                max_ratio_cv: 0.05,
                max_bytes_cv: 0.05,
                max_timing_cv: Some(0.50),
                min_median_ratio_effective_per_byte: None,
                require_machine_metadata: true,
                require_campaign_exports: true,
                require_realish_workloads: false,
                allow_validate: false,
                candidate_min_runs_for_future_validation: 30,
                candidate_min_campaigns_for_future_validation: 3,
                candidate_max_ratio_cv: 0.03,
                candidate_max_bytes_cv: 0.03,
                candidate_max_intra_mode_ratio_shift_percent: 5.0,
                candidate_max_intra_mode_bytes_shift_percent: 5.0,
                candidate_requires_multi_machine: true,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct P63ThresholdProfileSpec {
    pub profile_id: &'static str,
    pub alias: Option<&'static str>,
    pub min_runs_required: usize,
    pub max_ratio_cv: f64,
    pub max_bytes_cv: f64,
    pub max_timing_cv: Option<f64>,
    pub min_median_ratio_effective_per_byte: Option<f64>,
    pub require_machine_metadata: bool,
    pub require_campaign_exports: bool,
    pub require_realish_workloads: bool,
    pub allow_validate: bool,
    pub candidate_min_runs_for_future_validation: usize,
    pub candidate_min_campaigns_for_future_validation: usize,
    pub candidate_max_ratio_cv: f64,
    pub candidate_max_bytes_cv: f64,
    pub candidate_max_intra_mode_ratio_shift_percent: f64,
    pub candidate_max_intra_mode_bytes_shift_percent: f64,
    pub candidate_requires_multi_machine: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P63StabilityStatus {
    Stable,
    Warn,
    Unstable,
    NotEnoughRuns,
    NotAvailable,
}

impl P63StabilityStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            P63StabilityStatus::Stable => "STABLE",
            P63StabilityStatus::Warn => "WARN",
            P63StabilityStatus::Unstable => "UNSTABLE",
            P63StabilityStatus::NotEnoughRuns => "NOT_ENOUGH_RUNS",
            P63StabilityStatus::NotAvailable => "NOT_AVAILABLE",
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
    pub threshold_profile_resolved: String,
    pub threshold_profile_config: P63ThresholdProfileSpec,
    pub repeat_count: usize,
    pub operation_count: usize,
    pub machine_metadata: P63MachineMetadata,
    pub summary: P63CampaignSummary,
    pub core_metrics: P63CoreRatioMetrics,
    pub ratio_stability_status: P63StabilityStatus,
    pub bytes_stability_status: P63StabilityStatus,
    pub timing_stability_status: P63StabilityStatus,
    pub campaign_stability_status: P63StabilityStatus,
    pub stability_reasons: Vec<String>,
    pub runs: Vec<P62MeasuredRun>,
    pub decision: P63Decision,
    pub decision_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P63CoreRatioMetrics {
    pub virtual_declared_units: u128,
    pub virtual_reachable_units: u128,
    pub virtual_readable_units: u128,
    pub virtual_updatable_units: u128,
    pub virtual_safe_units: u128,
    pub virtual_effective_units: u128,
    pub total_persisted_bytes: u64,
    pub payload_file_bytes: u64,
    pub index_file_bytes: u64,
    pub journal_file_bytes: u64,
    pub manifest_file_bytes: u64,
    pub checksum_or_audit_bytes: Option<u64>,
    pub metadata_bytes: Option<u64>,
    pub ratio_declared_per_byte: f64,
    pub ratio_reachable_per_byte: f64,
    pub ratio_readable_per_byte: f64,
    pub ratio_updatable_per_byte: f64,
    pub ratio_safe_per_byte: f64,
    pub ratio_effective_per_byte: f64,
    pub assumed_materialized_value_bytes: u128,
    pub estimated_materialized_bytes: u128,
    pub gain_vs_materialized: f64,
    pub effective_gain_vs_materialized: f64,
}

#[derive(Debug, Clone)]
struct P63StabilityEvaluation {
    ratio: P63StabilityStatus,
    bytes: P63StabilityStatus,
    timing: P63StabilityStatus,
    campaign: P63StabilityStatus,
    reasons: Vec<String>,
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
        let threshold_profile_config = threshold_profile.spec();
        let summary = P63CampaignSummary::from_runs(&measured.runs, measured.operation_count);
        let core_metrics = P63CoreRatioMetrics::from_p62_report(&measured);
        let stability = p63_stability(&summary, &threshold_profile_config);
        let decision = p63_decision(&summary, &measured, &threshold_profile_config, &stability);
        let decision_reasons =
            p63_decision_reasons(decision, &summary, &threshold_profile_config, &stability);
        Self {
            campaign_id: format!("p63-{}", measured.measurement_id),
            measurement_id: measured.measurement_id,
            astra_step: "P63".to_string(),
            mode: measured.mode,
            program_path: measured.program_path,
            cost_model: measured.cost_model,
            measurement_kind: measured.measurement_kind,
            threshold_profile: threshold_profile.as_str().to_string(),
            threshold_profile_resolved: threshold_profile_config.profile_id.to_string(),
            threshold_profile_config,
            repeat_count: measured.repeat_count,
            operation_count: measured.operation_count,
            machine_metadata: P63MachineMetadata::collect(),
            summary,
            core_metrics,
            ratio_stability_status: stability.ratio,
            bytes_stability_status: stability.bytes,
            timing_stability_status: stability.timing,
            campaign_stability_status: stability.campaign,
            stability_reasons: stability.reasons,
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

impl P63CoreRatioMetrics {
    fn from_p62_report(report: &P62RealRatioReport) -> Self {
        let total_persisted_bytes = report.persisted_bytes.total();
        let estimated_materialized_bytes = report
            .virtual_declared
            .saturating_mul(ASSUMED_MATERIALIZED_VALUE_BYTES);
        let effective_materialized_bytes = report
            .virtual_effective
            .saturating_mul(ASSUMED_MATERIALIZED_VALUE_BYTES);

        Self {
            virtual_declared_units: report.virtual_declared,
            virtual_reachable_units: report.virtual_reachable,
            virtual_readable_units: report.virtual_readable,
            virtual_updatable_units: report.virtual_updatable,
            virtual_safe_units: report.virtual_safe,
            virtual_effective_units: report.virtual_effective,
            total_persisted_bytes,
            payload_file_bytes: report.persisted_bytes.payload_file_bytes,
            index_file_bytes: report.persisted_bytes.index_file_bytes,
            journal_file_bytes: report.persisted_bytes.journal_file_bytes,
            manifest_file_bytes: report.persisted_bytes.manifest_file_bytes,
            checksum_or_audit_bytes: Some(report.persisted_bytes.audit_file_bytes),
            metadata_bytes: None,
            ratio_declared_per_byte: ratio_u128(report.virtual_declared, total_persisted_bytes),
            ratio_reachable_per_byte: ratio_u128(report.virtual_reachable, total_persisted_bytes),
            ratio_readable_per_byte: ratio_u128(report.virtual_readable, total_persisted_bytes),
            ratio_updatable_per_byte: ratio_u128(report.virtual_updatable, total_persisted_bytes),
            ratio_safe_per_byte: ratio_u128(report.virtual_safe, total_persisted_bytes),
            ratio_effective_per_byte: ratio_u128(report.virtual_effective, total_persisted_bytes),
            assumed_materialized_value_bytes: ASSUMED_MATERIALIZED_VALUE_BYTES,
            estimated_materialized_bytes,
            gain_vs_materialized: ratio_u128(estimated_materialized_bytes, total_persisted_bytes),
            effective_gain_vs_materialized: ratio_u128(
                effective_materialized_bytes,
                total_persisted_bytes,
            ),
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

pub fn p63_campaign_compare_json_files(path_a: &str, path_b: &str) -> AtlasResult<String> {
    let report_a = fs::read_to_string(path_a)
        .map_err(|err| io_diagnostic(format!("read campaign A {:?}: {}", path_a, err)))?;
    let report_b = fs::read_to_string(path_b)
        .map_err(|err| io_diagnostic(format!("read campaign B {:?}: {}", path_b, err)))?;
    Ok(p63_campaign_compare_json(
        path_a, &report_a, path_b, &report_b,
    ))
}

pub fn p63_campaign_compare_json(
    path_a: &str,
    report_a: &str,
    path_b: &str,
    report_b: &str,
) -> String {
    let parsed_a = ParsedCampaign::from_json(path_a, report_a);
    let parsed_b = ParsedCampaign::from_json(path_b, report_b);
    let compatibility_status = compatibility_status(&parsed_a, &parsed_b);
    let ratio_shift = parsed_b.median_ratio - parsed_a.median_ratio;
    let ratio_shift_percent = percent_shift(parsed_a.median_ratio, parsed_b.median_ratio);
    let bytes_shift = parsed_b.median_bytes - parsed_a.median_bytes;
    let bytes_shift_percent = percent_shift(parsed_a.median_bytes, parsed_b.median_bytes);
    let stability_delta = format!("{} -> {}", parsed_a.stability, parsed_b.stability);
    let decision_compatibility = if parsed_a.decision == parsed_b.decision {
        "SAME_DECISION"
    } else {
        "DIFFERENT_DECISION"
    };
    let intra_mode_status = intra_mode_status(
        compatibility_status,
        ratio_shift_percent,
        bytes_shift_percent,
        parsed_a.decision == parsed_b.decision,
    );
    let interpretation = comparison_interpretation(compatibility_status);
    let comparison_decision = comparison_decision(compatibility_status);

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("campaign_a", &parsed_a.campaign_id, true, 2));
    out.push_str(&json_string("campaign_b", &parsed_b.campaign_id, true, 2));
    out.push_str(&json_string("mode_a", &parsed_a.mode, true, 2));
    out.push_str(&json_string("mode_b", &parsed_b.mode, true, 2));
    out.push_str(&json_string(
        "threshold_profile_a",
        &parsed_a.threshold_profile,
        true,
        2,
    ));
    out.push_str(&json_string(
        "threshold_profile_b",
        &parsed_b.threshold_profile,
        true,
        2,
    ));
    out.push_str(&json_f64("median_ratio_a", parsed_a.median_ratio, true, 2));
    out.push_str(&json_f64("median_ratio_b", parsed_b.median_ratio, true, 2));
    out.push_str(&json_f64("ratio_shift", ratio_shift, true, 2));
    out.push_str(&json_f64(
        "ratio_shift_percent",
        ratio_shift_percent,
        true,
        2,
    ));
    out.push_str(&json_f64("median_bytes_a", parsed_a.median_bytes, true, 2));
    out.push_str(&json_f64("median_bytes_b", parsed_b.median_bytes, true, 2));
    out.push_str(&json_f64("bytes_shift", bytes_shift, true, 2));
    out.push_str(&json_f64(
        "bytes_shift_percent",
        bytes_shift_percent,
        true,
        2,
    ));
    out.push_str(&json_string("decision_a", &parsed_a.decision, true, 2));
    out.push_str(&json_string("decision_b", &parsed_b.decision, true, 2));
    out.push_str(&json_string("stability_a", &parsed_a.stability, true, 2));
    out.push_str(&json_string("stability_b", &parsed_b.stability, true, 2));
    out.push_str(&json_string("stability_delta", &stability_delta, true, 2));
    out.push_str(&json_string(
        "decision_compatibility",
        decision_compatibility,
        true,
        2,
    ));
    out.push_str(&json_string(
        "intra_mode_status",
        intra_mode_status,
        true,
        2,
    ));
    out.push_str(&json_string(
        "compatibility_status",
        compatibility_status.as_str(),
        true,
        2,
    ));
    out.push_str(&json_string("interpretation", interpretation, true, 2));
    out.push_str(&json_string(
        "comparison_decision",
        comparison_decision,
        false,
        2,
    ));
    out.push_str("}\n");
    out
}

pub fn p63_campaign_register_json_file(
    report_path: &str,
    registry_path: &str,
    campaign_name: &str,
) -> AtlasResult<String> {
    let report = fs::read_to_string(report_path)
        .map_err(|err| io_diagnostic(format!("read campaign report {:?}: {}", report_path, err)))?;
    let entry = P63CampaignRegistryEntry::from_report(report_path, campaign_name, &report);
    if !entry.valid {
        return Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            "ratio-campaign-register requires a valid P63 campaign_report.json",
        ));
    }

    let mut campaigns = if Path::new(registry_path).exists() {
        let registry = fs::read_to_string(registry_path)
            .map_err(|err| io_diagnostic(format!("read registry {:?}: {}", registry_path, err)))?;
        parse_registry_entries(&registry)
    } else {
        Vec::new()
    };
    campaigns.retain(|existing| {
        existing.campaign_id != entry.campaign_id && existing.campaign_name != entry.campaign_name
    });
    campaigns.push(entry);
    campaigns.sort_by(|a, b| a.campaign_name.cmp(&b.campaign_name));

    let registry_json = p63_campaign_registry_to_json(&campaigns);
    if let Some(parent) = Path::new(registry_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                io_diagnostic(format!("create registry dir {:?}: {}", parent, err))
            })?;
        }
    }
    write_string(Path::new(registry_path), &registry_json)?;
    Ok(registry_json)
}

pub fn p63_campaign_summary_json_file(registry_path: &str) -> AtlasResult<String> {
    let registry = fs::read_to_string(registry_path)
        .map_err(|err| io_diagnostic(format!("read registry {:?}: {}", registry_path, err)))?;
    let campaigns = parse_registry_entries(&registry);
    Ok(p63_campaign_registry_summary_json(&campaigns))
}

pub fn p63_campaign_set_summary_json_file(
    registry_path: &str,
    set_name: &str,
    mode: Option<WorkloadMode>,
    threshold_profile: Option<P63ThresholdProfile>,
) -> AtlasResult<String> {
    let registry = fs::read_to_string(registry_path)
        .map_err(|err| io_diagnostic(format!("read registry {:?}: {}", registry_path, err)))?;
    let mut campaigns = parse_registry_entries(&registry);
    if let Some(mode) = mode {
        campaigns.retain(|campaign| campaign.mode == mode.as_str());
    }
    if let Some(threshold_profile) = threshold_profile {
        let spec = threshold_profile.spec();
        campaigns.retain(|campaign| {
            campaign.threshold_profile == spec.profile_id
                || campaign.threshold_profile == threshold_profile.as_str()
        });
        if campaigns
            .iter()
            .any(|campaign| campaign.repeat_count >= spec.candidate_min_runs_for_future_validation)
        {
            campaigns.retain(|campaign| {
                campaign.repeat_count >= spec.candidate_min_runs_for_future_validation
            });
        }
    }
    Ok(p63_campaign_set_summary_json(set_name, &campaigns))
}

#[derive(Debug, Clone, PartialEq)]
pub struct P63CampaignRegistryEntry {
    pub valid: bool,
    pub campaign_id: String,
    pub campaign_name: String,
    pub mode: String,
    pub threshold_profile: String,
    pub report_path: String,
    pub repeat_count: usize,
    pub median_ratio_effective_per_byte: f64,
    pub median_total_persisted_bytes: f64,
    pub virtual_declared_units: u128,
    pub virtual_effective_units: u128,
    pub total_persisted_bytes: u64,
    pub ratio_effective_per_byte: f64,
    pub gain_vs_materialized: f64,
    pub effective_gain_vs_materialized: f64,
    pub campaign_stability_status: String,
    pub decision: String,
    pub timestamp_utc: String,
    pub machine_os: String,
    pub machine_arch: String,
    pub machine_cpu_count: Option<usize>,
    pub git_commit: String,
}

impl P63CampaignRegistryEntry {
    fn from_report(report_path: &str, campaign_name: &str, json: &str) -> Self {
        let threshold_profile = extract_json_string(json, "threshold_profile_resolved")
            .or_else(|| extract_json_string(json, "threshold_profile"))
            .unwrap_or_default();
        let core_metrics = extract_object_block(json, "core_ratio_metrics").unwrap_or("");
        let entry = Self {
            valid: true,
            campaign_id: extract_json_string(json, "campaign_id").unwrap_or_default(),
            campaign_name: campaign_name.to_string(),
            mode: extract_json_string(json, "mode").unwrap_or_default(),
            threshold_profile,
            report_path: report_path.to_string(),
            repeat_count: extract_json_number(json, "repeat_count").unwrap_or(0.0) as usize,
            median_ratio_effective_per_byte: extract_metric_median(
                json,
                "ratio_effective_per_byte",
            )
            .unwrap_or(0.0),
            median_total_persisted_bytes: extract_metric_median(json, "total_persisted_bytes")
                .unwrap_or(0.0),
            virtual_declared_units: extract_json_number(core_metrics, "virtual_declared_units")
                .unwrap_or(0.0) as u128,
            virtual_effective_units: extract_json_number(core_metrics, "virtual_effective_units")
                .unwrap_or(0.0) as u128,
            total_persisted_bytes: extract_json_number(core_metrics, "total_persisted_bytes")
                .unwrap_or(0.0) as u64,
            ratio_effective_per_byte: extract_json_number(core_metrics, "ratio_effective_per_byte")
                .unwrap_or(0.0),
            gain_vs_materialized: extract_json_number(core_metrics, "gain_vs_materialized")
                .unwrap_or(0.0),
            effective_gain_vs_materialized: extract_json_number(
                core_metrics,
                "effective_gain_vs_materialized",
            )
            .unwrap_or(0.0),
            campaign_stability_status: extract_json_string(json, "campaign_stability_status")
                .unwrap_or_default(),
            decision: extract_json_string(json, "decision").unwrap_or_default(),
            timestamp_utc: extract_json_string(json, "timestamp_utc").unwrap_or_default(),
            machine_os: extract_json_string(json, "os").unwrap_or_else(|| "unknown".to_string()),
            machine_arch: extract_json_string(json, "arch")
                .unwrap_or_else(|| "unknown".to_string()),
            machine_cpu_count: extract_json_number(json, "cpu_count").map(|value| value as usize),
            git_commit: extract_json_string(json, "git_commit")
                .unwrap_or_else(|| "unknown".to_string()),
        };
        if entry.campaign_id.is_empty()
            || entry.mode.is_empty()
            || entry.threshold_profile.is_empty()
            || entry.decision.is_empty()
            || entry.campaign_stability_status.is_empty()
        {
            return Self {
                valid: false,
                ..entry
            };
        }
        entry
    }

    fn from_registry_json(json: &str) -> Option<Self> {
        let entry = Self {
            valid: true,
            campaign_id: extract_json_string(json, "campaign_id")?,
            campaign_name: extract_json_string(json, "campaign_name")?,
            mode: extract_json_string(json, "mode")?,
            threshold_profile: extract_json_string(json, "threshold_profile")?,
            report_path: extract_json_string(json, "report_path")?,
            repeat_count: extract_json_number(json, "repeat_count")? as usize,
            median_ratio_effective_per_byte: extract_json_number(
                json,
                "median_ratio_effective_per_byte",
            )?,
            median_total_persisted_bytes: extract_json_number(
                json,
                "median_total_persisted_bytes",
            )?,
            virtual_declared_units: extract_json_number(json, "virtual_declared_units")
                .unwrap_or(0.0) as u128,
            virtual_effective_units: extract_json_number(json, "virtual_effective_units")
                .unwrap_or(0.0) as u128,
            total_persisted_bytes: extract_json_number(json, "total_persisted_bytes").unwrap_or(0.0)
                as u64,
            ratio_effective_per_byte: extract_json_number(json, "ratio_effective_per_byte")
                .unwrap_or(0.0),
            gain_vs_materialized: extract_json_number(json, "gain_vs_materialized").unwrap_or(0.0),
            effective_gain_vs_materialized: extract_json_number(
                json,
                "effective_gain_vs_materialized",
            )
            .unwrap_or(0.0),
            campaign_stability_status: extract_json_string(json, "campaign_stability_status")?,
            decision: extract_json_string(json, "decision")?,
            timestamp_utc: extract_json_string(json, "timestamp_utc")?,
            machine_os: extract_json_string(json, "os").unwrap_or_else(|| "unknown".to_string()),
            machine_arch: extract_json_string(json, "arch")
                .unwrap_or_else(|| "unknown".to_string()),
            machine_cpu_count: extract_json_number(json, "cpu_count").map(|value| value as usize),
            git_commit: extract_json_string(json, "git_commit")
                .unwrap_or_else(|| "unknown".to_string()),
        };
        Some(entry)
    }
}

fn p63_campaign_registry_to_json(campaigns: &[P63CampaignRegistryEntry]) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("registry_version", "p63_registry_v1", true, 2));
    out.push_str(&json_string("astra_step", "P63", true, 2));
    out.push_str("  \"campaigns\": [\n");
    for (idx, campaign) in campaigns.iter().enumerate() {
        out.push_str(&indent_json(&registry_entry_json(campaign), 4));
        out.push_str(&format!("{}\n", comma(idx, campaigns.len())));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn registry_entry_json(entry: &P63CampaignRegistryEntry) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("campaign_id", &entry.campaign_id, true, 2));
    out.push_str(&json_string("campaign_name", &entry.campaign_name, true, 2));
    out.push_str(&json_string("mode", &entry.mode, true, 2));
    out.push_str(&json_string(
        "threshold_profile",
        &entry.threshold_profile,
        true,
        2,
    ));
    out.push_str(&json_string("report_path", &entry.report_path, true, 2));
    out.push_str(&json_usize("repeat_count", entry.repeat_count, true, 2));
    out.push_str(&json_f64(
        "median_ratio_effective_per_byte",
        entry.median_ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "median_total_persisted_bytes",
        entry.median_total_persisted_bytes,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_declared_units",
        entry.virtual_declared_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_effective_units",
        entry.virtual_effective_units,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "total_persisted_bytes",
        entry.total_persisted_bytes,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_byte",
        entry.ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "gain_vs_materialized",
        entry.gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "effective_gain_vs_materialized",
        entry.effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_string(
        "campaign_stability_status",
        &entry.campaign_stability_status,
        true,
        2,
    ));
    out.push_str(&json_string("decision", &entry.decision, true, 2));
    out.push_str(&json_string("timestamp_utc", &entry.timestamp_utc, true, 2));
    out.push_str("  \"machine_metadata\": {\n");
    out.push_str(&json_string("os", &entry.machine_os, true, 4));
    out.push_str(&json_string("arch", &entry.machine_arch, true, 4));
    match entry.machine_cpu_count {
        Some(cpu_count) => out.push_str(&json_usize("cpu_count", cpu_count, false, 4)),
        None => out.push_str("    \"cpu_count\": null\n"),
    }
    out.push_str("  },\n");
    out.push_str(&json_string("git_commit", &entry.git_commit, false, 2));
    out.push_str("}");
    out
}

fn p63_campaign_registry_summary_json(campaigns: &[P63CampaignRegistryEntry]) -> String {
    let modes = unique_strings(campaigns.iter().map(|campaign| campaign.mode.as_str()));
    let profiles = unique_strings(
        campaigns
            .iter()
            .map(|campaign| campaign.threshold_profile.as_str()),
    );
    let decisions = unique_strings(campaigns.iter().map(|campaign| campaign.decision.as_str()));
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string("registry_version", "p63_registry_v1", true, 2));
    out.push_str(&json_string("astra_step", "P63", true, 2));
    out.push_str(&json_usize("campaign_count", campaigns.len(), true, 2));
    string_array_json(&mut out, "modes", &modes, true, 2);
    string_array_json(&mut out, "threshold_profiles", &profiles, true, 2);
    string_array_json(&mut out, "decisions", &decisions, true, 2);
    out.push_str("  \"campaigns\": [\n");
    for (idx, campaign) in campaigns.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&json_string(
            "campaign_name",
            &campaign.campaign_name,
            true,
            6,
        ));
        out.push_str(&json_string("mode", &campaign.mode, true, 6));
        out.push_str(&json_f64(
            "median_ratio_effective_per_byte",
            campaign.median_ratio_effective_per_byte,
            true,
            6,
        ));
        out.push_str(&json_f64(
            "median_total_persisted_bytes",
            campaign.median_total_persisted_bytes,
            true,
            6,
        ));
        out.push_str(&json_u128(
            "virtual_effective_units",
            campaign.virtual_effective_units,
            true,
            6,
        ));
        out.push_str(&json_f64(
            "gain_vs_materialized",
            campaign.gain_vs_materialized,
            true,
            6,
        ));
        out.push_str(&json_string(
            "campaign_stability_status",
            &campaign.campaign_stability_status,
            true,
            6,
        ));
        out.push_str(&json_string("decision", &campaign.decision, false, 6));
        out.push_str(&format!("    }}{}\n", comma(idx, campaigns.len())));
    }
    out.push_str("  ],\n");
    out.push_str("  \"warnings\": [\n");
    out.push_str("    \"registry summaries are local analysis artifacts and are not scientific validation\",\n");
    out.push_str("    \"artifacts/p63/ remains ignored by git\"\n");
    out.push_str("  ],\n");
    out.push_str(&json_string(
        "recommendation",
        "continue collecting comparable same-mode campaigns before calibration",
        false,
        2,
    ));
    out.push_str("}\n");
    out
}

fn p63_campaign_set_summary_json(set_name: &str, campaigns: &[P63CampaignRegistryEntry]) -> String {
    let profile = P63ThresholdProfile::P63.spec();
    let modes = unique_strings(campaigns.iter().map(|campaign| campaign.mode.as_str()));
    let profiles = unique_strings(
        campaigns
            .iter()
            .map(|campaign| campaign.threshold_profile.as_str()),
    );
    let median_ratio_values: Vec<f64> = campaigns
        .iter()
        .map(|campaign| campaign.median_ratio_effective_per_byte)
        .collect();
    let median_bytes_values: Vec<f64> = campaigns
        .iter()
        .map(|campaign| campaign.median_total_persisted_bytes)
        .collect();
    let virtual_declared_values: Vec<u128> = campaigns
        .iter()
        .map(|campaign| campaign.virtual_declared_units)
        .collect();
    let virtual_effective_values: Vec<u128> = campaigns
        .iter()
        .map(|campaign| campaign.virtual_effective_units)
        .collect();
    let total_persisted_bytes_values: Vec<u64> = campaigns
        .iter()
        .map(|campaign| campaign.total_persisted_bytes)
        .collect();
    let gain_vs_materialized_values: Vec<f64> = campaigns
        .iter()
        .map(|campaign| campaign.gain_vs_materialized)
        .collect();
    let effective_gain_vs_materialized_values: Vec<f64> = campaigns
        .iter()
        .map(|campaign| campaign.effective_gain_vs_materialized)
        .collect();
    let campaign_count = campaigns.len();
    let total_runs = campaigns.iter().map(|campaign| campaign.repeat_count).sum();
    let stable_campaign_count = campaigns
        .iter()
        .filter(|campaign| campaign.campaign_stability_status == "STABLE")
        .count();
    let warn_campaign_count = campaigns
        .iter()
        .filter(|campaign| campaign.campaign_stability_status == "WARN")
        .count();
    let unstable_campaign_count = campaigns
        .iter()
        .filter(|campaign| campaign.campaign_stability_status == "UNSTABLE")
        .count();
    let min_repeat_count = campaigns
        .iter()
        .map(|campaign| campaign.repeat_count)
        .min()
        .unwrap_or(0);
    let ratio_shift_percent_range = percent_range(&median_ratio_values);
    let bytes_shift_percent_range = percent_range(&median_bytes_values);
    let virtual_declared_units = median_u128(&virtual_declared_values);
    let virtual_effective_units = median_u128(&virtual_effective_values);
    let total_persisted_bytes = median_u64(&total_persisted_bytes_values);
    let gain_vs_materialized = median_f64(&gain_vs_materialized_values);
    let effective_gain_vs_materialized = median_f64(&effective_gain_vs_materialized_values);
    let ratio_effective_per_byte = ratio_u128(virtual_effective_units, total_persisted_bytes);
    let intra_mode_set_status = campaign_set_status(
        campaign_count,
        min_repeat_count,
        modes.len(),
        profiles.len(),
        unstable_campaign_count,
        warn_campaign_count,
        ratio_shift_percent_range,
        bytes_shift_percent_range,
        &profile,
    );
    let set_decision = campaign_set_decision(intra_mode_set_status);
    let set_reasons = campaign_set_reasons(
        intra_mode_set_status,
        campaign_count,
        total_runs,
        min_repeat_count,
        ratio_shift_percent_range,
        bytes_shift_percent_range,
        &profile,
    );

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&json_string(
        "campaign_set_version",
        "p63_campaign_set_v1",
        true,
        2,
    ));
    out.push_str(&json_string("astra_step", "P63", true, 2));
    out.push_str(&json_string("set_name", set_name, true, 2));
    out.push_str(&json_string("mode", single_or_mixed(&modes), true, 2));
    out.push_str(&json_string(
        "threshold_profile",
        single_or_mixed(&profiles),
        true,
        2,
    ));
    out.push_str(&json_usize("campaign_count", campaign_count, true, 2));
    out.push_str(&json_usize("total_runs", total_runs, true, 2));
    out.push_str("  \"campaigns\": [\n");
    for (idx, campaign) in campaigns.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&json_string(
            "campaign_name",
            &campaign.campaign_name,
            true,
            6,
        ));
        out.push_str(&json_string("mode", &campaign.mode, true, 6));
        out.push_str(&json_string(
            "threshold_profile",
            &campaign.threshold_profile,
            true,
            6,
        ));
        out.push_str(&json_usize("repeat_count", campaign.repeat_count, true, 6));
        out.push_str(&json_f64(
            "median_ratio_effective_per_byte",
            campaign.median_ratio_effective_per_byte,
            true,
            6,
        ));
        out.push_str(&json_f64(
            "median_total_persisted_bytes",
            campaign.median_total_persisted_bytes,
            true,
            6,
        ));
        out.push_str(&json_u128(
            "virtual_declared_units",
            campaign.virtual_declared_units,
            true,
            6,
        ));
        out.push_str(&json_u128(
            "virtual_effective_units",
            campaign.virtual_effective_units,
            true,
            6,
        ));
        out.push_str(&json_u64(
            "total_persisted_bytes",
            campaign.total_persisted_bytes,
            true,
            6,
        ));
        out.push_str(&json_f64(
            "gain_vs_materialized",
            campaign.gain_vs_materialized,
            true,
            6,
        ));
        out.push_str(&json_f64(
            "effective_gain_vs_materialized",
            campaign.effective_gain_vs_materialized,
            true,
            6,
        ));
        out.push_str(&json_string(
            "campaign_stability_status",
            &campaign.campaign_stability_status,
            true,
            6,
        ));
        out.push_str(&json_string("decision", &campaign.decision, false, 6));
        out.push_str(&format!("    }}{}\n", comma(idx, campaigns.len())));
    }
    out.push_str("  ],\n");
    f64_array_json(
        &mut out,
        "median_ratio_values",
        &median_ratio_values,
        true,
        2,
    );
    f64_array_json(
        &mut out,
        "median_bytes_values",
        &median_bytes_values,
        true,
        2,
    );
    u128_array_json(
        &mut out,
        "virtual_declared_values",
        &virtual_declared_values,
        true,
        2,
    );
    u128_array_json(
        &mut out,
        "virtual_effective_values",
        &virtual_effective_values,
        true,
        2,
    );
    u64_array_json(
        &mut out,
        "total_persisted_bytes_values",
        &total_persisted_bytes_values,
        true,
        2,
    );
    f64_array_json(
        &mut out,
        "gain_vs_materialized_values",
        &gain_vs_materialized_values,
        true,
        2,
    );
    f64_array_json(
        &mut out,
        "effective_gain_vs_materialized_values",
        &effective_gain_vs_materialized_values,
        true,
        2,
    );
    out.push_str(&json_u128(
        "virtual_declared_units",
        virtual_declared_units,
        true,
        2,
    ));
    out.push_str(&json_u128(
        "virtual_effective_units",
        virtual_effective_units,
        true,
        2,
    ));
    out.push_str(&json_u64(
        "total_persisted_bytes",
        total_persisted_bytes,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_byte",
        ratio_effective_per_byte,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "gain_vs_materialized",
        gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "effective_gain_vs_materialized",
        effective_gain_vs_materialized,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "ratio_shift_percent_range",
        ratio_shift_percent_range,
        true,
        2,
    ));
    out.push_str(&json_f64(
        "bytes_shift_percent_range",
        bytes_shift_percent_range,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "stable_campaign_count",
        stable_campaign_count,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "warn_campaign_count",
        warn_campaign_count,
        true,
        2,
    ));
    out.push_str(&json_usize(
        "unstable_campaign_count",
        unstable_campaign_count,
        true,
        2,
    ));
    out.push_str(&json_string(
        "intra_mode_set_status",
        intra_mode_set_status,
        true,
        2,
    ));
    out.push_str(&json_string("set_decision", set_decision, true, 2));
    string_array_json(&mut out, "set_reasons", &set_reasons, false, 2);
    out.push_str("}\n");
    out
}

fn campaign_set_status(
    campaign_count: usize,
    min_repeat_count: usize,
    mode_count: usize,
    profile_count: usize,
    unstable_campaign_count: usize,
    warn_campaign_count: usize,
    ratio_shift_percent_range: f64,
    bytes_shift_percent_range: f64,
    profile: &P63ThresholdProfileSpec,
) -> &'static str {
    if campaign_count == 0 {
        return "CAMPAIGN_SET_NOT_ENOUGH_DATA";
    }
    if mode_count > 1 {
        return "CAMPAIGN_SET_MIXED_MODES";
    }
    if profile_count > 1 {
        return "CAMPAIGN_SET_MIXED_PROFILES";
    }
    if campaign_count < profile.candidate_min_campaigns_for_future_validation {
        return "CAMPAIGN_SET_NOT_ENOUGH_DATA";
    }
    if min_repeat_count < profile.candidate_min_runs_for_future_validation {
        return "CAMPAIGN_SET_NOT_ENOUGH_DATA";
    }
    if unstable_campaign_count > 0
        || ratio_shift_percent_range > profile.candidate_max_intra_mode_ratio_shift_percent * 2.0
        || bytes_shift_percent_range > profile.candidate_max_intra_mode_bytes_shift_percent * 2.0
    {
        return "CAMPAIGN_SET_UNSTABLE";
    }
    if warn_campaign_count > 0
        || ratio_shift_percent_range > profile.candidate_max_intra_mode_ratio_shift_percent
        || bytes_shift_percent_range > profile.candidate_max_intra_mode_bytes_shift_percent
    {
        return "CAMPAIGN_SET_WARN";
    }
    "CAMPAIGN_SET_STABLE"
}

fn campaign_set_decision(status: &str) -> &'static str {
    match status {
        "CAMPAIGN_SET_UNSTABLE" => "NO_GO_P63_MEASURED_RATIO_STABILITY",
        "CAMPAIGN_SET_MIXED_MODES" | "CAMPAIGN_SET_MIXED_PROFILES" => "RECALIBRATE_P63_WORKLOADS",
        _ => "RECALIBRATE_P63_THRESHOLDS",
    }
}

fn campaign_set_reasons(
    status: &str,
    campaign_count: usize,
    total_runs: usize,
    min_repeat_count: usize,
    ratio_shift_percent_range: f64,
    bytes_shift_percent_range: f64,
    profile: &P63ThresholdProfileSpec,
) -> Vec<String> {
    let mut reasons = vec![
        "campaign set summaries are local-first analysis artifacts".to_string(),
        format!("campaign_set_status: {}", status),
        format!("campaign_count: {}", campaign_count),
        format!("total_runs: {}", total_runs),
        format!("min_repeat_count: {}", min_repeat_count),
        format!(
            "candidate_min_campaigns_for_future_validation: {}",
            profile.candidate_min_campaigns_for_future_validation
        ),
        format!(
            "candidate_min_runs_for_future_validation: {}",
            profile.candidate_min_runs_for_future_validation
        ),
        format!(
            "candidate_max_intra_mode_ratio_shift_percent: {:.6}",
            profile.candidate_max_intra_mode_ratio_shift_percent
        ),
        format!(
            "candidate_max_intra_mode_bytes_shift_percent: {:.6}",
            profile.candidate_max_intra_mode_bytes_shift_percent
        ),
        format!(
            "ratio_shift_percent_range: {:.6}",
            ratio_shift_percent_range
        ),
        format!(
            "bytes_shift_percent_range: {:.6}",
            bytes_shift_percent_range
        ),
        format!(
            "candidate_requires_multi_machine: {}",
            profile.candidate_requires_multi_machine
        ),
        format!("allow_validate: {}", profile.allow_validate),
    ];
    reasons.push("scientific validation remains disabled in this prompt".to_string());
    reasons
}

fn percent_range(values: &[f64]) -> f64 {
    let finite: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if finite.len() < 2 {
        return 0.0;
    }
    let min = finite.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max = finite.iter().copied().reduce(f64::max).unwrap_or(0.0);
    percent_shift(min, max).abs()
}

fn parse_registry_entries(registry_json: &str) -> Vec<P63CampaignRegistryEntry> {
    let Some(campaigns_idx) = registry_json.find("\"campaigns\"") else {
        return Vec::new();
    };
    let Some(open_idx_rel) = registry_json[campaigns_idx..].find('[') else {
        return Vec::new();
    };
    let array_start = campaigns_idx + open_idx_rel + 1;
    let mut entries = Vec::new();
    let mut depth = 0usize;
    let mut object_start = None;
    for (offset, ch) in registry_json[array_start..].char_indices() {
        let idx = array_start + offset;
        match ch {
            '{' => {
                if depth == 0 {
                    object_start = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    if let Some(start) = object_start.take() {
                        let object = &registry_json[start..=idx];
                        if let Some(entry) = P63CampaignRegistryEntry::from_registry_json(object) {
                            entries.push(entry);
                        }
                    }
                }
            }
            ']' if depth == 0 => break,
            _ => {}
        }
    }
    entries
}

fn unique_strings<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    values
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn single_or_mixed(values: &[String]) -> &str {
    match values {
        [] => "unknown",
        [value] => value.as_str(),
        _ => "mixed",
    }
}

#[derive(Debug, Clone)]
struct ParsedCampaign {
    valid: bool,
    campaign_id: String,
    mode: String,
    threshold_profile: String,
    median_ratio: f64,
    median_bytes: f64,
    decision: String,
    stability: String,
}

impl ParsedCampaign {
    fn from_json(path: &str, json: &str) -> Self {
        let threshold_profile = extract_json_string(json, "threshold_profile_resolved")
            .or_else(|| extract_json_string(json, "threshold_profile"));
        let parsed = Self {
            valid: true,
            campaign_id: extract_json_string(json, "campaign_id").unwrap_or_default(),
            mode: extract_json_string(json, "mode").unwrap_or_default(),
            threshold_profile: threshold_profile.unwrap_or_default(),
            median_ratio: extract_metric_median(json, "ratio_effective_per_byte").unwrap_or(0.0),
            median_bytes: extract_metric_median(json, "total_persisted_bytes").unwrap_or(0.0),
            decision: extract_json_string(json, "decision").unwrap_or_default(),
            stability: extract_json_string(json, "campaign_stability_status").unwrap_or_default(),
        };
        if parsed.campaign_id.is_empty()
            || parsed.mode.is_empty()
            || parsed.threshold_profile.is_empty()
            || parsed.decision.is_empty()
            || parsed.stability.is_empty()
        {
            return Self {
                valid: false,
                campaign_id: path.to_string(),
                mode: "unknown".to_string(),
                threshold_profile: "unknown".to_string(),
                median_ratio: 0.0,
                median_bytes: 0.0,
                decision: "unknown".to_string(),
                stability: "unknown".to_string(),
            };
        }
        parsed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum P63CampaignCompatibilityStatus {
    SameModeComparable,
    DifferentModesInformational,
    IncompatibleProfile,
    MissingFields,
    InvalidReport,
}

impl P63CampaignCompatibilityStatus {
    fn as_str(self) -> &'static str {
        match self {
            P63CampaignCompatibilityStatus::SameModeComparable => "SAME_MODE_COMPARABLE",
            P63CampaignCompatibilityStatus::DifferentModesInformational => {
                "DIFFERENT_MODES_INFORMATIONAL"
            }
            P63CampaignCompatibilityStatus::IncompatibleProfile => "INCOMPATIBLE_PROFILE",
            P63CampaignCompatibilityStatus::MissingFields => "MISSING_FIELDS",
            P63CampaignCompatibilityStatus::InvalidReport => "INVALID_REPORT",
        }
    }
}

fn compatibility_status(a: &ParsedCampaign, b: &ParsedCampaign) -> P63CampaignCompatibilityStatus {
    if !a.valid || !b.valid {
        return P63CampaignCompatibilityStatus::InvalidReport;
    }
    if a.median_ratio == 0.0
        || b.median_ratio == 0.0
        || a.median_bytes == 0.0
        || b.median_bytes == 0.0
    {
        return P63CampaignCompatibilityStatus::MissingFields;
    }
    if a.threshold_profile != b.threshold_profile {
        return P63CampaignCompatibilityStatus::IncompatibleProfile;
    }
    if a.mode != b.mode {
        return P63CampaignCompatibilityStatus::DifferentModesInformational;
    }
    P63CampaignCompatibilityStatus::SameModeComparable
}

fn comparison_interpretation(status: P63CampaignCompatibilityStatus) -> &'static str {
    match status {
        P63CampaignCompatibilityStatus::SameModeComparable => {
            "campaigns share mode and threshold profile; deltas are directly comparable"
        }
        P63CampaignCompatibilityStatus::DifferentModesInformational => {
            "campaigns use different modes; deltas are informational and not a regression claim"
        }
        P63CampaignCompatibilityStatus::IncompatibleProfile => {
            "campaigns use different threshold profiles; compare only after recalibration"
        }
        P63CampaignCompatibilityStatus::MissingFields => {
            "one campaign is missing required numeric fields"
        }
        P63CampaignCompatibilityStatus::InvalidReport => {
            "one input is not a valid P63 campaign report"
        }
    }
}

fn comparison_decision(status: P63CampaignCompatibilityStatus) -> &'static str {
    match status {
        P63CampaignCompatibilityStatus::SameModeComparable => "COMPARE_P63_SAME_MODE_INFORMATIONAL",
        P63CampaignCompatibilityStatus::DifferentModesInformational => {
            "COMPARE_P63_DIFFERENT_MODES_INFORMATIONAL"
        }
        P63CampaignCompatibilityStatus::IncompatibleProfile => "RECALIBRATE_P63_COMPARISON_PROFILE",
        P63CampaignCompatibilityStatus::MissingFields => "NO_GO_P63_COMPARISON_MISSING_FIELDS",
        P63CampaignCompatibilityStatus::InvalidReport => "NO_GO_P63_COMPARISON_INVALID_REPORT",
    }
}

fn intra_mode_status(
    compatibility_status: P63CampaignCompatibilityStatus,
    ratio_shift_percent: f64,
    bytes_shift_percent: f64,
    same_decision: bool,
) -> &'static str {
    if compatibility_status != P63CampaignCompatibilityStatus::SameModeComparable {
        return "INTRA_MODE_NOT_ENOUGH_DATA";
    }
    let profile = P63ThresholdProfile::P63.spec();
    let ratio_shift = ratio_shift_percent.abs();
    let bytes_shift = bytes_shift_percent.abs();
    if ratio_shift <= profile.candidate_max_intra_mode_ratio_shift_percent
        && bytes_shift <= profile.candidate_max_intra_mode_bytes_shift_percent
        && same_decision
    {
        "INTRA_MODE_STABLE"
    } else if ratio_shift <= profile.candidate_max_intra_mode_ratio_shift_percent * 2.0
        && bytes_shift <= profile.candidate_max_intra_mode_bytes_shift_percent * 2.0
    {
        "INTRA_MODE_WARN"
    } else {
        "INTRA_MODE_UNSTABLE"
    }
}

fn percent_shift(a: f64, b: f64) -> f64 {
    if a == 0.0 {
        0.0
    } else {
        ((b - a) / a) * 100.0
    }
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
    out.push_str(&json_string(
        "threshold_profile_resolved",
        &report.threshold_profile_resolved,
        true,
        2,
    ));
    threshold_profile_config_json(&mut out, &report.threshold_profile_config);
    out.push_str(&json_usize("repeat_count", report.repeat_count, true, 2));
    out.push_str(&json_usize(
        "operation_count",
        report.operation_count,
        true,
        2,
    ));
    machine_metadata_json(&mut out, &report.machine_metadata);
    campaign_summary_json(&mut out, &report.summary);
    core_ratio_metrics_json(&mut out, &report.core_metrics);
    out.push_str(&json_string(
        "ratio_stability_status",
        report.ratio_stability_status.as_str(),
        true,
        2,
    ));
    out.push_str(&json_string(
        "bytes_stability_status",
        report.bytes_stability_status.as_str(),
        true,
        2,
    ));
    out.push_str(&json_string(
        "timing_stability_status",
        report.timing_stability_status.as_str(),
        true,
        2,
    ));
    out.push_str(&json_string(
        "campaign_stability_status",
        report.campaign_stability_status.as_str(),
        true,
        2,
    ));
    string_array_json(
        &mut out,
        "stability_reasons",
        &report.stability_reasons,
        true,
        2,
    );
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
        out.push_str(&run_jsonl(report, run));
        out.push('\n');
    }
    out
}

fn p63_runs_csv(report: &P63CampaignReport) -> String {
    let mut out = String::new();
    out.push_str("campaign_id,run_index,mode,threshold_profile,ratio_effective_per_byte,total_persisted_bytes,operation_count,decision,timestamp_utc,read_p99_us,update_p99_us,snapshot_p99_us,rebuild_p99_us,audit_p99_us,all_gates_passed\n");
    for run in &report.runs {
        out.push_str(&format!(
            "{},{},{},{},{:.6},{},{},{},{},{},{},{},{},{},{}\n",
            report.campaign_id,
            run.run_index,
            report.mode,
            report.threshold_profile_resolved,
            run.ratio_effective_per_byte,
            run.total_persisted_bytes,
            report.operation_count,
            report.decision.as_str(),
            report.machine_metadata.timestamp_utc,
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
         - Campaign stability: `{}`\n\
         - Threshold profile: `{}` (`{}`)\n\
         - Median ratio_effective_per_byte: `{:.6}`\n\
         - Median total_persisted_bytes: `{:.0}`\n\
         - Virtual effective units: `{}`\n\
         - Effective gain vs materialized: `{:.6}`\n\
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
        report.campaign_stability_status.as_str(),
        report.threshold_profile,
        report.threshold_profile_resolved,
        report.summary.ratio_effective_per_byte.median,
        report.summary.total_persisted_bytes.median,
        report.core_metrics.virtual_effective_units,
        report.core_metrics.effective_gain_vs_materialized,
        report.cost_model,
        report.measurement_kind
    )
}

fn p63_stability(
    summary: &P63CampaignSummary,
    profile: &P63ThresholdProfileSpec,
) -> P63StabilityEvaluation {
    let mut reasons = Vec::new();
    if summary.run_count == 0 {
        return P63StabilityEvaluation {
            ratio: P63StabilityStatus::NotAvailable,
            bytes: P63StabilityStatus::NotAvailable,
            timing: P63StabilityStatus::NotAvailable,
            campaign: P63StabilityStatus::NotAvailable,
            reasons: vec!["no measured runs are available".to_string()],
        };
    }

    if summary.run_count < profile.min_runs_required {
        reasons.push(format!(
            "run_count {} is below min_runs_required {}",
            summary.run_count, profile.min_runs_required
        ));
        return P63StabilityEvaluation {
            ratio: P63StabilityStatus::NotEnoughRuns,
            bytes: P63StabilityStatus::NotEnoughRuns,
            timing: P63StabilityStatus::NotEnoughRuns,
            campaign: P63StabilityStatus::NotEnoughRuns,
            reasons,
        };
    }

    let ratio = stability_for_cv(
        "ratio_effective_per_byte",
        summary.ratio_effective_per_byte.coefficient_of_variation,
        profile.max_ratio_cv,
        false,
        &mut reasons,
    );
    let bytes = stability_for_cv(
        "total_persisted_bytes",
        summary.total_persisted_bytes.coefficient_of_variation,
        profile.max_bytes_cv,
        false,
        &mut reasons,
    );
    let timing_cv = max_f64(&[
        summary.read_p99_us.coefficient_of_variation,
        summary.update_p99_us.coefficient_of_variation,
        summary.snapshot_p99_us.coefficient_of_variation,
        summary.rebuild_p99_us.coefficient_of_variation,
        summary.audit_p99_us.coefficient_of_variation,
    ]);
    let timing = match profile.max_timing_cv {
        Some(max_timing_cv) => {
            stability_for_cv("timing_p99", timing_cv, max_timing_cv, true, &mut reasons)
        }
        None => {
            reasons.push("timing stability threshold is not configured".to_string());
            P63StabilityStatus::NotAvailable
        }
    };
    let campaign = aggregate_stability(&[ratio, bytes, timing]);
    reasons.push(format!(
        "campaign stability aggregated as {}",
        campaign.as_str()
    ));
    P63StabilityEvaluation {
        ratio,
        bytes,
        timing,
        campaign,
        reasons,
    }
}

fn p63_decision(
    summary: &P63CampaignSummary,
    measured: &P62RealRatioReport,
    profile: &P63ThresholdProfileSpec,
    stability: &P63StabilityEvaluation,
) -> P63Decision {
    if measured.runs.is_empty()
        || !summary.all_runs_passed
        || stability.campaign == P63StabilityStatus::Unstable
    {
        return P63Decision::NoGoMeasuredRatioStability;
    }
    if measured.workloads.is_empty() || profile.require_realish_workloads {
        return P63Decision::RecalibrateWorkloads;
    }
    if !profile.allow_validate {
        return P63Decision::RecalibrateThresholds;
    }
    if stability.campaign == P63StabilityStatus::Stable {
        P63Decision::ValidateMeasuredRatioCalibration
    } else {
        P63Decision::RecalibrateThresholds
    }
}

fn p63_decision_reasons(
    decision: P63Decision,
    summary: &P63CampaignSummary,
    profile: &P63ThresholdProfileSpec,
    stability: &P63StabilityEvaluation,
) -> Vec<String> {
    let mut reasons = vec![
        "campaign exports available".to_string(),
        "robust summary available".to_string(),
        "measured_real_v1 still needs threshold calibration".to_string(),
        "no external dataset yet".to_string(),
        "timing values are machine-dependent".to_string(),
        format!("threshold profile: {}", profile.profile_id),
        format!("allow_validate: {}", profile.allow_validate),
        format!("measured run count: {}", summary.run_count),
        format!("campaign stability status: {}", stability.campaign.as_str()),
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

fn stability_for_cv(
    label: &str,
    cv: f64,
    threshold: f64,
    timing_is_soft: bool,
    reasons: &mut Vec<String>,
) -> P63StabilityStatus {
    if !cv.is_finite() {
        reasons.push(format!(
            "{} coefficient of variation is not available",
            label
        ));
        return P63StabilityStatus::NotAvailable;
    }
    if cv <= threshold {
        reasons.push(format!(
            "{} coefficient of variation {:.6} <= threshold {:.6}",
            label, cv, threshold
        ));
        return P63StabilityStatus::Stable;
    }
    if timing_is_soft {
        reasons.push(format!(
            "{} coefficient of variation {:.6} exceeds soft threshold {:.6}",
            label, cv, threshold
        ));
        if cv > threshold * 4.0 {
            P63StabilityStatus::Unstable
        } else {
            P63StabilityStatus::Warn
        }
    } else {
        reasons.push(format!(
            "{} coefficient of variation {:.6} exceeds threshold {:.6}",
            label, cv, threshold
        ));
        if cv > threshold * 2.0 {
            P63StabilityStatus::Unstable
        } else {
            P63StabilityStatus::Warn
        }
    }
}

fn aggregate_stability(statuses: &[P63StabilityStatus]) -> P63StabilityStatus {
    if statuses.contains(&P63StabilityStatus::Unstable) {
        P63StabilityStatus::Unstable
    } else if statuses.contains(&P63StabilityStatus::NotEnoughRuns) {
        P63StabilityStatus::NotEnoughRuns
    } else if statuses.contains(&P63StabilityStatus::Warn) {
        P63StabilityStatus::Warn
    } else if statuses.contains(&P63StabilityStatus::NotAvailable) {
        P63StabilityStatus::NotAvailable
    } else {
        P63StabilityStatus::Stable
    }
}

fn max_f64(samples: &[f64]) -> f64 {
    samples.iter().copied().reduce(f64::max).unwrap_or(0.0)
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

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_metric_median(json: &str, metric_name: &str) -> Option<f64> {
    let metric_needle = format!("\"{}\": {{", metric_name);
    let metric_start = json.find(&metric_needle)?;
    let metric_block = &json[metric_start..];
    extract_json_number(metric_block, "median")
}

fn extract_json_number(json: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    let end = rest
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '-'))
        .unwrap_or(rest.len());
    rest[..end].parse::<f64>().ok()
}

fn extract_object_block<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{}\": {{", key);
    let start = json.find(&needle)?;
    let object_start = start + needle.len() - 1;
    let mut depth = 0usize;
    for (offset, ch) in json[object_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(&json[object_start..=object_start + offset]);
                }
            }
            _ => {}
        }
    }
    None
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

fn threshold_profile_config_json(out: &mut String, profile: &P63ThresholdProfileSpec) {
    out.push_str("  \"threshold_profile_config\": {\n");
    out.push_str(&json_string("profile_id", profile.profile_id, true, 4));
    match profile.alias {
        Some(alias) => out.push_str(&json_string("alias", alias, true, 4)),
        None => out.push_str("    \"alias\": null,\n"),
    }
    out.push_str(&json_usize(
        "min_runs_required",
        profile.min_runs_required,
        true,
        4,
    ));
    out.push_str(&json_f64("max_ratio_cv", profile.max_ratio_cv, true, 4));
    out.push_str(&json_f64("max_bytes_cv", profile.max_bytes_cv, true, 4));
    match profile.max_timing_cv {
        Some(value) => out.push_str(&json_f64("max_timing_cv", value, true, 4)),
        None => out.push_str("    \"max_timing_cv\": null,\n"),
    }
    match profile.min_median_ratio_effective_per_byte {
        Some(value) => out.push_str(&json_f64(
            "min_median_ratio_effective_per_byte",
            value,
            true,
            4,
        )),
        None => out.push_str("    \"min_median_ratio_effective_per_byte\": null,\n"),
    }
    out.push_str(&json_bool(
        "require_machine_metadata",
        profile.require_machine_metadata,
        true,
        4,
    ));
    out.push_str(&json_bool(
        "require_campaign_exports",
        profile.require_campaign_exports,
        true,
        4,
    ));
    out.push_str(&json_bool(
        "require_realish_workloads",
        profile.require_realish_workloads,
        true,
        4,
    ));
    out.push_str(&json_bool(
        "allow_validate",
        profile.allow_validate,
        true,
        4,
    ));
    out.push_str(&json_usize(
        "candidate_min_runs_for_future_validation",
        profile.candidate_min_runs_for_future_validation,
        true,
        4,
    ));
    out.push_str(&json_usize(
        "candidate_min_campaigns_for_future_validation",
        profile.candidate_min_campaigns_for_future_validation,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "candidate_max_ratio_cv",
        profile.candidate_max_ratio_cv,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "candidate_max_bytes_cv",
        profile.candidate_max_bytes_cv,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "candidate_max_intra_mode_ratio_shift_percent",
        profile.candidate_max_intra_mode_ratio_shift_percent,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "candidate_max_intra_mode_bytes_shift_percent",
        profile.candidate_max_intra_mode_bytes_shift_percent,
        true,
        4,
    ));
    out.push_str(&json_bool(
        "candidate_requires_multi_machine",
        profile.candidate_requires_multi_machine,
        false,
        4,
    ));
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

fn core_ratio_metrics_json(out: &mut String, metrics: &P63CoreRatioMetrics) {
    out.push_str("  \"core_ratio_metrics\": {\n");
    out.push_str(&json_u128(
        "virtual_declared_units",
        metrics.virtual_declared_units,
        true,
        4,
    ));
    out.push_str(&json_u128(
        "virtual_reachable_units",
        metrics.virtual_reachable_units,
        true,
        4,
    ));
    out.push_str(&json_u128(
        "virtual_readable_units",
        metrics.virtual_readable_units,
        true,
        4,
    ));
    out.push_str(&json_u128(
        "virtual_updatable_units",
        metrics.virtual_updatable_units,
        true,
        4,
    ));
    out.push_str(&json_u128(
        "virtual_safe_units",
        metrics.virtual_safe_units,
        true,
        4,
    ));
    out.push_str(&json_u128(
        "virtual_effective_units",
        metrics.virtual_effective_units,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "total_persisted_bytes",
        metrics.total_persisted_bytes,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "payload_file_bytes",
        metrics.payload_file_bytes,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "index_file_bytes",
        metrics.index_file_bytes,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "journal_file_bytes",
        metrics.journal_file_bytes,
        true,
        4,
    ));
    out.push_str(&json_u64(
        "manifest_file_bytes",
        metrics.manifest_file_bytes,
        true,
        4,
    ));
    match metrics.checksum_or_audit_bytes {
        Some(value) => out.push_str(&json_u64("checksum_or_audit_bytes", value, true, 4)),
        None => out.push_str("    \"checksum_or_audit_bytes\": null,\n"),
    }
    match metrics.metadata_bytes {
        Some(value) => out.push_str(&json_u64("metadata_bytes", value, true, 4)),
        None => out.push_str("    \"metadata_bytes\": null,\n"),
    }
    out.push_str(&json_f64(
        "ratio_declared_per_byte",
        metrics.ratio_declared_per_byte,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "ratio_reachable_per_byte",
        metrics.ratio_reachable_per_byte,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "ratio_readable_per_byte",
        metrics.ratio_readable_per_byte,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "ratio_updatable_per_byte",
        metrics.ratio_updatable_per_byte,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "ratio_safe_per_byte",
        metrics.ratio_safe_per_byte,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "ratio_effective_per_byte",
        metrics.ratio_effective_per_byte,
        true,
        4,
    ));
    out.push_str(&json_u128(
        "assumed_materialized_value_bytes",
        metrics.assumed_materialized_value_bytes,
        true,
        4,
    ));
    out.push_str(&json_u128(
        "estimated_materialized_bytes",
        metrics.estimated_materialized_bytes,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "gain_vs_materialized",
        metrics.gain_vs_materialized,
        true,
        4,
    ));
    out.push_str(&json_f64(
        "effective_gain_vs_materialized",
        metrics.effective_gain_vs_materialized,
        false,
        4,
    ));
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

fn run_jsonl(report: &P63CampaignReport, run: &P62MeasuredRun) -> String {
    format!(
        "{{\"campaign_id\":\"{}\",\"run_index\":{},\"run_id\":\"{}\",\"mode\":\"{}\",\"threshold_profile\":\"{}\",\"ratio_effective_per_byte\":{:.6},\"total_persisted_bytes\":{},\"operation_count\":{},\"decision\":\"{}\",\"timestamp_utc\":\"{}\",\"create_count\":{},\"read_count\":{},\"update_count\":{},\"delete_count\":{},\"snapshot_count\":{},\"rebuild_count\":{},\"audit_count\":{},\"read_p99_us\":{},\"update_p99_us\":{},\"snapshot_p99_us\":{},\"rebuild_p99_us\":{},\"audit_p99_us\":{},\"guard_refused\":{},\"dangerous_or_adversarial_refused\":{},\"audit_passed\":{},\"rebuild_passed\":{},\"snapshot_roundtrip_passed\":{}}}",
        escape_json(&report.campaign_id),
        run.run_index,
        escape_json(&run.run_id),
        escape_json(&report.mode),
        escape_json(&report.threshold_profile_resolved),
        run.ratio_effective_per_byte,
        run.total_persisted_bytes,
        report.operation_count,
        report.decision.as_str(),
        escape_json(&report.machine_metadata.timestamp_utc),
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

fn f64_array_json(
    out: &mut String,
    name: &str,
    values: &[f64],
    trailing_comma: bool,
    indent: usize,
) {
    let spaces = " ".repeat(indent);
    out.push_str(&format!("{}\"{}\": [\n", spaces, name));
    for (idx, value) in values.iter().enumerate() {
        out.push_str(&format!(
            "{}  {:.6}{}\n",
            spaces,
            value,
            comma(idx, values.len())
        ));
    }
    out.push_str(&format!(
        "{}]{}\n",
        spaces,
        if trailing_comma { "," } else { "" }
    ));
}

fn u128_array_json(
    out: &mut String,
    name: &str,
    values: &[u128],
    trailing_comma: bool,
    indent: usize,
) {
    let spaces = " ".repeat(indent);
    out.push_str(&format!("{}\"{}\": [\n", spaces, name));
    for (idx, value) in values.iter().enumerate() {
        out.push_str(&format!(
            "{}  {}{}\n",
            spaces,
            value,
            comma(idx, values.len())
        ));
    }
    out.push_str(&format!(
        "{}]{}\n",
        spaces,
        if trailing_comma { "," } else { "" }
    ));
}

fn u64_array_json(
    out: &mut String,
    name: &str,
    values: &[u64],
    trailing_comma: bool,
    indent: usize,
) {
    let spaces = " ".repeat(indent);
    out.push_str(&format!("{}\"{}\": [\n", spaces, name));
    for (idx, value) in values.iter().enumerate() {
        out.push_str(&format!(
            "{}  {}{}\n",
            spaces,
            value,
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

fn json_u128(name: &str, value: u128, trailing_comma: bool, indent: usize) -> String {
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

fn ratio_u128(numerator: u128, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn median_u128(values: &[u128]) -> u128 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) / 2]
}

fn median_u64(values: &[u64]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) / 2]
}

fn median_f64(values: &[f64]) -> f64 {
    let mut sorted: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if sorted.is_empty() {
        return 0.0;
    }
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("finite samples sort"));
    sorted[(sorted.len() - 1) / 2]
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
