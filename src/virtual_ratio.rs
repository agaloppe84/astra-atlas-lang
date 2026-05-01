use crate::{validate_file, AtlasProgram, AtlasResult, WorkloadMode};

const COST_MODEL: &str = "deterministic_proxy_v1";
const RATIO_DECISION_THRESHOLD: f64 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualWorkloadKind {
    GuardRandomSpace,
    LatticeSymmetrySpace,
    SparseEventSpace,
    HybridFieldSpace,
    TopologicalAtlasSpace,
    AdversarialVirtualSpace,
}

impl VirtualWorkloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            VirtualWorkloadKind::GuardRandomSpace => "W0_guard_random_space",
            VirtualWorkloadKind::LatticeSymmetrySpace => "W1_lattice_symmetry_space",
            VirtualWorkloadKind::SparseEventSpace => "W2_sparse_event_space",
            VirtualWorkloadKind::HybridFieldSpace => "W3_hybrid_field_space",
            VirtualWorkloadKind::TopologicalAtlasSpace => "W4_topological_atlas_space",
            VirtualWorkloadKind::AdversarialVirtualSpace => "W5_adversarial_virtual_space",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            VirtualWorkloadKind::GuardRandomSpace => "W0",
            VirtualWorkloadKind::LatticeSymmetrySpace => "W1",
            VirtualWorkloadKind::SparseEventSpace => "W2",
            VirtualWorkloadKind::HybridFieldSpace => "W3",
            VirtualWorkloadKind::TopologicalAtlasSpace => "W4",
            VirtualWorkloadKind::AdversarialVirtualSpace => "W5",
        }
    }

    pub fn kind_label(self) -> &'static str {
        match self {
            VirtualWorkloadKind::GuardRandomSpace => "guard_random_space",
            VirtualWorkloadKind::LatticeSymmetrySpace => "lattice_symmetry_space",
            VirtualWorkloadKind::SparseEventSpace => "sparse_event_space",
            VirtualWorkloadKind::HybridFieldSpace => "hybrid_field_space",
            VirtualWorkloadKind::TopologicalAtlasSpace => "topological_atlas_space",
            VirtualWorkloadKind::AdversarialVirtualSpace => "adversarial_virtual_space",
        }
    }

    pub fn mechanism(self) -> &'static str {
        match self {
            VirtualWorkloadKind::GuardRandomSpace => "guard_refusal",
            VirtualWorkloadKind::LatticeSymmetrySpace => "lattice_symmetry",
            VirtualWorkloadKind::SparseEventSpace => "sparse_indexed_events",
            VirtualWorkloadKind::HybridFieldSpace => "hybrid_global_local_field",
            VirtualWorkloadKind::TopologicalAtlasSpace => "topological_atlas_gluing",
            VirtualWorkloadKind::AdversarialVirtualSpace => "adversarial_refusal",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            VirtualWorkloadKind::GuardRandomSpace => {
                "guard/random virtual address space is evaluated and refused"
            }
            VirtualWorkloadKind::LatticeSymmetrySpace => {
                "regular lattice-like generated space with deterministic reads and updates"
            }
            VirtualWorkloadKind::SparseEventSpace => {
                "sparse event space using index and journal proxy costs"
            }
            VirtualWorkloadKind::HybridFieldSpace => {
                "global-local hybrid field proxy without mathematical validation claims"
            }
            VirtualWorkloadKind::TopologicalAtlasSpace => {
                "local chart and gluing proxy with audit cost"
            }
            VirtualWorkloadKind::AdversarialVirtualSpace => {
                "apparently compressible adversarial virtual space is refused"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VirtualCostBreakdown {
    pub payload_bytes: u128,
    pub index_bytes: u128,
    pub journal_bytes: u128,
    pub manifest_bytes: u128,
    pub checksum_bytes: u128,
    pub rom_bytes: u128,
    pub redundancy_bytes: u128,
    pub metadata_bytes: u128,
    pub runtime_cost_units: u128,
}

impl VirtualCostBreakdown {
    pub fn real_total_bytes(self) -> u128 {
        self.payload_bytes
            + self.index_bytes
            + self.journal_bytes
            + self.manifest_bytes
            + self.checksum_bytes
            + self.rom_bytes
            + self.redundancy_bytes
            + self.metadata_bytes
    }

    pub fn real_total_cost_units(self) -> u128 {
        self.real_total_bytes() + self.runtime_cost_units
    }

    fn add_assign(&mut self, other: &Self) {
        self.payload_bytes += other.payload_bytes;
        self.index_bytes += other.index_bytes;
        self.journal_bytes += other.journal_bytes;
        self.manifest_bytes += other.manifest_bytes;
        self.checksum_bytes += other.checksum_bytes;
        self.rom_bytes += other.rom_bytes;
        self.redundancy_bytes += other.redundancy_bytes;
        self.metadata_bytes += other.metadata_bytes;
        self.runtime_cost_units += other.runtime_cost_units;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualWorkloadMetrics {
    pub kind: VirtualWorkloadKind,
    pub virtual_declared: u128,
    pub virtual_reachable: u128,
    pub virtual_readable: u128,
    pub virtual_updatable: u128,
    pub virtual_safe: u128,
    pub virtual_effective: u128,
    pub cost: VirtualCostBreakdown,
    pub create_count: u64,
    pub read_count: u64,
    pub update_count: u64,
    pub delete_count: u64,
    pub snapshot_count: u64,
    pub rebuild_count: u64,
    pub audit_count: u64,
    pub guard_refused: bool,
    pub dangerous_or_adversarial_refused: bool,
    pub accepted: bool,
    pub refusal_reason: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VirtualRatioMetrics {
    pub virtual_declared: u128,
    pub virtual_reachable: u128,
    pub virtual_readable: u128,
    pub virtual_updatable: u128,
    pub virtual_safe: u128,
    pub virtual_effective: u128,
    pub cost: VirtualCostBreakdown,
    pub ratio_declared: f64,
    pub ratio_addressable: f64,
    pub ratio_safe: f64,
    pub ratio_effective: f64,
    pub create_count: u64,
    pub read_count: u64,
    pub update_count: u64,
    pub delete_count: u64,
    pub snapshot_count: u64,
    pub rebuild_count: u64,
    pub audit_count: u64,
    pub guard_refused: bool,
    pub dangerous_or_adversarial_refused: bool,
    pub ordering_invariants_hold: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P61Decision {
    ValidateVirtualRatioCore,
    RecalibrateRatioCostModel,
    RecalibrateAddressability,
    NoGoVirtualRatio,
}

impl P61Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            P61Decision::ValidateVirtualRatioCore => "VALIDATE_P61_VIRTUAL_RATIO_CORE",
            P61Decision::RecalibrateRatioCostModel => "RECALIBRATE_P61_RATIO_COST_MODEL",
            P61Decision::RecalibrateAddressability => "RECALIBRATE_P61_ADDRESSABILITY",
            P61Decision::NoGoVirtualRatio => "NO_GO_P61_VIRTUAL_RATIO",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct P61VirtualRatioReport {
    pub astra_iteration: String,
    pub mode: String,
    pub program_path: String,
    pub cost_model: String,
    pub metrics: VirtualRatioMetrics,
    pub read_p50_us: Option<u64>,
    pub read_p95_us: Option<u64>,
    pub read_p99_us: Option<u64>,
    pub update_p50_us: Option<u64>,
    pub update_p95_us: Option<u64>,
    pub update_p99_us: Option<u64>,
    pub decision: P61Decision,
    pub warnings: Vec<String>,
    pub workloads: Vec<VirtualWorkloadMetrics>,
}

pub fn p61_virtual_ratio_report_file(
    path: &str,
    mode: WorkloadMode,
) -> AtlasResult<P61VirtualRatioReport> {
    let program = validate_file(path)?;
    Ok(p61_virtual_ratio_report_from_program(
        &program,
        path.to_string(),
        mode,
    ))
}

pub fn p61_virtual_ratio_report_json_file(path: &str, mode: WorkloadMode) -> AtlasResult<String> {
    p61_virtual_ratio_report_file(path, mode)
        .map(|report| p61_virtual_ratio_report_to_json(&report))
}

pub fn p61_virtual_ratio_report_from_program(
    program: &AtlasProgram,
    program_path: String,
    mode: WorkloadMode,
) -> P61VirtualRatioReport {
    let workloads = virtual_workloads(program, mode);
    let metrics = VirtualRatioMetrics::from_workloads(&workloads);
    let decision = p61_decision(&metrics);
    let mut warnings = vec![
        "deterministic proxy v1; no scientific validation is claimed".to_string(),
        "ratio_declared is informational only; decision uses ratio_effective".to_string(),
        "real microsecond timing is not measured; timing fields are null".to_string(),
    ];
    if matches!(mode, WorkloadMode::Smoke) {
        warnings
            .push("smoke mode is deterministic and CI-safe but intentionally partial".to_string());
    }
    if matches!(mode, WorkloadMode::Ambitious) {
        warnings.push("ambitious mode is local/manual and not a CI requirement".to_string());
    }
    if decision == P61Decision::RecalibrateRatioCostModel {
        warnings.push("cost model requires calibration before validation".to_string());
    }

    P61VirtualRatioReport {
        astra_iteration: "ASTRA-P61".to_string(),
        mode: mode.as_str().to_string(),
        program_path,
        cost_model: COST_MODEL.to_string(),
        metrics,
        read_p50_us: None,
        read_p95_us: None,
        read_p99_us: None,
        update_p50_us: None,
        update_p95_us: None,
        update_p99_us: None,
        decision,
        warnings,
        workloads,
    }
}

impl VirtualRatioMetrics {
    fn from_workloads(workloads: &[VirtualWorkloadMetrics]) -> Self {
        let mut cost = VirtualCostBreakdown::default();
        let mut virtual_declared = 0_u128;
        let mut virtual_reachable = 0_u128;
        let mut virtual_readable = 0_u128;
        let mut virtual_updatable = 0_u128;
        let mut virtual_safe = 0_u128;
        let mut virtual_effective = 0_u128;
        let mut create_count = 0_u64;
        let mut read_count = 0_u64;
        let mut update_count = 0_u64;
        let mut delete_count = 0_u64;
        let mut snapshot_count = 0_u64;
        let mut rebuild_count = 0_u64;
        let mut audit_count = 0_u64;
        let mut guard_refused = false;
        let mut dangerous_or_adversarial_refused = false;

        for workload in workloads {
            virtual_declared += workload.virtual_declared;
            virtual_reachable += workload.virtual_reachable;
            virtual_readable += workload.virtual_readable;
            virtual_updatable += workload.virtual_updatable;
            virtual_safe += workload.virtual_safe;
            virtual_effective += workload.virtual_effective;
            cost.add_assign(&workload.cost);
            create_count += workload.create_count;
            read_count += workload.read_count;
            update_count += workload.update_count;
            delete_count += workload.delete_count;
            snapshot_count += workload.snapshot_count;
            rebuild_count += workload.rebuild_count;
            audit_count += workload.audit_count;
            guard_refused |= workload.guard_refused;
            dangerous_or_adversarial_refused |= workload.dangerous_or_adversarial_refused;
        }

        let real_total_cost_units = cost.real_total_cost_units();
        let ordering_invariants_hold = virtual_effective <= virtual_safe
            && virtual_safe <= virtual_updatable
            && virtual_updatable <= virtual_readable
            && virtual_readable <= virtual_reachable
            && virtual_reachable <= virtual_declared;

        Self {
            virtual_declared,
            virtual_reachable,
            virtual_readable,
            virtual_updatable,
            virtual_safe,
            virtual_effective,
            cost,
            ratio_declared: ratio(virtual_declared, real_total_cost_units),
            ratio_addressable: ratio(virtual_reachable, real_total_cost_units),
            ratio_safe: ratio(virtual_safe, real_total_cost_units),
            ratio_effective: ratio(virtual_effective, real_total_cost_units),
            create_count,
            read_count,
            update_count,
            delete_count,
            snapshot_count,
            rebuild_count,
            audit_count,
            guard_refused,
            dangerous_or_adversarial_refused,
            ordering_invariants_hold,
        }
    }
}

fn virtual_workloads(program: &AtlasProgram, mode: WorkloadMode) -> Vec<VirtualWorkloadMetrics> {
    let family_count = program.families.len() as u128;
    let mut workloads = vec![
        guard_random_space(family_count),
        lattice_symmetry_space(family_count),
        adversarial_virtual_space(family_count),
    ];

    if matches!(mode, WorkloadMode::Standard | WorkloadMode::Ambitious) {
        workloads.insert(2, sparse_event_space(family_count));
        workloads.insert(3, hybrid_field_space(family_count));
        workloads.insert(4, topological_atlas_space(family_count));
    }

    if matches!(mode, WorkloadMode::Ambitious) {
        for workload in &mut workloads {
            workload.cost.runtime_cost_units +=
                workload.read_count as u128 + workload.update_count as u128;
            workload.audit_count += 1;
        }
    }

    workloads
}

fn guard_random_space(family_count: u128) -> VirtualWorkloadMetrics {
    VirtualWorkloadMetrics {
        kind: VirtualWorkloadKind::GuardRandomSpace,
        virtual_declared: 1_000_000 * family_count.max(1),
        virtual_reachable: 0,
        virtual_readable: 0,
        virtual_updatable: 0,
        virtual_safe: 0,
        virtual_effective: 0,
        cost: VirtualCostBreakdown {
            manifest_bytes: 128,
            checksum_bytes: 64,
            metadata_bytes: 256,
            runtime_cost_units: 20,
            ..VirtualCostBreakdown::default()
        },
        create_count: 0,
        read_count: 0,
        update_count: 0,
        delete_count: 1,
        snapshot_count: 0,
        rebuild_count: 0,
        audit_count: 1,
        guard_refused: true,
        dangerous_or_adversarial_refused: false,
        accepted: false,
        refusal_reason: "guard_random_space".to_string(),
        note: "guard random virtual space is refused and contributes no effective space"
            .to_string(),
    }
}

fn lattice_symmetry_space(family_count: u128) -> VirtualWorkloadMetrics {
    VirtualWorkloadMetrics {
        kind: VirtualWorkloadKind::LatticeSymmetrySpace,
        virtual_declared: 10_000 * family_count.max(1),
        virtual_reachable: 9_500 * family_count.max(1),
        virtual_readable: 9_500 * family_count.max(1),
        virtual_updatable: 9_000 * family_count.max(1),
        virtual_safe: 9_000 * family_count.max(1),
        virtual_effective: 9_000 * family_count.max(1),
        cost: VirtualCostBreakdown {
            payload_bytes: 512,
            index_bytes: 384,
            journal_bytes: 128,
            manifest_bytes: 96,
            checksum_bytes: 64,
            rom_bytes: 256,
            redundancy_bytes: 128,
            metadata_bytes: 128,
            runtime_cost_units: 100,
        },
        create_count: 32,
        read_count: 64,
        update_count: 16,
        delete_count: 0,
        snapshot_count: 1,
        rebuild_count: 1,
        audit_count: 1,
        guard_refused: false,
        dangerous_or_adversarial_refused: false,
        accepted: true,
        refusal_reason: "none".to_string(),
        note: "regular generated proxy space with deterministic reachability".to_string(),
    }
}

fn sparse_event_space(family_count: u128) -> VirtualWorkloadMetrics {
    VirtualWorkloadMetrics {
        kind: VirtualWorkloadKind::SparseEventSpace,
        virtual_declared: 500_000 * family_count.max(1),
        virtual_reachable: 100_000 * family_count.max(1),
        virtual_readable: 80_000 * family_count.max(1),
        virtual_updatable: 70_000 * family_count.max(1),
        virtual_safe: 68_000 * family_count.max(1),
        virtual_effective: 68_000 * family_count.max(1),
        cost: VirtualCostBreakdown {
            payload_bytes: 1_024,
            index_bytes: 4_096,
            journal_bytes: 2_048,
            manifest_bytes: 128,
            checksum_bytes: 128,
            rom_bytes: 512,
            redundancy_bytes: 512,
            metadata_bytes: 512,
            runtime_cost_units: 500,
        },
        create_count: 128,
        read_count: 256,
        update_count: 64,
        delete_count: 8,
        snapshot_count: 1,
        rebuild_count: 1,
        audit_count: 1,
        guard_refused: false,
        dangerous_or_adversarial_refused: false,
        accepted: true,
        refusal_reason: "none".to_string(),
        note: "sparse event proxy pays index and journal costs".to_string(),
    }
}

fn hybrid_field_space(family_count: u128) -> VirtualWorkloadMetrics {
    VirtualWorkloadMetrics {
        kind: VirtualWorkloadKind::HybridFieldSpace,
        virtual_declared: 100_000 * family_count.max(1),
        virtual_reachable: 60_000 * family_count.max(1),
        virtual_readable: 55_000 * family_count.max(1),
        virtual_updatable: 40_000 * family_count.max(1),
        virtual_safe: 38_000 * family_count.max(1),
        virtual_effective: 38_000 * family_count.max(1),
        cost: VirtualCostBreakdown {
            payload_bytes: 2_048,
            index_bytes: 1_024,
            journal_bytes: 1_024,
            manifest_bytes: 256,
            checksum_bytes: 128,
            rom_bytes: 2_048,
            redundancy_bytes: 1_024,
            metadata_bytes: 512,
            runtime_cost_units: 600,
        },
        create_count: 64,
        read_count: 192,
        update_count: 96,
        delete_count: 4,
        snapshot_count: 1,
        rebuild_count: 1,
        audit_count: 1,
        guard_refused: false,
        dangerous_or_adversarial_refused: false,
        accepted: true,
        refusal_reason: "none".to_string(),
        note: "hybrid field proxy for global rule plus localized atoms; no math validation claimed"
            .to_string(),
    }
}

fn topological_atlas_space(family_count: u128) -> VirtualWorkloadMetrics {
    VirtualWorkloadMetrics {
        kind: VirtualWorkloadKind::TopologicalAtlasSpace,
        virtual_declared: 250_000 * family_count.max(1),
        virtual_reachable: 125_000 * family_count.max(1),
        virtual_readable: 110_000 * family_count.max(1),
        virtual_updatable: 95_000 * family_count.max(1),
        virtual_safe: 90_000 * family_count.max(1),
        virtual_effective: 90_000 * family_count.max(1),
        cost: VirtualCostBreakdown {
            payload_bytes: 1_536,
            index_bytes: 3_072,
            journal_bytes: 1_536,
            manifest_bytes: 512,
            checksum_bytes: 256,
            rom_bytes: 1_024,
            redundancy_bytes: 2_048,
            metadata_bytes: 1_024,
            runtime_cost_units: 900,
        },
        create_count: 96,
        read_count: 224,
        update_count: 80,
        delete_count: 4,
        snapshot_count: 1,
        rebuild_count: 1,
        audit_count: 2,
        guard_refused: false,
        dangerous_or_adversarial_refused: false,
        accepted: true,
        refusal_reason: "none".to_string(),
        note: "topological atlas proxy assumes local chart gluing audit passes".to_string(),
    }
}

fn adversarial_virtual_space(family_count: u128) -> VirtualWorkloadMetrics {
    VirtualWorkloadMetrics {
        kind: VirtualWorkloadKind::AdversarialVirtualSpace,
        virtual_declared: 1_000_000 * family_count.max(1),
        virtual_reachable: 0,
        virtual_readable: 0,
        virtual_updatable: 0,
        virtual_safe: 0,
        virtual_effective: 0,
        cost: VirtualCostBreakdown {
            payload_bytes: 64,
            index_bytes: 128,
            journal_bytes: 512,
            manifest_bytes: 128,
            checksum_bytes: 128,
            rom_bytes: 128,
            metadata_bytes: 256,
            runtime_cost_units: 200,
            ..VirtualCostBreakdown::default()
        },
        create_count: 0,
        read_count: 0,
        update_count: 0,
        delete_count: 1,
        snapshot_count: 0,
        rebuild_count: 0,
        audit_count: 1,
        guard_refused: false,
        dangerous_or_adversarial_refused: true,
        accepted: false,
        refusal_reason: "adversarial_or_dangerous_space".to_string(),
        note: "adversarial apparently-compressible virtual space is refused".to_string(),
    }
}

fn p61_decision(metrics: &VirtualRatioMetrics) -> P61Decision {
    if !metrics.guard_refused
        || !metrics.dangerous_or_adversarial_refused
        || metrics.real_total_cost_units() == 0
        || metrics.virtual_effective == 0
    {
        return P61Decision::NoGoVirtualRatio;
    }
    if !metrics.ordering_invariants_hold {
        return P61Decision::RecalibrateAddressability;
    }
    let cost_model_calibrated = false;
    if metrics.ratio_effective >= RATIO_DECISION_THRESHOLD && cost_model_calibrated {
        P61Decision::ValidateVirtualRatioCore
    } else {
        P61Decision::RecalibrateRatioCostModel
    }
}

impl VirtualRatioMetrics {
    pub fn real_total_bytes(&self) -> u128 {
        self.cost.real_total_bytes()
    }

    pub fn real_total_cost_units(&self) -> u128 {
        self.cost.real_total_cost_units()
    }
}

impl VirtualWorkloadMetrics {
    pub fn id(&self) -> &'static str {
        self.kind.id()
    }

    pub fn kind_label(&self) -> &'static str {
        self.kind.kind_label()
    }

    pub fn mechanism(&self) -> &'static str {
        self.kind.mechanism()
    }

    pub fn description(&self) -> &'static str {
        self.kind.description()
    }

    pub fn refused(&self) -> bool {
        !self.accepted
    }

    pub fn real_total_bytes(&self) -> u128 {
        self.cost.real_total_bytes()
    }

    pub fn real_total_cost_units(&self) -> u128 {
        self.cost.real_total_cost_units()
    }

    pub fn ratio_declared(&self) -> f64 {
        ratio(self.virtual_declared, self.real_total_cost_units())
    }

    pub fn ratio_addressable(&self) -> f64 {
        ratio(self.virtual_reachable, self.real_total_cost_units())
    }

    pub fn ratio_safe(&self) -> f64 {
        ratio(self.virtual_safe, self.real_total_cost_units())
    }

    pub fn ratio_effective(&self) -> f64 {
        ratio(self.virtual_effective, self.real_total_cost_units())
    }
}

pub fn p61_virtual_ratio_report_to_json(report: &P61VirtualRatioReport) -> String {
    let metrics = &report.metrics;
    let cost = metrics.cost;
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!(
        "  \"astra_iteration\": \"{}\",\n",
        escape_json(&report.astra_iteration)
    ));
    out.push_str(&format!("  \"mode\": \"{}\",\n", escape_json(&report.mode)));
    out.push_str(&format!(
        "  \"program_path\": \"{}\",\n",
        escape_json(&report.program_path)
    ));
    out.push_str(&format!(
        "  \"cost_model\": \"{}\",\n",
        escape_json(&report.cost_model)
    ));
    out.push_str(&format!(
        "  \"virtual_declared\": {},\n",
        metrics.virtual_declared
    ));
    out.push_str(&format!(
        "  \"virtual_reachable\": {},\n",
        metrics.virtual_reachable
    ));
    out.push_str(&format!(
        "  \"virtual_readable\": {},\n",
        metrics.virtual_readable
    ));
    out.push_str(&format!(
        "  \"virtual_updatable\": {},\n",
        metrics.virtual_updatable
    ));
    out.push_str(&format!("  \"virtual_safe\": {},\n", metrics.virtual_safe));
    out.push_str(&format!(
        "  \"virtual_effective\": {},\n",
        metrics.virtual_effective
    ));
    out.push_str(&format!(
        "  \"real_payload_bytes\": {},\n",
        cost.payload_bytes
    ));
    out.push_str(&format!("  \"real_index_bytes\": {},\n", cost.index_bytes));
    out.push_str(&format!(
        "  \"real_journal_bytes\": {},\n",
        cost.journal_bytes
    ));
    out.push_str(&format!(
        "  \"real_manifest_bytes\": {},\n",
        cost.manifest_bytes
    ));
    out.push_str(&format!(
        "  \"real_checksum_bytes\": {},\n",
        cost.checksum_bytes
    ));
    out.push_str(&format!("  \"real_rom_bytes\": {},\n", cost.rom_bytes));
    out.push_str(&format!(
        "  \"real_redundancy_bytes\": {},\n",
        cost.redundancy_bytes
    ));
    out.push_str(&format!(
        "  \"real_metadata_bytes\": {},\n",
        cost.metadata_bytes
    ));
    out.push_str(&format!(
        "  \"real_total_bytes\": {},\n",
        metrics.real_total_bytes()
    ));
    out.push_str(&format!(
        "  \"real_runtime_cost_units\": {},\n",
        cost.runtime_cost_units
    ));
    out.push_str(&format!(
        "  \"real_total_cost_units\": {},\n",
        metrics.real_total_cost_units()
    ));
    out.push_str(&format!(
        "  \"ratio_declared\": {:.6},\n",
        metrics.ratio_declared
    ));
    out.push_str(&format!(
        "  \"ratio_addressable\": {:.6},\n",
        metrics.ratio_addressable
    ));
    out.push_str(&format!("  \"ratio_safe\": {:.6},\n", metrics.ratio_safe));
    out.push_str(&format!(
        "  \"ratio_effective\": {:.6},\n",
        metrics.ratio_effective
    ));
    out.push_str(&format!(
        "  \"read_p50_us\": {},\n",
        option_u64_json(report.read_p50_us)
    ));
    out.push_str(&format!(
        "  \"read_p95_us\": {},\n",
        option_u64_json(report.read_p95_us)
    ));
    out.push_str(&format!(
        "  \"read_p99_us\": {},\n",
        option_u64_json(report.read_p99_us)
    ));
    out.push_str(&format!(
        "  \"update_p50_us\": {},\n",
        option_u64_json(report.update_p50_us)
    ));
    out.push_str(&format!(
        "  \"update_p95_us\": {},\n",
        option_u64_json(report.update_p95_us)
    ));
    out.push_str(&format!(
        "  \"update_p99_us\": {},\n",
        option_u64_json(report.update_p99_us)
    ));
    out.push_str(&format!("  \"create_count\": {},\n", metrics.create_count));
    out.push_str(&format!("  \"read_count\": {},\n", metrics.read_count));
    out.push_str(&format!("  \"update_count\": {},\n", metrics.update_count));
    out.push_str(&format!("  \"delete_count\": {},\n", metrics.delete_count));
    out.push_str(&format!(
        "  \"snapshot_count\": {},\n",
        metrics.snapshot_count
    ));
    out.push_str(&format!(
        "  \"rebuild_count\": {},\n",
        metrics.rebuild_count
    ));
    out.push_str(&format!("  \"audit_count\": {},\n", metrics.audit_count));
    out.push_str(&format!(
        "  \"guard_refused\": {},\n",
        metrics.guard_refused
    ));
    out.push_str(&format!(
        "  \"dangerous_or_adversarial_refused\": {},\n",
        metrics.dangerous_or_adversarial_refused
    ));
    out.push_str(&format!(
        "  \"decision\": \"{}\",\n",
        report.decision.as_str()
    ));
    out.push_str("  \"warnings\": [\n");
    for (idx, warning) in report.warnings.iter().enumerate() {
        let comma = if idx + 1 == report.warnings.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!("    \"{}\"{}\n", escape_json(warning), comma));
    }
    out.push_str("  ],\n");
    out.push_str("  \"workloads\": [\n");
    for (idx, workload) in report.workloads.iter().enumerate() {
        let comma = if idx + 1 == report.workloads.len() {
            ""
        } else {
            ","
        };
        out.push_str("    {\n");
        out.push_str(&format!(
            "      \"id\": \"{}\",\n",
            escape_json(workload.id())
        ));
        out.push_str(&format!(
            "      \"kind\": \"{}\",\n",
            escape_json(workload.kind_label())
        ));
        out.push_str(&format!(
            "      \"name\": \"{}\",\n",
            escape_json(workload.kind.as_str())
        ));
        out.push_str(&format!(
            "      \"description\": \"{}\",\n",
            escape_json(workload.description())
        ));
        out.push_str(&format!(
            "      \"mechanism\": \"{}\",\n",
            escape_json(workload.mechanism())
        ));
        out.push_str(&format!(
            "      \"virtual_declared\": {},\n",
            workload.virtual_declared
        ));
        out.push_str(&format!(
            "      \"virtual_reachable\": {},\n",
            workload.virtual_reachable
        ));
        out.push_str(&format!(
            "      \"virtual_readable\": {},\n",
            workload.virtual_readable
        ));
        out.push_str(&format!(
            "      \"virtual_updatable\": {},\n",
            workload.virtual_updatable
        ));
        out.push_str(&format!(
            "      \"virtual_safe\": {},\n",
            workload.virtual_safe
        ));
        out.push_str(&format!(
            "      \"virtual_effective\": {},\n",
            workload.virtual_effective
        ));
        out.push_str(&format!(
            "      \"real_payload_bytes\": {},\n",
            workload.cost.payload_bytes
        ));
        out.push_str(&format!(
            "      \"real_index_bytes\": {},\n",
            workload.cost.index_bytes
        ));
        out.push_str(&format!(
            "      \"real_journal_bytes\": {},\n",
            workload.cost.journal_bytes
        ));
        out.push_str(&format!(
            "      \"real_manifest_bytes\": {},\n",
            workload.cost.manifest_bytes
        ));
        out.push_str(&format!(
            "      \"real_checksum_bytes\": {},\n",
            workload.cost.checksum_bytes
        ));
        out.push_str(&format!(
            "      \"real_rom_bytes\": {},\n",
            workload.cost.rom_bytes
        ));
        out.push_str(&format!(
            "      \"real_redundancy_bytes\": {},\n",
            workload.cost.redundancy_bytes
        ));
        out.push_str(&format!(
            "      \"real_metadata_bytes\": {},\n",
            workload.cost.metadata_bytes
        ));
        out.push_str(&format!(
            "      \"real_total_bytes\": {},\n",
            workload.real_total_bytes()
        ));
        out.push_str(&format!(
            "      \"real_runtime_cost_units\": {},\n",
            workload.cost.runtime_cost_units
        ));
        out.push_str(&format!(
            "      \"real_total_cost_units\": {},\n",
            workload.real_total_cost_units()
        ));
        out.push_str(&format!(
            "      \"ratio_declared\": {:.6},\n",
            workload.ratio_declared()
        ));
        out.push_str(&format!(
            "      \"ratio_addressable\": {:.6},\n",
            workload.ratio_addressable()
        ));
        out.push_str(&format!(
            "      \"ratio_safe\": {:.6},\n",
            workload.ratio_safe()
        ));
        out.push_str(&format!(
            "      \"ratio_effective\": {:.6},\n",
            workload.ratio_effective()
        ));
        out.push_str(&format!(
            "      \"create_count\": {},\n",
            workload.create_count
        ));
        out.push_str(&format!("      \"read_count\": {},\n", workload.read_count));
        out.push_str(&format!(
            "      \"update_count\": {},\n",
            workload.update_count
        ));
        out.push_str(&format!(
            "      \"delete_count\": {},\n",
            workload.delete_count
        ));
        out.push_str(&format!(
            "      \"snapshot_count\": {},\n",
            workload.snapshot_count
        ));
        out.push_str(&format!(
            "      \"rebuild_count\": {},\n",
            workload.rebuild_count
        ));
        out.push_str(&format!(
            "      \"audit_count\": {},\n",
            workload.audit_count
        ));
        out.push_str(&format!("      \"accepted\": {},\n", workload.accepted));
        out.push_str(&format!("      \"refused\": {},\n", workload.refused()));
        out.push_str(&format!(
            "      \"guard_refused\": {},\n",
            workload.guard_refused
        ));
        out.push_str(&format!(
            "      \"dangerous_or_adversarial_refused\": {},\n",
            workload.dangerous_or_adversarial_refused
        ));
        out.push_str(&format!(
            "      \"refusal_reason\": \"{}\",\n",
            escape_json(&workload.refusal_reason)
        ));
        out.push_str(&format!(
            "      \"note\": \"{}\"\n",
            escape_json(&workload.note)
        ));
        out.push_str(&format!("    }}{}\n", comma));
    }
    out.push_str("  ]\n");
    out.push('}');
    out
}

fn ratio(numerator: u128, denominator: u128) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn option_u64_json(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
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
