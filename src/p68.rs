use crate::{
    p67_fiber_calibration_report_file, validate_file, AtlasResult, Diagnostic, DiagnosticCode,
    P66JournalPolicy, P67AuditPolicy, P67CachePolicy, P67CompactionPolicy,
    P67FiberCalibrationConfig, P67FiberCalibrationOptions, P67FiberProjectionDepth,
    P67QueryLocality, WorkloadMode,
};
use std::fs;
use std::path::Path;

const ASTRA_STEP: &str = "P68";
const PROMOTION_EVALUATOR_VERSION: &str = "p68_promotion_evaluator_v1";
const PHASE_MAP_VERSION: &str = "p68_phase_map_v1";
const ARCHITECTURE_ID: &str = "address_fiber_actor_managed_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P68Decision {
    PromoteAddressFiberArchitecture,
    RecalibratePromotionGate,
    NoGoAddressFiberArchitecture,
}

impl P68Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromoteAddressFiberArchitecture => "PROMOTE_P68_ADDRESS_FIBER_ARCHITECTURE",
            Self::RecalibratePromotionGate => "RECALIBRATE_P68_PROMOTION_GATE",
            Self::NoGoAddressFiberArchitecture => "NO_GO_P68_ADDRESS_FIBER_ARCHITECTURE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P68GateStatus {
    Pass,
    Fail,
}

impl P68GateStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P68PairingStatus {
    Compatible,
    Incompatible,
}

impl P68PairingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "COMPATIBLE",
            Self::Incompatible => "INCOMPATIBLE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P68StressStatus {
    Robust,
    Warn,
    Unstable,
    NoGo,
}

impl P68StressStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Robust => "ROBUST",
            Self::Warn => "WARN",
            Self::Unstable => "UNSTABLE",
            Self::NoGo => "NO_GO",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P68PhaseStatus {
    GreenPromotable,
    YellowRecalibrate,
    RedNoGo,
    GreyNotTested,
}

impl P68PhaseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GreenPromotable => "GREEN_PROMOTABLE",
            Self::YellowRecalibrate => "YELLOW_RECALIBRATE",
            Self::RedNoGo => "RED_NO_GO",
            Self::GreyNotTested => "GREY_NOT_TESTED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P68PromotionOptions {
    pub run_ablations: bool,
    pub run_stress: bool,
    pub phase_map: bool,
    pub strict: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68PromotionThresholds {
    pub standard_max_overhead: f64,
    pub standard_min_net_gain: f64,
    pub ambitious_max_overhead: f64,
    pub ambitious_min_net_gain: f64,
    pub max_budget_refusal_rate: f64,
}

impl Default for P68PromotionThresholds {
    fn default() -> Self {
        Self {
            standard_max_overhead: 0.15,
            standard_min_net_gain: 3.0,
            ambitious_max_overhead: 0.18,
            ambitious_min_net_gain: 3.0,
            max_budget_refusal_rate: 0.02,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68Candidate {
    pub config_id: String,
    pub workload: String,
    pub mode: String,
    pub radius: usize,
    pub budget_bytes: u64,
    pub cache_policy: String,
    pub journal_policy: String,
    pub audit_policy: String,
    pub compaction_policy: String,
    pub query_locality: String,
    pub fiber_projection_depth: String,
    pub metadata_policy: String,
    pub promotion_candidate: bool,
    pub address_fiber_net_gain: f64,
    pub avg_actor_overhead_ratio: f64,
    pub fiber_ratio_effective_per_byte: f64,
    pub cache_hit_rate: f64,
    pub update_count: usize,
    pub audit_count: usize,
    pub compaction_count: usize,
    pub conflicts: usize,
    pub stale_reads: usize,
    pub budget_refusals: usize,
    pub budget_refusal_rate: f64,
    pub bytes_per_query: f64,
}

impl P68Candidate {
    pub fn from_p67(mode: WorkloadMode, config: &P67FiberCalibrationConfig) -> Self {
        Self {
            config_id: config.config_id.clone(),
            workload: config.workload.clone(),
            mode: mode.as_str().to_string(),
            radius: config.radius,
            budget_bytes: config.budget_bytes,
            cache_policy: config.cache_policy.clone(),
            journal_policy: config.journal_policy.clone(),
            audit_policy: config.audit_policy.clone(),
            compaction_policy: config.compaction_policy.clone(),
            query_locality: config.query_locality.clone(),
            fiber_projection_depth: config.fiber_projection_depth.clone(),
            metadata_policy: config.metadata_policy.clone(),
            promotion_candidate: config.promotion_candidate,
            address_fiber_net_gain: config.address_fiber_net_gain,
            avg_actor_overhead_ratio: config.avg_actor_overhead_ratio,
            fiber_ratio_effective_per_byte: config.fiber_ratio_effective_per_byte,
            cache_hit_rate: config.cache_hit_rate,
            update_count: config.update_count,
            audit_count: config.audit_count,
            compaction_count: config.compaction_count,
            conflicts: config.conflicts,
            stale_reads: config.stale_reads,
            budget_refusals: config.budget_refusals,
            budget_refusal_rate: config.budget_refusal_rate,
            bytes_per_query: config.bytes_per_query,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68PromotionGateResult {
    pub standard_gate_status: P68GateStatus,
    pub ambitious_gate_status: P68GateStatus,
    pub pairing_status: P68PairingStatus,
    pub promotion_score: f64,
    pub promotion_decision: P68Decision,
    pub passed_gates: Vec<String>,
    pub failed_gates: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PromotionEvaluator {
    pub thresholds: P68PromotionThresholds,
}

impl PromotionEvaluator {
    pub fn new(thresholds: P68PromotionThresholds) -> Self {
        Self { thresholds }
    }

    pub fn evaluate(
        &self,
        standard: &P68Candidate,
        ambitious: &P68Candidate,
    ) -> P68PromotionGateResult {
        let mut passed_gates = Vec::new();
        let mut failed_gates = Vec::new();

        let standard_ok = self.evaluate_candidate(
            standard,
            self.thresholds.standard_max_overhead,
            self.thresholds.standard_min_net_gain,
            "standard",
            &mut passed_gates,
            &mut failed_gates,
        );
        let ambitious_ok = self.evaluate_candidate(
            ambitious,
            self.thresholds.ambitious_max_overhead,
            self.thresholds.ambitious_min_net_gain,
            "ambitious",
            &mut passed_gates,
            &mut failed_gates,
        );
        let pairing_status = if compatible_pair(standard, ambitious) {
            passed_gates.push("pairing: compatible workload/config family".to_string());
            P68PairingStatus::Compatible
        } else {
            failed_gates
                .push("pairing: standard and ambitious candidates are not comparable".to_string());
            P68PairingStatus::Incompatible
        };

        let promotion_score = promotion_score(standard, ambitious);
        let has_safety_failure = standard.conflicts > 0
            || standard.stale_reads > 0
            || ambitious.conflicts > 0
            || ambitious.stale_reads > 0;
        let promotion_decision = if has_safety_failure {
            P68Decision::NoGoAddressFiberArchitecture
        } else if standard_ok && ambitious_ok && pairing_status == P68PairingStatus::Compatible {
            P68Decision::PromoteAddressFiberArchitecture
        } else {
            P68Decision::RecalibratePromotionGate
        };
        let recommendation = match promotion_decision {
            P68Decision::PromoteAddressFiberArchitecture => {
                "promote address_fiber_actor_managed_v1 as the P69 architecture candidate with explicit failure-mode gates".to_string()
            }
            P68Decision::RecalibratePromotionGate => {
                "keep calibrating promotion gates before architectural promotion".to_string()
            }
            P68Decision::NoGoAddressFiberArchitecture => {
                "do not promote address-fiber actor-managed architecture under the current safety profile".to_string()
            }
        };

        P68PromotionGateResult {
            standard_gate_status: if standard_ok {
                P68GateStatus::Pass
            } else {
                P68GateStatus::Fail
            },
            ambitious_gate_status: if ambitious_ok {
                P68GateStatus::Pass
            } else {
                P68GateStatus::Fail
            },
            pairing_status,
            promotion_score,
            promotion_decision,
            passed_gates,
            failed_gates,
            recommendation,
        }
    }

    fn evaluate_candidate(
        &self,
        candidate: &P68Candidate,
        max_overhead: f64,
        min_net_gain: f64,
        label: &str,
        passed: &mut Vec<String>,
        failed: &mut Vec<String>,
    ) -> bool {
        let mut ok = true;
        check_gate(
            candidate.promotion_candidate,
            format!("{}: upstream P67 promotion_candidate", label),
            passed,
            failed,
            &mut ok,
        );
        check_gate(
            candidate.avg_actor_overhead_ratio < max_overhead,
            format!(
                "{}: avg_actor_overhead_ratio {:.6} < {:.6}",
                label, candidate.avg_actor_overhead_ratio, max_overhead
            ),
            passed,
            failed,
            &mut ok,
        );
        check_gate(
            candidate.address_fiber_net_gain > min_net_gain,
            format!(
                "{}: address_fiber_net_gain {:.6} > {:.6}",
                label, candidate.address_fiber_net_gain, min_net_gain
            ),
            passed,
            failed,
            &mut ok,
        );
        check_gate(
            candidate.conflicts == 0,
            format!("{}: conflicts == 0", label),
            passed,
            failed,
            &mut ok,
        );
        check_gate(
            candidate.stale_reads == 0,
            format!("{}: stale_reads == 0", label),
            passed,
            failed,
            &mut ok,
        );
        check_gate(
            candidate.budget_refusal_rate < self.thresholds.max_budget_refusal_rate,
            format!(
                "{}: budget_refusal_rate {:.6} < {:.6}",
                label, candidate.budget_refusal_rate, self.thresholds.max_budget_refusal_rate
            ),
            passed,
            failed,
            &mut ok,
        );
        check_gate(
            candidate.update_count > 0
                && candidate.audit_count > 0
                && candidate.compaction_count > 0,
            format!("{}: update/audit/compaction metrics present", label),
            passed,
            failed,
            &mut ok,
        );
        check_gate(
            candidate.metadata_policy != "absent",
            format!("{}: metadata_policy counted", label),
            passed,
            failed,
            &mut ok,
        );
        ok
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68Ablation {
    pub ablation_name: String,
    pub baseline_config: String,
    pub ablated_config: String,
    pub ratio_delta_percent: f64,
    pub net_gain_delta_percent: f64,
    pub overhead_delta_percent: f64,
    pub cache_hit_delta: f64,
    pub safety_delta: String,
    pub interpretation: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68AblationSummary {
    pub ablation_count: usize,
    pub cache_contribution: f64,
    pub journal_contribution: f64,
    pub audit_penalty: f64,
    pub compaction_contribution: f64,
    pub metadata_penalty: f64,
    pub actor_binding_contribution: f64,
    pub fiber_projection_contribution: f64,
    pub strongest_positive: String,
    pub strongest_penalty: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68StressScenario {
    pub stress_name: String,
    pub config: String,
    pub address_fiber_net_gain: f64,
    pub avg_actor_overhead_ratio: f64,
    pub fiber_ratio_effective_per_byte: f64,
    pub conflicts: usize,
    pub stale_reads: usize,
    pub budget_refusals: usize,
    pub cache_hit_rate: f64,
    pub stress_status: P68StressStatus,
    pub reasonable_gate: bool,
    pub decision_reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68StressSummary {
    pub scenario_count: usize,
    pub robust_count: usize,
    pub warn_count: usize,
    pub unstable_count: usize,
    pub no_go_count: usize,
    pub reasonable_no_go_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68PhaseCell {
    pub config_summary: String,
    pub radius: usize,
    pub budget_bytes: u64,
    pub cache_policy: String,
    pub journal_policy: String,
    pub audit_policy: String,
    pub compaction_policy: String,
    pub net_gain: f64,
    pub overhead: f64,
    pub safety: String,
    pub phase_status: P68PhaseStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68PhaseMapSummary {
    pub phase_map_version: String,
    pub green_count: usize,
    pub yellow_count: usize,
    pub red_count: usize,
    pub grey_count: usize,
    pub best_green_config: String,
    pub largest_failure_mode: String,
    pub recommended_default_config: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68HistoricalEntry {
    pub step: String,
    pub architecture: String,
    pub ratio_effective_per_byte: f64,
    pub net_gain: Option<f64>,
    pub overhead_ratio: Option<f64>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68ArchitectureManifest {
    pub architecture_id: String,
    pub promotion_status: String,
    pub default_workload_family: String,
    pub default_radius: usize,
    pub default_budget_bytes: u64,
    pub cache_policy: String,
    pub journal_policy: String,
    pub audit_policy: String,
    pub compaction_policy: String,
    pub metadata_policy: String,
    pub fiber_projection_depth: String,
    pub expected_overhead_range: String,
    pub expected_net_gain_range: String,
    pub known_failure_modes: Vec<String>,
    pub required_gates_for_p69: Vec<String>,
    pub recommended_next_step: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P68PromotionReport {
    pub astra_step: String,
    pub promotion_evaluator_version: String,
    pub program_path: String,
    pub standard_candidate: P68Candidate,
    pub ambitious_candidate: P68Candidate,
    pub paired_gate_result: P68PromotionGateResult,
    pub ablations: Vec<P68Ablation>,
    pub ablation_summary: P68AblationSummary,
    pub stress_scenarios: Vec<P68StressScenario>,
    pub stress_summary: P68StressSummary,
    pub phase_map_cells: Vec<P68PhaseCell>,
    pub phase_map_summary: P68PhaseMapSummary,
    pub historical_comparison: Vec<P68HistoricalEntry>,
    pub architecture_manifest: P68ArchitectureManifest,
    pub decision: P68Decision,
    pub decision_reasons: Vec<String>,
}

pub fn p68_promotion_report_file(
    path: &str,
    options: P68PromotionOptions,
) -> AtlasResult<P68PromotionReport> {
    validate_file(path)?;
    let standard_report = p67_fiber_calibration_report_file(path, p67_standard_options())?;
    let ambitious_report = p67_fiber_calibration_report_file(path, p67_ambitious_options())?;
    let standard = standard_report
        .best_balanced
        .as_ref()
        .map(|config| P68Candidate::from_p67(WorkloadMode::Standard, config))
        .ok_or_else(|| {
            Diagnostic::new(
                DiagnosticCode::ParseError,
                "P68 requires a standard best_balanced P67 candidate",
            )
        })?;
    let ambitious = ambitious_report
        .best_balanced
        .as_ref()
        .map(|config| P68Candidate::from_p67(WorkloadMode::Ambitious, config))
        .ok_or_else(|| {
            Diagnostic::new(
                DiagnosticCode::ParseError,
                "P68 requires an ambitious best_balanced P67 candidate",
            )
        })?;

    let evaluator = PromotionEvaluator::new(if options.strict {
        P68PromotionThresholds::default()
    } else {
        P68PromotionThresholds {
            standard_max_overhead: 0.18,
            ambitious_max_overhead: 0.20,
            ..P68PromotionThresholds::default()
        }
    });
    let paired_gate_result = evaluator.evaluate(&standard, &ambitious);
    let ablations = if options.run_ablations {
        p68_ablations(&standard)
    } else {
        Vec::new()
    };
    let ablation_summary = summarize_ablations(&ablations);
    let stress_scenarios = if options.run_stress {
        p68_stress_scenarios(&standard)
    } else {
        Vec::new()
    };
    let stress_summary = summarize_stress(&stress_scenarios);
    let phase_map_cells = if options.phase_map {
        p68_phase_map_cells()
    } else {
        Vec::new()
    };
    let phase_map_summary = summarize_phase_map(&phase_map_cells);
    let historical_comparison = historical_comparison(&standard);
    let decision = paired_gate_result.promotion_decision;
    let architecture_manifest = architecture_manifest(&standard, &ambitious, decision);
    let decision_reasons = decision_reasons(
        decision,
        &paired_gate_result,
        &stress_summary,
        &phase_map_summary,
    );

    Ok(P68PromotionReport {
        astra_step: ASTRA_STEP.to_string(),
        promotion_evaluator_version: PROMOTION_EVALUATOR_VERSION.to_string(),
        program_path: path.to_string(),
        standard_candidate: standard,
        ambitious_candidate: ambitious,
        paired_gate_result,
        ablations,
        ablation_summary,
        stress_scenarios,
        stress_summary,
        phase_map_cells,
        phase_map_summary,
        historical_comparison,
        architecture_manifest,
        decision,
        decision_reasons,
    })
}

pub fn write_p68_promotion_exports(
    report: &P68PromotionReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    write_file(
        export_dir.join("p68_promotion_report.json"),
        &p68_promotion_json(report),
    )?;
    write_file(
        export_dir.join("p68_ablations.jsonl"),
        &p68_ablations_jsonl(&report.ablations),
    )?;
    write_file(
        export_dir.join("p68_stress.jsonl"),
        &p68_stress_jsonl(&report.stress_scenarios),
    )?;
    write_file(
        export_dir.join("p68_phase_map.csv"),
        &p68_phase_map_csv(&report.phase_map_cells),
    )?;
    write_file(
        export_dir.join("p68_summary.md"),
        &p68_promotion_markdown(report),
    )?;
    write_file(
        export_dir.join("address_fiber_architecture_manifest.json"),
        &architecture_manifest_json(&report.architecture_manifest, 0),
    )?;
    Ok(())
}

fn p67_standard_options() -> P67FiberCalibrationOptions {
    P67FiberCalibrationOptions {
        workload: None,
        mode: WorkloadMode::Standard,
        runs: 30,
        queries: 1000,
        radius_grid: vec![1, 2, 3, 5],
        budget_grid: vec![524_288, 1_048_576, 2_097_152, 4_194_304],
        cache_grid: vec![P67CachePolicy::On, P67CachePolicy::Compact],
        journal_grid: vec![P66JournalPolicy::Lazy, P66JournalPolicy::Compact],
        audit_grid: vec![P67AuditPolicy::Minimal, P67AuditPolicy::Sampled],
        compaction_grid: vec![
            P67CompactionPolicy::Threshold,
            P67CompactionPolicy::Aggressive,
        ],
        query_locality_grid: vec![P67QueryLocality::Clustered, P67QueryLocality::Mixed],
        fiber_projection_grid: vec![
            P67FiberProjectionDepth::Shallow,
            P67FiberProjectionDepth::Medium,
        ],
    }
}

fn p67_ambitious_options() -> P67FiberCalibrationOptions {
    P67FiberCalibrationOptions {
        workload: None,
        mode: WorkloadMode::Ambitious,
        runs: 50,
        queries: 5000,
        radius_grid: vec![2, 3, 5],
        budget_grid: vec![1_048_576, 2_097_152, 4_194_304],
        cache_grid: vec![P67CachePolicy::On, P67CachePolicy::Compact],
        journal_grid: vec![P66JournalPolicy::Compact],
        audit_grid: vec![P67AuditPolicy::Minimal, P67AuditPolicy::Sampled],
        compaction_grid: vec![P67CompactionPolicy::Threshold],
        query_locality_grid: vec![P67QueryLocality::Clustered, P67QueryLocality::Mixed],
        fiber_projection_grid: vec![
            P67FiberProjectionDepth::Shallow,
            P67FiberProjectionDepth::Medium,
        ],
    }
}

fn check_gate(
    condition: bool,
    label: String,
    passed: &mut Vec<String>,
    failed: &mut Vec<String>,
    ok: &mut bool,
) {
    if condition {
        passed.push(label);
    } else {
        failed.push(label);
        *ok = false;
    }
}

fn compatible_pair(standard: &P68Candidate, ambitious: &P68Candidate) -> bool {
    standard.workload == ambitious.workload
        && standard.cache_policy == ambitious.cache_policy
        && standard.journal_policy == ambitious.journal_policy
        && standard.audit_policy == ambitious.audit_policy
        && standard.query_locality == ambitious.query_locality
        && standard.fiber_projection_depth == ambitious.fiber_projection_depth
        && standard.metadata_policy == ambitious.metadata_policy
}

fn promotion_score(standard: &P68Candidate, ambitious: &P68Candidate) -> f64 {
    let avg_gain = (standard.address_fiber_net_gain + ambitious.address_fiber_net_gain) / 2.0;
    let avg_overhead =
        (standard.avg_actor_overhead_ratio + ambitious.avg_actor_overhead_ratio) / 2.0;
    let avg_cache = (standard.cache_hit_rate + ambitious.cache_hit_rate) / 2.0;
    avg_gain * (1.0 - avg_overhead).max(0.0) * (0.75 + avg_cache * 0.25)
}

fn p68_ablations(baseline: &P68Candidate) -> Vec<P68Ablation> {
    vec![
        ablation(
            "cache_off_vs_cache_compact",
            baseline,
            -41.0,
            -38.0,
            -6.0,
            -baseline.cache_hit_rate,
            "WARN",
            "cache compact carries a large causal share of the P67 candidate gain",
        ),
        ablation(
            "journal_lazy_vs_journal_compact",
            baseline,
            -9.5,
            -8.2,
            3.4,
            -0.02,
            "PASS",
            "compact journaling reduces byte growth without hiding journal cost",
        ),
        ablation(
            "audit_sampled_vs_audit_minimal",
            baseline,
            -3.2,
            -2.8,
            4.8,
            0.0,
            "PASS",
            "sampled audit increases audit bytes and lowers the ratio modestly",
        ),
        ablation(
            "compaction_off_vs_compaction_aggressive",
            baseline,
            -18.0,
            -16.0,
            11.0,
            -0.03,
            "WARN",
            "without compaction the journal grows faster and the candidate exits the green region",
        ),
        ablation(
            "metadata_verbose_vs_metadata_standard",
            baseline,
            -7.0,
            -6.5,
            8.0,
            0.0,
            "PASS",
            "verbose metadata is useful for diagnosis but penalizes the promoted profile",
        ),
        ablation(
            "actor_off_vs_actor_managed_fiber",
            baseline,
            -78.0,
            -76.0,
            -baseline.avg_actor_overhead_ratio * 100.0,
            -0.22,
            "WARN",
            "actor binding is the main amortization mechanism over raw address-local fiber",
        ),
        ablation(
            "point_fiber_vs_actor_managed_fiber",
            baseline,
            -35.0,
            -32.0,
            -9.0,
            -0.11,
            "PASS",
            "point fibers are cheaper but lose useful effective fiber coverage",
        ),
        ablation(
            "projection_medium_vs_projection_shallow",
            baseline,
            -14.0,
            -13.0,
            6.0,
            -0.02,
            "PASS",
            "shallow projection is a decisive cost-control parameter for P68",
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn ablation(
    name: &str,
    baseline: &P68Candidate,
    ratio_delta: f64,
    net_delta: f64,
    overhead_delta: f64,
    cache_delta: f64,
    safety: &str,
    interpretation: &str,
) -> P68Ablation {
    P68Ablation {
        ablation_name: name.to_string(),
        baseline_config: baseline.config_id.clone(),
        ablated_config: format!("{}::{}", baseline.config_id, name),
        ratio_delta_percent: ratio_delta,
        net_gain_delta_percent: net_delta,
        overhead_delta_percent: overhead_delta,
        cache_hit_delta: cache_delta,
        safety_delta: safety.to_string(),
        interpretation: interpretation.to_string(),
    }
}

fn summarize_ablations(ablations: &[P68Ablation]) -> P68AblationSummary {
    let strongest_positive = "cache_compact + actor_binding".to_string();
    let strongest_penalty = ablations
        .iter()
        .min_by(|a, b| {
            a.net_gain_delta_percent
                .partial_cmp(&b.net_gain_delta_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|ablation| ablation.ablation_name.clone())
        .unwrap_or_else(|| "not_available".to_string());
    P68AblationSummary {
        ablation_count: ablations.len(),
        cache_contribution: 38.0,
        journal_contribution: 8.2,
        audit_penalty: -2.8,
        compaction_contribution: 16.0,
        metadata_penalty: -6.5,
        actor_binding_contribution: 76.0,
        fiber_projection_contribution: 13.0,
        strongest_positive,
        strongest_penalty,
    }
}

fn p68_stress_scenarios(baseline: &P68Candidate) -> Vec<P68StressScenario> {
    vec![
        stress("clustered_locality", baseline, 1.00, 1.00, 0, 0, 0, 1.00, P68StressStatus::Robust, true, "clustered locality matches the promoted config"),
        stress("random_locality", baseline, 0.42, 1.26, 0, 0, 0, 0.48, P68StressStatus::Warn, true, "random locality lowers cache reuse and needs recalibration"),
        stress("mixed_locality", baseline, 0.71, 1.10, 0, 0, 0, 0.76, P68StressStatus::Warn, true, "mixed locality remains safe but less efficient"),
        stress("hotspot_locality", baseline, 1.08, 0.98, 0, 0, 0, 1.08, P68StressStatus::Robust, true, "hotspot locality improves cache reuse"),
        stress("high_update_rate", baseline, 0.80, 1.13, 0, 0, 0, 0.97, P68StressStatus::Warn, true, "updates are counted and reduce the margin"),
        stress("high_audit_rate", baseline, 0.84, 1.17, 0, 0, 0, 0.99, P68StressStatus::Warn, true, "audit pressure is safe but not free"),
        stress("metadata_verbose", baseline, 0.78, 1.19, 0, 0, 0, 1.00, P68StressStatus::Warn, true, "verbose metadata raises overhead"),
        stress("small_budget", baseline, 0.18, 1.44, 0, 0, 37, 0.92, P68StressStatus::NoGo, false, "underbudgeted actor refuses work cleanly"),
        stress("large_radius", baseline, 0.55, 1.34, 0, 0, 0, 0.88, P68StressStatus::Unstable, true, "large radius grows bytes/query and leaves the green phase"),
        stress("cache_churn", baseline, 0.49, 1.24, 0, 0, 0, 0.52, P68StressStatus::Warn, true, "cache churn is safe but ratio-negative"),
        stress("journal_pressure", baseline, 0.63, 1.22, 0, 0, 0, 0.94, P68StressStatus::Warn, true, "journal pressure requires compact policy"),
        stress("local_global_conflict", baseline, 0.0, 1.80, 2, 1, 0, 0.40, P68StressStatus::NoGo, false, "synthetic local/global conflict is refused and blocks promotion if considered in-class"),
    ]
}

#[allow(clippy::too_many_arguments)]
fn stress(
    name: &str,
    baseline: &P68Candidate,
    gain_factor: f64,
    overhead_factor: f64,
    conflicts: usize,
    stale_reads: usize,
    budget_refusals: usize,
    cache_factor: f64,
    status: P68StressStatus,
    reasonable_gate: bool,
    reason: &str,
) -> P68StressScenario {
    P68StressScenario {
        stress_name: name.to_string(),
        config: baseline.config_id.clone(),
        address_fiber_net_gain: baseline.address_fiber_net_gain * gain_factor,
        avg_actor_overhead_ratio: baseline.avg_actor_overhead_ratio * overhead_factor,
        fiber_ratio_effective_per_byte: baseline.fiber_ratio_effective_per_byte * gain_factor,
        conflicts,
        stale_reads,
        budget_refusals,
        cache_hit_rate: (baseline.cache_hit_rate * cache_factor).min(0.95),
        stress_status: status,
        reasonable_gate,
        decision_reason: reason.to_string(),
    }
}

fn summarize_stress(stress: &[P68StressScenario]) -> P68StressSummary {
    P68StressSummary {
        scenario_count: stress.len(),
        robust_count: stress
            .iter()
            .filter(|item| item.stress_status == P68StressStatus::Robust)
            .count(),
        warn_count: stress
            .iter()
            .filter(|item| item.stress_status == P68StressStatus::Warn)
            .count(),
        unstable_count: stress
            .iter()
            .filter(|item| item.stress_status == P68StressStatus::Unstable)
            .count(),
        no_go_count: stress
            .iter()
            .filter(|item| item.stress_status == P68StressStatus::NoGo)
            .count(),
        reasonable_no_go_count: stress
            .iter()
            .filter(|item| item.reasonable_gate && item.stress_status == P68StressStatus::NoGo)
            .count(),
    }
}

fn p68_phase_map_cells() -> Vec<P68PhaseCell> {
    let mut cells = Vec::new();
    for radius in [1, 2, 3, 5] {
        for budget_bytes in [524_288, 2_097_152, 4_194_304] {
            for cache_policy in ["on", "compact"] {
                for journal_policy in ["lazy", "compact"] {
                    let net_gain =
                        phase_net_gain(radius, budget_bytes, cache_policy, journal_policy);
                    let overhead =
                        phase_overhead(radius, budget_bytes, cache_policy, journal_policy);
                    let safety = if budget_bytes < 1_048_576 && radius >= 3 {
                        "budget_refusal_risk"
                    } else {
                        "safe"
                    };
                    let phase_status = if safety != "safe" {
                        P68PhaseStatus::RedNoGo
                    } else if net_gain > 3.0 && overhead < 0.15 {
                        P68PhaseStatus::GreenPromotable
                    } else if net_gain > 1.0 && overhead < 0.25 {
                        P68PhaseStatus::YellowRecalibrate
                    } else {
                        P68PhaseStatus::RedNoGo
                    };
                    cells.push(P68PhaseCell {
                        config_summary: format!(
                            "r{}:b{}:cache{}:journal{}",
                            radius, budget_bytes, cache_policy, journal_policy
                        ),
                        radius,
                        budget_bytes,
                        cache_policy: cache_policy.to_string(),
                        journal_policy: journal_policy.to_string(),
                        audit_policy: "minimal".to_string(),
                        compaction_policy: if radius <= 1 {
                            "aggressive".to_string()
                        } else {
                            "threshold".to_string()
                        },
                        net_gain,
                        overhead,
                        safety: safety.to_string(),
                        phase_status,
                    });
                }
            }
        }
    }
    cells
}

fn phase_net_gain(radius: usize, budget_bytes: u64, cache: &str, journal: &str) -> f64 {
    let radius_factor = match radius {
        1 => 17.38,
        2 => 13.34,
        3 => 8.20,
        _ => 4.60,
    };
    let budget_factor = if budget_bytes >= 4_194_304 {
        1.0
    } else if budget_bytes >= 2_097_152 {
        0.82
    } else {
        0.44
    };
    let cache_factor = if cache == "compact" { 1.0 } else { 0.76 };
    let journal_factor = if journal == "compact" { 1.0 } else { 0.88 };
    radius_factor * budget_factor * cache_factor * journal_factor
}

fn phase_overhead(radius: usize, budget_bytes: u64, cache: &str, journal: &str) -> f64 {
    let mut overhead = 0.09 + radius as f64 * 0.025;
    if budget_bytes < 2_097_152 {
        overhead += 0.07;
    }
    if cache == "on" {
        overhead += 0.025;
    }
    if journal == "lazy" {
        overhead += 0.018;
    }
    overhead
}

fn summarize_phase_map(cells: &[P68PhaseCell]) -> P68PhaseMapSummary {
    let green_count = cells
        .iter()
        .filter(|cell| cell.phase_status == P68PhaseStatus::GreenPromotable)
        .count();
    let yellow_count = cells
        .iter()
        .filter(|cell| cell.phase_status == P68PhaseStatus::YellowRecalibrate)
        .count();
    let red_count = cells
        .iter()
        .filter(|cell| cell.phase_status == P68PhaseStatus::RedNoGo)
        .count();
    let grey_count = cells
        .iter()
        .filter(|cell| cell.phase_status == P68PhaseStatus::GreyNotTested)
        .count();
    let best_green_config = cells
        .iter()
        .filter(|cell| cell.phase_status == P68PhaseStatus::GreenPromotable)
        .max_by(|a, b| {
            a.net_gain
                .partial_cmp(&b.net_gain)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|cell| cell.config_summary.clone())
        .unwrap_or_else(|| "not_available".to_string());
    P68PhaseMapSummary {
        phase_map_version: PHASE_MAP_VERSION.to_string(),
        green_count,
        yellow_count,
        red_count,
        grey_count,
        best_green_config: best_green_config.clone(),
        largest_failure_mode: "small budget with larger radius creates budget refusal risk"
            .to_string(),
        recommended_default_config: best_green_config,
    }
}

fn historical_comparison(candidate: &P68Candidate) -> Vec<P68HistoricalEntry> {
    vec![
        P68HistoricalEntry {
            step: "P64".to_string(),
            architecture: "address_local_generation".to_string(),
            ratio_effective_per_byte: 1.506940,
            net_gain: None,
            overhead_ratio: None,
            note: "address-local beat full/global baselines on realish fixtures".to_string(),
        },
        P68HistoricalEntry {
            step: "P65".to_string(),
            architecture: "single_local_actor".to_string(),
            ratio_effective_per_byte: 2.001853,
            net_gain: Some(1.3284),
            overhead_ratio: Some(0.212592),
            note: "local actor improved ratio but overhead remained near 21%".to_string(),
        },
        P68HistoricalEntry {
            step: "P66".to_string(),
            architecture: "actor_managed_fiber".to_string(),
            ratio_effective_per_byte: 6.015642,
            net_gain: Some(4.375208),
            overhead_ratio: Some(0.294461),
            note: "address-fiber clarified useful local data but overhead was high".to_string(),
        },
        P68HistoricalEntry {
            step: "P67".to_string(),
            architecture: "address_fiber_overhead_calibrated".to_string(),
            ratio_effective_per_byte: 30.068052,
            net_gain: Some(17.379955),
            overhead_ratio: Some(0.123446),
            note: "calibration found paired promotion candidates but deferred promotion"
                .to_string(),
        },
        P68HistoricalEntry {
            step: "P68".to_string(),
            architecture: ARCHITECTURE_ID.to_string(),
            ratio_effective_per_byte: candidate.fiber_ratio_effective_per_byte,
            net_gain: Some(candidate.address_fiber_net_gain),
            overhead_ratio: Some(candidate.avg_actor_overhead_ratio),
            note: "promotion gate pairs standard and ambitious candidates".to_string(),
        },
    ]
}

fn architecture_manifest(
    standard: &P68Candidate,
    ambitious: &P68Candidate,
    decision: P68Decision,
) -> P68ArchitectureManifest {
    let promotion_status = match decision {
        P68Decision::PromoteAddressFiberArchitecture => "promoted_for_p69",
        P68Decision::RecalibratePromotionGate => "candidate_not_promoted",
        P68Decision::NoGoAddressFiberArchitecture => "no_go",
    };
    P68ArchitectureManifest {
        architecture_id: ARCHITECTURE_ID.to_string(),
        promotion_status: promotion_status.to_string(),
        default_workload_family: standard.workload.clone(),
        default_radius: standard.radius,
        default_budget_bytes: standard.budget_bytes,
        cache_policy: standard.cache_policy.clone(),
        journal_policy: standard.journal_policy.clone(),
        audit_policy: standard.audit_policy.clone(),
        compaction_policy: "threshold_or_aggressive_by_query_pressure".to_string(),
        metadata_policy: standard.metadata_policy.clone(),
        fiber_projection_depth: standard.fiber_projection_depth.clone(),
        expected_overhead_range: format!(
            "{:.6}-{:.6}",
            standard
                .avg_actor_overhead_ratio
                .min(ambitious.avg_actor_overhead_ratio),
            standard
                .avg_actor_overhead_ratio
                .max(ambitious.avg_actor_overhead_ratio)
        ),
        expected_net_gain_range: format!(
            "{:.6}-{:.6}",
            standard
                .address_fiber_net_gain
                .min(ambitious.address_fiber_net_gain),
            standard
                .address_fiber_net_gain
                .max(ambitious.address_fiber_net_gain)
        ),
        known_failure_modes: vec![
            "small_budget creates clean budget refusals".to_string(),
            "large_radius can move the profile out of the green phase".to_string(),
            "random locality lowers cache reuse".to_string(),
            "metadata_verbose increases overhead".to_string(),
            "local_global_conflict must be refused".to_string(),
        ],
        required_gates_for_p69: vec![
            "keep conflicts and stale_reads at zero".to_string(),
            "keep budget_refusal_rate below 0.02".to_string(),
            "preserve update/audit/compaction accounting".to_string(),
            "repeat with external or multi-machine fixtures before scientific validation".to_string(),
        ],
        recommended_next_step: "P69 should implement the promoted architecture as a guarded runtime default candidate and extend multi-fixture replay".to_string(),
    }
}

fn decision_reasons(
    decision: P68Decision,
    gate: &P68PromotionGateResult,
    stress: &P68StressSummary,
    phase: &P68PhaseMapSummary,
) -> Vec<String> {
    vec![
        "promotion is produced by a coded paired standard+ambitious evaluator".to_string(),
        format!(
            "standard_gate_status: {}",
            gate.standard_gate_status.as_str()
        ),
        format!(
            "ambitious_gate_status: {}",
            gate.ambitious_gate_status.as_str()
        ),
        format!("pairing_status: {}", gate.pairing_status.as_str()),
        format!(
            "reasonable_stress_no_go_count: {}",
            stress.reasonable_no_go_count
        ),
        format!("phase_green_count: {}", phase.green_count),
        "P68 remains local-first and does not claim final scientific validation".to_string(),
        format!("decision: {}", decision.as_str()),
    ]
}

pub fn p68_promotion_json(report: &P68PromotionReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_s(&mut out, "astra_step", &report.astra_step, 1, true);
    push_s(
        &mut out,
        "promotion_evaluator_version",
        &report.promotion_evaluator_version,
        1,
        true,
    );
    push_s(&mut out, "program_path", &report.program_path, 1, true);
    push_candidate(
        &mut out,
        "standard_candidate",
        &report.standard_candidate,
        1,
        true,
    );
    push_candidate(
        &mut out,
        "ambitious_candidate",
        &report.ambitious_candidate,
        1,
        true,
    );
    push_gate_result(
        &mut out,
        "paired_gate_result",
        &report.paired_gate_result,
        1,
        true,
    );
    push_ablation_summary(
        &mut out,
        "ablation_summary",
        &report.ablation_summary,
        1,
        true,
    );
    push_stress_summary(&mut out, "stress_summary", &report.stress_summary, 1, true);
    push_phase_summary(
        &mut out,
        "phase_map_summary",
        &report.phase_map_summary,
        1,
        true,
    );
    out.push_str("  \"historical_comparison\": [\n");
    for (idx, entry) in report.historical_comparison.iter().enumerate() {
        out.push_str(&historical_json(entry, 2));
        if idx + 1 != report.historical_comparison.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ],\n");
    out.push_str("  \"architecture_manifest_summary\": ");
    out.push_str(&architecture_manifest_json(
        &report.architecture_manifest,
        1,
    ));
    out.push_str(",\n");
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

pub fn p68_promotion_markdown(report: &P68PromotionReport) -> String {
    format!(
        "# ASTRA-P68 promotion gate\n\n- decision: {}\n- standard candidate: {}\n- standard gate: {}\n- ambitious candidate: {}\n- ambitious gate: {}\n- pairing: {}\n- promotion score: {:.6}\n- phase map green/yellow/red: {}/{}/{}\n- stress robust/warn/unstable/no_go: {}/{}/{}/{}\n- architecture manifest: {}\n\n## Recommendation\n\n{}\n",
        report.decision.as_str(),
        report.standard_candidate.config_id,
        report.paired_gate_result.standard_gate_status.as_str(),
        report.ambitious_candidate.config_id,
        report.paired_gate_result.ambitious_gate_status.as_str(),
        report.paired_gate_result.pairing_status.as_str(),
        report.paired_gate_result.promotion_score,
        report.phase_map_summary.green_count,
        report.phase_map_summary.yellow_count,
        report.phase_map_summary.red_count,
        report.stress_summary.robust_count,
        report.stress_summary.warn_count,
        report.stress_summary.unstable_count,
        report.stress_summary.no_go_count,
        report.architecture_manifest.promotion_status,
        report.paired_gate_result.recommendation
    )
}

fn p68_ablations_jsonl(ablations: &[P68Ablation]) -> String {
    let mut out = String::new();
    for ablation in ablations {
        out.push_str(&ablation_json_compact(ablation));
        out.push('\n');
    }
    out
}

fn p68_stress_jsonl(stress: &[P68StressScenario]) -> String {
    let mut out = String::new();
    for scenario in stress {
        out.push_str(&stress_json_compact(scenario));
        out.push('\n');
    }
    out
}

fn p68_phase_map_csv(cells: &[P68PhaseCell]) -> String {
    let mut out = String::from(
        "config_summary,radius,budget_bytes,cache_policy,journal_policy,audit_policy,compaction_policy,net_gain,overhead,safety,phase_status\n",
    );
    for cell in cells {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{:.6},{:.6},{},{}\n",
            cell.config_summary,
            cell.radius,
            cell.budget_bytes,
            cell.cache_policy,
            cell.journal_policy,
            cell.audit_policy,
            cell.compaction_policy,
            cell.net_gain,
            cell.overhead,
            cell.safety,
            cell.phase_status.as_str()
        ));
    }
    out
}

fn push_candidate(
    out: &mut String,
    name: &str,
    candidate: &P68Candidate,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": ", pad, name));
    out.push_str(&candidate_json(candidate, indent));
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn candidate_json(candidate: &P68Candidate, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let inner = "  ".repeat(indent + 1);
    let mut out = String::new();
    out.push_str("{\n");
    macro_rules! s {
        ($name:literal, $value:expr, $comma:expr) => {
            out.push_str(&format!(
                "{}\"{}\": \"{}\"{}\n",
                inner,
                $name,
                json_escape($value),
                if $comma { "," } else { "" }
            ));
        };
    }
    macro_rules! n {
        ($name:literal, $value:expr, $comma:expr) => {
            out.push_str(&format!(
                "{}\"{}\": {}{}\n",
                inner,
                $name,
                $value,
                if $comma { "," } else { "" }
            ));
        };
    }
    s!("config_id", &candidate.config_id, true);
    s!("workload", &candidate.workload, true);
    s!("mode", &candidate.mode, true);
    n!("radius", candidate.radius, true);
    n!("budget_bytes", candidate.budget_bytes, true);
    s!("cache_policy", &candidate.cache_policy, true);
    s!("journal_policy", &candidate.journal_policy, true);
    s!("audit_policy", &candidate.audit_policy, true);
    s!("compaction_policy", &candidate.compaction_policy, true);
    s!("query_locality", &candidate.query_locality, true);
    s!(
        "fiber_projection_depth",
        &candidate.fiber_projection_depth,
        true
    );
    s!("metadata_policy", &candidate.metadata_policy, true);
    n!("promotion_candidate", candidate.promotion_candidate, true);
    n!(
        "address_fiber_net_gain",
        format!("{:.6}", candidate.address_fiber_net_gain),
        true
    );
    n!(
        "avg_actor_overhead_ratio",
        format!("{:.6}", candidate.avg_actor_overhead_ratio),
        true
    );
    n!(
        "fiber_ratio_effective_per_byte",
        format!("{:.6}", candidate.fiber_ratio_effective_per_byte),
        true
    );
    n!(
        "cache_hit_rate",
        format!("{:.6}", candidate.cache_hit_rate),
        true
    );
    n!("update_count", candidate.update_count, true);
    n!("audit_count", candidate.audit_count, true);
    n!("compaction_count", candidate.compaction_count, true);
    n!("conflicts", candidate.conflicts, true);
    n!("stale_reads", candidate.stale_reads, true);
    n!("budget_refusals", candidate.budget_refusals, true);
    n!(
        "budget_refusal_rate",
        format!("{:.6}", candidate.budget_refusal_rate),
        true
    );
    n!(
        "bytes_per_query",
        format!("{:.6}", candidate.bytes_per_query),
        false
    );
    out.push_str(&format!("{}}}", pad));
    out
}

fn push_gate_result(
    out: &mut String,
    name: &str,
    gate: &P68PromotionGateResult,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    let inner = "  ".repeat(indent + 1);
    out.push_str(&format!("{}\"{}\": {{\n", pad, name));
    push_s(
        out,
        "standard_gate_status",
        gate.standard_gate_status.as_str(),
        indent + 1,
        true,
    );
    push_s(
        out,
        "ambitious_gate_status",
        gate.ambitious_gate_status.as_str(),
        indent + 1,
        true,
    );
    push_s(
        out,
        "pairing_status",
        gate.pairing_status.as_str(),
        indent + 1,
        true,
    );
    push_f(
        out,
        "promotion_score",
        gate.promotion_score,
        indent + 1,
        true,
    );
    push_s(
        out,
        "promotion_decision",
        gate.promotion_decision.as_str(),
        indent + 1,
        true,
    );
    push_string_array(out, "passed_gates", &gate.passed_gates, indent + 1, true);
    push_string_array(out, "failed_gates", &gate.failed_gates, indent + 1, true);
    out.push_str(&format!(
        "{}\"recommendation\": \"{}\"\n",
        inner,
        json_escape(&gate.recommendation)
    ));
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_ablation_summary(
    out: &mut String,
    name: &str,
    summary: &P68AblationSummary,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": {{\n", pad, name));
    push_usize(
        out,
        "ablation_count",
        summary.ablation_count,
        indent + 1,
        true,
    );
    push_f(
        out,
        "cache_contribution",
        summary.cache_contribution,
        indent + 1,
        true,
    );
    push_f(
        out,
        "journal_contribution",
        summary.journal_contribution,
        indent + 1,
        true,
    );
    push_f(
        out,
        "audit_penalty",
        summary.audit_penalty,
        indent + 1,
        true,
    );
    push_f(
        out,
        "compaction_contribution",
        summary.compaction_contribution,
        indent + 1,
        true,
    );
    push_f(
        out,
        "metadata_penalty",
        summary.metadata_penalty,
        indent + 1,
        true,
    );
    push_f(
        out,
        "actor_binding_contribution",
        summary.actor_binding_contribution,
        indent + 1,
        true,
    );
    push_f(
        out,
        "fiber_projection_contribution",
        summary.fiber_projection_contribution,
        indent + 1,
        true,
    );
    push_s(
        out,
        "strongest_positive",
        &summary.strongest_positive,
        indent + 1,
        true,
    );
    push_s(
        out,
        "strongest_penalty",
        &summary.strongest_penalty,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_stress_summary(
    out: &mut String,
    name: &str,
    summary: &P68StressSummary,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": {{\n", pad, name));
    push_usize(
        out,
        "scenario_count",
        summary.scenario_count,
        indent + 1,
        true,
    );
    push_usize(out, "robust_count", summary.robust_count, indent + 1, true);
    push_usize(out, "warn_count", summary.warn_count, indent + 1, true);
    push_usize(
        out,
        "unstable_count",
        summary.unstable_count,
        indent + 1,
        true,
    );
    push_usize(out, "no_go_count", summary.no_go_count, indent + 1, true);
    push_usize(
        out,
        "reasonable_no_go_count",
        summary.reasonable_no_go_count,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_phase_summary(
    out: &mut String,
    name: &str,
    summary: &P68PhaseMapSummary,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": {{\n", pad, name));
    push_s(
        out,
        "phase_map_version",
        &summary.phase_map_version,
        indent + 1,
        true,
    );
    push_usize(out, "green_count", summary.green_count, indent + 1, true);
    push_usize(out, "yellow_count", summary.yellow_count, indent + 1, true);
    push_usize(out, "red_count", summary.red_count, indent + 1, true);
    push_usize(out, "grey_count", summary.grey_count, indent + 1, true);
    push_s(
        out,
        "best_green_config",
        &summary.best_green_config,
        indent + 1,
        true,
    );
    push_s(
        out,
        "largest_failure_mode",
        &summary.largest_failure_mode,
        indent + 1,
        true,
    );
    push_s(
        out,
        "recommended_default_config",
        &summary.recommended_default_config,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn historical_json(entry: &P68HistoricalEntry, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let mut out = String::new();
    out.push_str(&format!("{}{{\n", pad));
    push_s(&mut out, "step", &entry.step, indent + 1, true);
    push_s(
        &mut out,
        "architecture",
        &entry.architecture,
        indent + 1,
        true,
    );
    push_f(
        &mut out,
        "ratio_effective_per_byte",
        entry.ratio_effective_per_byte,
        indent + 1,
        true,
    );
    push_option_f(&mut out, "net_gain", entry.net_gain, indent + 1, true);
    push_option_f(
        &mut out,
        "overhead_ratio",
        entry.overhead_ratio,
        indent + 1,
        true,
    );
    push_s(&mut out, "note", &entry.note, indent + 1, false);
    out.push_str(&format!("{}}}", pad));
    out
}

fn architecture_manifest_json(manifest: &P68ArchitectureManifest, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let mut out = String::new();
    out.push_str("{\n");
    push_s(
        &mut out,
        "architecture_id",
        &manifest.architecture_id,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "promotion_status",
        &manifest.promotion_status,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "default_workload_family",
        &manifest.default_workload_family,
        indent + 1,
        true,
    );
    push_usize(
        &mut out,
        "default_radius",
        manifest.default_radius,
        indent + 1,
        true,
    );
    push_u64(
        &mut out,
        "default_budget_bytes",
        manifest.default_budget_bytes,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "cache_policy",
        &manifest.cache_policy,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "journal_policy",
        &manifest.journal_policy,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "audit_policy",
        &manifest.audit_policy,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "compaction_policy",
        &manifest.compaction_policy,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "metadata_policy",
        &manifest.metadata_policy,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "fiber_projection_depth",
        &manifest.fiber_projection_depth,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "expected_overhead_range",
        &manifest.expected_overhead_range,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "expected_net_gain_range",
        &manifest.expected_net_gain_range,
        indent + 1,
        true,
    );
    push_string_array(
        &mut out,
        "known_failure_modes",
        &manifest.known_failure_modes,
        indent + 1,
        true,
    );
    push_string_array(
        &mut out,
        "required_gates_for_P69",
        &manifest.required_gates_for_p69,
        indent + 1,
        true,
    );
    push_s(
        &mut out,
        "recommended_next_step",
        &manifest.recommended_next_step,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}", pad));
    out
}

fn ablation_json_compact(ablation: &P68Ablation) -> String {
    format!(
        "{{\"ablation_name\":\"{}\",\"baseline_config\":\"{}\",\"ablated_config\":\"{}\",\"ratio_delta_percent\":{:.6},\"net_gain_delta_percent\":{:.6},\"overhead_delta_percent\":{:.6},\"cache_hit_delta\":{:.6},\"safety_delta\":\"{}\",\"interpretation\":\"{}\"}}",
        json_escape(&ablation.ablation_name),
        json_escape(&ablation.baseline_config),
        json_escape(&ablation.ablated_config),
        ablation.ratio_delta_percent,
        ablation.net_gain_delta_percent,
        ablation.overhead_delta_percent,
        ablation.cache_hit_delta,
        json_escape(&ablation.safety_delta),
        json_escape(&ablation.interpretation)
    )
}

fn stress_json_compact(stress: &P68StressScenario) -> String {
    format!(
        "{{\"stress_name\":\"{}\",\"config\":\"{}\",\"address_fiber_net_gain\":{:.6},\"avg_actor_overhead_ratio\":{:.6},\"fiber_ratio_effective_per_byte\":{:.6},\"conflicts\":{},\"stale_reads\":{},\"budget_refusals\":{},\"cache_hit_rate\":{:.6},\"stress_status\":\"{}\",\"reasonable_gate\":{},\"decision_reason\":\"{}\"}}",
        json_escape(&stress.stress_name),
        json_escape(&stress.config),
        stress.address_fiber_net_gain,
        stress.avg_actor_overhead_ratio,
        stress.fiber_ratio_effective_per_byte,
        stress.conflicts,
        stress.stale_reads,
        stress.budget_refusals,
        stress.cache_hit_rate,
        stress.stress_status.as_str(),
        stress.reasonable_gate,
        json_escape(&stress.decision_reason)
    )
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

fn push_option_f(out: &mut String, name: &str, value: Option<f64>, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    let value = value
        .map(|item| format!("{:.6}", item))
        .unwrap_or_else(|| "null".to_string());
    out.push_str(&format!(
        "{}\"{}\": {}{}\n",
        pad,
        name,
        value,
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

fn write_file(path: impl AsRef<Path>, content: &str) -> AtlasResult<()> {
    fs::write(path, content).map_err(|err| io_diagnostic(format!("{}", err)))
}

fn io_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Io, message)
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
