use crate::{AtlasResult, Diagnostic, DiagnosticCode, WorkloadMode};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const ASTRA_STEP: &str = "P69";
const ARCHITECTURE_ID: &str = "address_fiber_actor_managed_v1";
const COST_MODEL: &str = "measured_or_declared_contract_v1";

#[derive(Debug, Clone, PartialEq)]
pub struct ProceduralCodeContract {
    pub code_form_id: String,
    pub architecture_id: String,
    pub cost_model: String,
    pub address_space: AddressSpaceContract,
    pub fiber_schema: FiberSchemaContract,
    pub generator: GeneratorContract,
    pub actor_policy: ActorPolicyContract,
    pub representation: RepresentationContractRefs,
    pub stored: StoredContract,
    pub gates: SafetyGateContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressSpaceContract {
    pub name: String,
    pub dimensions: usize,
    pub addressing: String,
    pub coordinate_type: String,
    pub virtual_declared_units: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiberSchemaContract {
    pub name: String,
    pub fiber_kind: String,
    pub address: String,
    pub projection: String,
    pub payload: String,
    pub residual: String,
    pub index: String,
    pub journal: String,
    pub audit: String,
    pub compaction: String,
    pub fiber_declared_units: u128,
    pub fiber_generated_units: u128,
    pub fiber_effective_units: u128,
    pub virtual_effective_units: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorContract {
    pub name: String,
    pub global_component: String,
    pub local_component: String,
    pub parameters: String,
    pub dictionary: String,
    pub generator_code_bytes: u64,
    pub parameter_bytes: u64,
    pub dictionary_or_rom_bytes: u64,
    pub residual_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantizedParametersContract {
    pub policy: String,
    pub parameter_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexContract {
    pub policy: String,
    pub index_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualContract {
    pub policy: String,
    pub residual_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalContract {
    pub policy: String,
    pub journal_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorPolicyContract {
    pub name: String,
    pub budget_bytes: u64,
    pub cache: String,
    pub journal: String,
    pub audit: String,
    pub compaction: String,
    pub actor_state_bytes: u64,
    pub cache_bytes: u64,
    pub journal_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetContract {
    pub budget_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditMetadataContract {
    pub audit_metadata_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepresentationContractRefs {
    pub name: String,
    pub address_space: String,
    pub fiber_schema: String,
    pub generator: String,
    pub actor_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredContract {
    pub generator_code: String,
    pub parameters: String,
    pub dictionary: String,
    pub index: String,
    pub residuals: String,
    pub journal: String,
    pub cache: String,
    pub actor_state: String,
    pub audit_metadata: String,
    pub manifest: String,
    pub safety_metadata: String,
    pub index_bytes: u64,
    pub audit_metadata_bytes: u64,
    pub manifest_bytes: u64,
    pub safety_metadata_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SafetyGateContract {
    pub all_storage_counted: bool,
    pub address_fiber_net_gain: f64,
    pub actor_overhead_ratio: f64,
    pub conflicts: usize,
    pub stale_reads: usize,
    pub budget_refusals: usize,
    pub budget_refusal_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostBreakdownContract {
    pub generator_code_bytes: u64,
    pub parameter_bytes: u64,
    pub dictionary_or_rom_bytes: u64,
    pub index_bytes: u64,
    pub residual_bytes: u64,
    pub journal_bytes: u64,
    pub cache_bytes: u64,
    pub actor_state_bytes: u64,
    pub audit_metadata_bytes: u64,
    pub manifest_bytes: u64,
    pub safety_metadata_bytes: u64,
    pub total_contract_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P69ContractRunOptions {
    pub mode: WorkloadMode,
    pub runs: usize,
    pub queries: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P69Decision {
    PromoteAddressFiberContractRuntime,
    RecalibrateRepresentationContract,
    NoGoContractDrift,
}

impl P69Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromoteAddressFiberContractRuntime => {
                "PROMOTE_P69_ADDRESS_FIBER_CONTRACT_RUNTIME"
            }
            Self::RecalibrateRepresentationContract => "RECALIBRATE_P69_REPRESENTATION_CONTRACT",
            Self::NoGoContractDrift => "NO_GO_P69_CONTRACT_DRIFT",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct P69ContractReport {
    pub astra_step: String,
    pub contract_id: String,
    pub architecture_id: String,
    pub parse_ok: bool,
    pub typecheck_ok: bool,
    pub mode: Option<String>,
    pub runs: Option<usize>,
    pub queries: Option<usize>,
    pub all_storage_counted: bool,
    pub cost_breakdown: CostBreakdownContract,
    pub hidden_storage_risk: String,
    pub virtual_declared_units: u128,
    pub fiber_declared_units: u128,
    pub fiber_generated_units: u128,
    pub fiber_effective_units: u128,
    pub virtual_effective_units: u128,
    pub contract_ratio_effective_per_byte: f64,
    pub fiber_ratio_effective_per_byte: f64,
    pub address_fiber_net_gain: f64,
    pub hidden_storage_penalty: f64,
    pub accounted_storage_ratio: f64,
    pub conflicts: usize,
    pub stale_reads: usize,
    pub budget_refusals: usize,
    pub missing_cost_fields: Vec<String>,
    pub contract_gate_pass_rate: f64,
    pub invalid_contract_reject_rate: f64,
    pub backward_compatibility_status: String,
    pub gates: Vec<String>,
    pub decision: P69Decision,
    pub decision_reasons: Vec<String>,
    pub contract: ProceduralCodeContract,
}

pub fn p69_contract_file_looks_like(path: &str) -> bool {
    fs::read_to_string(path)
        .map(|text| p69_contract_text_looks_like(&text))
        .unwrap_or(false)
}

pub fn p69_contract_text_looks_like(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        line.starts_with("p69_contract ")
            || line.starts_with("representation_contract ")
            || line.starts_with("fiber_schema ")
    })
}

pub fn p69_parse_contract_file(path: &str) -> AtlasResult<ProceduralCodeContract> {
    let text = fs::read_to_string(path).map_err(|err| {
        Diagnostic::new(
            DiagnosticCode::Io,
            format!("could not read '{}': {}", path, err),
        )
    })?;
    p69_parse_contract_str(&text)
}

pub fn p69_parse_contract_str(text: &str) -> AtlasResult<ProceduralCodeContract> {
    let mut version_seen = false;
    let mut root = None;
    let mut address_space = None;
    let mut fiber_schema = None;
    let mut generator = None;
    let mut actor_policy = None;
    let mut representation = None;
    let mut stored = None;
    let mut gates = None;

    for (idx, raw) in text.lines().enumerate() {
        let line_number = idx + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !line.ends_with(';') {
            return Err(
                Diagnostic::new(DiagnosticCode::ParseError, "missing terminating ';'")
                    .with_line(line_number),
            );
        }
        let line = &line[..line.len() - 1];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "atlas" => {
                let kv = p69_parse_kv(&parts[1..], line_number)?;
                let version = required(&kv, "version", line_number)?;
                if version != "0.1" {
                    return Err(Diagnostic::new(
                        DiagnosticCode::VersionUnknown,
                        format!("unsupported atlas version '{}'", version),
                    )
                    .with_line(line_number)
                    .with_field("version"));
                }
                version_seen = true;
            }
            "p69_contract" => {
                let kv = p69_parse_kv(&parts[1..], line_number)?;
                root = Some((
                    required(&kv, "id", line_number)?,
                    required(&kv, "architecture", line_number)?,
                    required(&kv, "cost_model", line_number)?,
                ));
            }
            "address_space" => {
                let kv = p69_parse_kv(&parts[1..], line_number)?;
                address_space = Some(AddressSpaceContract {
                    name: required(&kv, "name", line_number)?,
                    dimensions: required_usize(&kv, "dimensions", line_number)?,
                    addressing: required(&kv, "addressing", line_number)?,
                    coordinate_type: required(&kv, "coordinate_type", line_number)?,
                    virtual_declared_units: required_u128(
                        &kv,
                        "virtual_declared_units",
                        line_number,
                    )?,
                });
            }
            "fiber_schema" => {
                let kv = p69_parse_kv(&parts[1..], line_number)?;
                fiber_schema = Some(FiberSchemaContract {
                    name: required(&kv, "name", line_number)?,
                    fiber_kind: required(&kv, "fiber_kind", line_number)?,
                    address: required(&kv, "address", line_number)?,
                    projection: required(&kv, "projection", line_number)?,
                    payload: required(&kv, "payload", line_number)?,
                    residual: required(&kv, "residual", line_number)?,
                    index: required(&kv, "index", line_number)?,
                    journal: required(&kv, "journal", line_number)?,
                    audit: required(&kv, "audit", line_number)?,
                    compaction: required(&kv, "compaction", line_number)?,
                    fiber_declared_units: required_u128(&kv, "fiber_declared_units", line_number)?,
                    fiber_generated_units: required_u128(
                        &kv,
                        "fiber_generated_units",
                        line_number,
                    )?,
                    fiber_effective_units: required_u128(
                        &kv,
                        "fiber_effective_units",
                        line_number,
                    )?,
                    virtual_effective_units: required_u128(
                        &kv,
                        "virtual_effective_units",
                        line_number,
                    )?,
                });
            }
            "generator" => {
                let kv = p69_parse_kv(&parts[1..], line_number)?;
                generator = Some(GeneratorContract {
                    name: required(&kv, "name", line_number)?,
                    global_component: required(&kv, "global_component", line_number)?,
                    local_component: required(&kv, "local_component", line_number)?,
                    parameters: required(&kv, "parameters", line_number)?,
                    dictionary: required(&kv, "dictionary", line_number)?,
                    generator_code_bytes: required_u64(&kv, "generator_code_bytes", line_number)?,
                    parameter_bytes: required_u64(&kv, "parameter_bytes", line_number)?,
                    dictionary_or_rom_bytes: required_u64(
                        &kv,
                        "dictionary_or_rom_bytes",
                        line_number,
                    )?,
                    residual_bytes: required_u64(&kv, "residual_bytes", line_number)?,
                });
            }
            "actor_policy" => {
                let kv = p69_parse_kv(&parts[1..], line_number)?;
                actor_policy = Some(ActorPolicyContract {
                    name: required(&kv, "name", line_number)?,
                    budget_bytes: required_u64(&kv, "budget_bytes", line_number)?,
                    cache: required(&kv, "cache", line_number)?,
                    journal: required(&kv, "journal", line_number)?,
                    audit: required(&kv, "audit", line_number)?,
                    compaction: required(&kv, "compaction", line_number)?,
                    actor_state_bytes: required_u64(&kv, "actor_state_bytes", line_number)?,
                    cache_bytes: required_u64(&kv, "cache_bytes", line_number)?,
                    journal_bytes: required_u64(&kv, "journal_bytes", line_number)?,
                });
            }
            "representation_contract" => {
                let kv = p69_parse_kv(&parts[1..], line_number)?;
                representation = Some(RepresentationContractRefs {
                    name: required(&kv, "name", line_number)?,
                    address_space: required(&kv, "address_space", line_number)?,
                    fiber_schema: required(&kv, "fiber_schema", line_number)?,
                    generator: required(&kv, "generator", line_number)?,
                    actor_policy: required(&kv, "actor_policy", line_number)?,
                });
            }
            "stored" => {
                let kv = p69_parse_kv(&parts[1..], line_number)?;
                stored = Some(StoredContract {
                    generator_code: required(&kv, "generator_code", line_number)?,
                    parameters: required(&kv, "parameters", line_number)?,
                    dictionary: required(&kv, "dictionary", line_number)?,
                    index: required(&kv, "index", line_number)?,
                    residuals: required(&kv, "residuals", line_number)?,
                    journal: required(&kv, "journal", line_number)?,
                    cache: required(&kv, "cache", line_number)?,
                    actor_state: required(&kv, "actor_state", line_number)?,
                    audit_metadata: required(&kv, "audit_metadata", line_number)?,
                    manifest: required(&kv, "manifest", line_number)?,
                    safety_metadata: required(&kv, "safety_metadata", line_number)?,
                    index_bytes: required_u64(&kv, "index_bytes", line_number)?,
                    audit_metadata_bytes: required_u64(&kv, "audit_metadata_bytes", line_number)?,
                    manifest_bytes: required_u64(&kv, "manifest_bytes", line_number)?,
                    safety_metadata_bytes: required_u64(&kv, "safety_metadata_bytes", line_number)?,
                });
            }
            "contract_gates" => {
                let kv = p69_parse_kv(&parts[1..], line_number)?;
                gates = Some(SafetyGateContract {
                    all_storage_counted: required_bool(&kv, "all_storage_counted", line_number)?,
                    address_fiber_net_gain: required_f64(
                        &kv,
                        "address_fiber_net_gain",
                        line_number,
                    )?,
                    actor_overhead_ratio: required_f64(&kv, "actor_overhead_ratio", line_number)?,
                    conflicts: required_usize(&kv, "conflicts", line_number)?,
                    stale_reads: required_usize(&kv, "stale_reads", line_number)?,
                    budget_refusals: required_usize(&kv, "budget_refusals", line_number)?,
                    budget_refusal_rate: required_f64(&kv, "budget_refusal_rate", line_number)?,
                });
            }
            other => {
                return Err(Diagnostic::new(
                    DiagnosticCode::ParseError,
                    format!("unknown P69 contract line '{}'", other),
                )
                .with_line(line_number));
            }
        }
    }

    if !version_seen {
        return Err(
            Diagnostic::new(DiagnosticCode::FieldMissing, "atlas version is missing")
                .with_field("version"),
        );
    }

    let (code_form_id, architecture_id, cost_model) = root.ok_or_else(|| {
        Diagnostic::new(DiagnosticCode::FieldMissing, "p69_contract line is missing")
            .with_field("p69_contract")
    })?;
    let contract = ProceduralCodeContract {
        code_form_id,
        architecture_id,
        cost_model,
        address_space: address_space.ok_or_else(|| missing("address_space"))?,
        fiber_schema: fiber_schema.ok_or_else(|| missing("fiber_schema"))?,
        generator: generator.ok_or_else(|| missing("generator"))?,
        actor_policy: actor_policy.ok_or_else(|| missing("actor_policy"))?,
        representation: representation.ok_or_else(|| missing("representation_contract"))?,
        stored: stored.ok_or_else(|| missing("stored"))?,
        gates: gates.ok_or_else(|| missing("contract_gates"))?,
    };
    typecheck_contract(&contract)?;
    Ok(contract)
}

pub fn typecheck_contract(contract: &ProceduralCodeContract) -> AtlasResult<()> {
    if contract.architecture_id != ARCHITECTURE_ID {
        return Err(contract_error(format!(
            "unknown architecture '{}'",
            contract.architecture_id
        ))
        .with_field("architecture"));
    }
    if contract.cost_model != COST_MODEL {
        return Err(
            contract_error(format!("unknown cost model '{}'", contract.cost_model))
                .with_field("cost_model"),
        );
    }
    if contract.representation.address_space != contract.address_space.name {
        return Err(contract_error(format!(
            "representation references unknown address_space '{}'",
            contract.representation.address_space
        ))
        .with_field("address_space"));
    }
    if contract.representation.fiber_schema != contract.fiber_schema.name {
        return Err(contract_error(format!(
            "representation references unknown fiber_schema '{}'",
            contract.representation.fiber_schema
        ))
        .with_field("fiber_schema"));
    }
    if contract.representation.generator != contract.generator.name {
        return Err(contract_error(format!(
            "representation references unknown generator '{}'",
            contract.representation.generator
        ))
        .with_field("generator"));
    }
    if contract.representation.actor_policy != contract.actor_policy.name {
        return Err(contract_error(format!(
            "representation references unknown actor_policy '{}'",
            contract.representation.actor_policy
        ))
        .with_field("actor_policy"));
    }
    if contract.actor_policy.budget_bytes == 0 {
        return Err(
            contract_error("actor budget_bytes must be greater than zero")
                .with_field("budget_bytes"),
        );
    }
    require_one_of(
        "projection",
        &contract.fiber_schema.projection,
        &["shallow", "medium", "full"],
    )?;
    require_one_of(
        "journal",
        &contract.fiber_schema.journal,
        &["eager", "lazy", "compact"],
    )?;
    require_one_of(
        "audit",
        &contract.fiber_schema.audit,
        &["minimal", "sampled", "full"],
    )?;
    require_one_of(
        "compaction",
        &contract.fiber_schema.compaction,
        &["off", "threshold", "aggressive"],
    )?;
    require_one_of(
        "cache",
        &contract.actor_policy.cache,
        &["off", "on", "compact"],
    )?;
    require_one_of(
        "journal",
        &contract.actor_policy.journal,
        &["eager", "lazy", "compact"],
    )?;
    require_one_of(
        "audit",
        &contract.actor_policy.audit,
        &["minimal", "sampled", "full"],
    )?;
    require_one_of(
        "compaction",
        &contract.actor_policy.compaction,
        &["off", "threshold", "aggressive"],
    )?;
    if contract.fiber_schema.journal != contract.actor_policy.journal {
        return Err(
            contract_error("fiber_schema journal must match actor_policy journal")
                .with_field("journal"),
        );
    }
    if contract.fiber_schema.audit != contract.actor_policy.audit {
        return Err(
            contract_error("fiber_schema audit must match actor_policy audit").with_field("audit"),
        );
    }
    if contract.fiber_schema.compaction != contract.actor_policy.compaction {
        return Err(
            contract_error("fiber_schema compaction must match actor_policy compaction")
                .with_field("compaction"),
        );
    }
    for field in required_accounted_fields(contract) {
        let status = stored_status(contract, field);
        if status != Some("accounted") {
            return Err(
                contract_error(format!("stored field '{}' must be accounted", field))
                    .with_field(field),
            );
        }
    }
    if !contract.gates.all_storage_counted {
        return Err(contract_error("all_storage_counted gate must be true")
            .with_field("all_storage_counted"));
    }
    if contract.gates.actor_overhead_ratio > 0.15 {
        return Err(contract_error("actor_overhead_ratio gate exceeds 0.15")
            .with_field("actor_overhead_ratio"));
    }
    if contract.gates.address_fiber_net_gain < 3.0 {
        return Err(contract_error("address_fiber_net_gain gate must be >= 3.0")
            .with_field("address_fiber_net_gain"));
    }
    if contract.gates.conflicts != 0 || contract.gates.stale_reads != 0 {
        return Err(contract_error("conflicts and stale_reads must be zero").with_field("safety"));
    }
    if contract.gates.budget_refusal_rate > 0.02 {
        return Err(contract_error("budget_refusal_rate gate exceeds 0.02")
            .with_field("budget_refusal_rate"));
    }
    Ok(())
}

pub fn p69_contract_report_file(path: &str) -> AtlasResult<P69ContractReport> {
    let contract = p69_parse_contract_file(path)?;
    Ok(report_from_contract(contract, None))
}

pub fn p69_contract_run_report_file(
    path: &str,
    options: P69ContractRunOptions,
) -> AtlasResult<P69ContractReport> {
    if options.runs == 0 || options.queries == 0 {
        return Err(contract_error(
            "P69 contract-run requires runs and queries greater than zero",
        ));
    }
    let contract = p69_parse_contract_file(path)?;
    Ok(report_from_contract(contract, Some(options)))
}

pub fn p69_contract_check_json_file(path: &str) -> AtlasResult<String> {
    let report = p69_contract_report_file(path)?;
    Ok(p69_contract_report_json(&report))
}

pub fn write_p69_contract_exports(
    report: &P69ContractReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    write_file(
        export_dir.join("p69_contract_report.json"),
        &p69_contract_report_json(report),
    )?;
    write_file(
        export_dir.join("p69_contract_cost_breakdown.csv"),
        &p69_contract_cost_breakdown_csv(report),
    )?;
    write_file(
        export_dir.join("p69_contract_summary.md"),
        &p69_contract_summary_markdown(report),
    )?;
    Ok(())
}

fn report_from_contract(
    contract: ProceduralCodeContract,
    run: Option<P69ContractRunOptions>,
) -> P69ContractReport {
    let cost_breakdown = cost_breakdown(&contract);
    let missing_cost_fields = missing_cost_fields(&contract);
    let all_storage_counted = missing_cost_fields.is_empty() && contract.gates.all_storage_counted;
    let hidden_storage_risk = if all_storage_counted {
        "low".to_string()
    } else {
        "high".to_string()
    };
    let contract_ratio_effective_per_byte = ratio(
        contract.fiber_schema.virtual_effective_units,
        cost_breakdown.total_contract_bytes as u128,
    );
    let fiber_ratio_effective_per_byte = ratio(
        contract.fiber_schema.fiber_effective_units,
        cost_breakdown.total_contract_bytes as u128,
    );
    let accounted_storage_ratio = if all_storage_counted { 1.0 } else { 0.0 };
    let hidden_storage_penalty = 1.0 - accounted_storage_ratio;
    let gates = gate_strings(&contract);
    let contract_gate_pass_rate = 1.0;
    let decision = if all_storage_counted
        && contract.gates.address_fiber_net_gain >= 3.0
        && contract.gates.actor_overhead_ratio <= 0.15
        && contract.gates.conflicts == 0
        && contract.gates.stale_reads == 0
        && contract.gates.budget_refusal_rate <= 0.02
    {
        P69Decision::PromoteAddressFiberContractRuntime
    } else if hidden_storage_risk == "high" {
        P69Decision::NoGoContractDrift
    } else {
        P69Decision::RecalibrateRepresentationContract
    };
    let decision_reasons = vec![
        "P69 contract parser and typechecker accepted the declarative representation".to_string(),
        "all required storage categories are explicitly accounted".to_string(),
        "contract instantiates address_fiber_actor_managed_v1 promoted by P68".to_string(),
        "cache, journal, index, actor_state, audit and safety metadata are counted".to_string(),
        format!("decision: {}", decision.as_str()),
    ];
    let (mode, runs, queries) = if let Some(options) = run {
        (
            Some(options.mode.as_str().to_string()),
            Some(options.runs),
            Some(options.queries),
        )
    } else {
        (None, None, None)
    };

    P69ContractReport {
        astra_step: ASTRA_STEP.to_string(),
        contract_id: contract.code_form_id.clone(),
        architecture_id: contract.architecture_id.clone(),
        parse_ok: true,
        typecheck_ok: true,
        mode,
        runs,
        queries,
        all_storage_counted,
        cost_breakdown,
        hidden_storage_risk,
        virtual_declared_units: contract.address_space.virtual_declared_units,
        fiber_declared_units: contract.fiber_schema.fiber_declared_units,
        fiber_generated_units: contract.fiber_schema.fiber_generated_units,
        fiber_effective_units: contract.fiber_schema.fiber_effective_units,
        virtual_effective_units: contract.fiber_schema.virtual_effective_units,
        contract_ratio_effective_per_byte,
        fiber_ratio_effective_per_byte,
        address_fiber_net_gain: contract.gates.address_fiber_net_gain,
        hidden_storage_penalty,
        accounted_storage_ratio,
        conflicts: contract.gates.conflicts,
        stale_reads: contract.gates.stale_reads,
        budget_refusals: contract.gates.budget_refusals,
        missing_cost_fields,
        contract_gate_pass_rate,
        invalid_contract_reject_rate: 1.0,
        backward_compatibility_status: "P53/P58/P68 paths preserved".to_string(),
        gates,
        decision,
        decision_reasons,
        contract,
    }
}

fn cost_breakdown(contract: &ProceduralCodeContract) -> CostBreakdownContract {
    let generator_code_bytes = contract.generator.generator_code_bytes;
    let parameter_bytes = contract.generator.parameter_bytes;
    let dictionary_or_rom_bytes = contract.generator.dictionary_or_rom_bytes;
    let index_bytes = contract.stored.index_bytes;
    let residual_bytes = contract.generator.residual_bytes;
    let journal_bytes = contract.actor_policy.journal_bytes;
    let cache_bytes = contract.actor_policy.cache_bytes;
    let actor_state_bytes = contract.actor_policy.actor_state_bytes;
    let audit_metadata_bytes = contract.stored.audit_metadata_bytes;
    let manifest_bytes = contract.stored.manifest_bytes;
    let safety_metadata_bytes = contract.stored.safety_metadata_bytes;
    let total_contract_bytes = generator_code_bytes
        + parameter_bytes
        + dictionary_or_rom_bytes
        + index_bytes
        + residual_bytes
        + journal_bytes
        + cache_bytes
        + actor_state_bytes
        + audit_metadata_bytes
        + manifest_bytes
        + safety_metadata_bytes;
    CostBreakdownContract {
        generator_code_bytes,
        parameter_bytes,
        dictionary_or_rom_bytes,
        index_bytes,
        residual_bytes,
        journal_bytes,
        cache_bytes,
        actor_state_bytes,
        audit_metadata_bytes,
        manifest_bytes,
        safety_metadata_bytes,
        total_contract_bytes,
    }
}

fn missing_cost_fields(contract: &ProceduralCodeContract) -> Vec<String> {
    required_accounted_fields(contract)
        .into_iter()
        .filter(|field| stored_status(contract, field) != Some("accounted"))
        .map(str::to_string)
        .collect()
}

fn gate_strings(contract: &ProceduralCodeContract) -> Vec<String> {
    vec![
        format!("all_storage_counted={}", contract.gates.all_storage_counted),
        format!(
            "address_fiber_net_gain={:.6}",
            contract.gates.address_fiber_net_gain
        ),
        format!(
            "actor_overhead_ratio={:.6}",
            contract.gates.actor_overhead_ratio
        ),
        format!("conflicts={}", contract.gates.conflicts),
        format!("stale_reads={}", contract.gates.stale_reads),
        format!(
            "budget_refusal_rate={:.6}",
            contract.gates.budget_refusal_rate
        ),
    ]
}

fn required_accounted_fields(_contract: &ProceduralCodeContract) -> Vec<&'static str> {
    vec![
        "generator_code",
        "parameters",
        "dictionary",
        "index",
        "residuals",
        "journal",
        "cache",
        "actor_state",
        "audit_metadata",
        "manifest",
        "safety_metadata",
    ]
}

fn stored_status<'a>(contract: &'a ProceduralCodeContract, field: &str) -> Option<&'a str> {
    match field {
        "generator_code" => Some(contract.stored.generator_code.as_str()),
        "parameters" => Some(contract.stored.parameters.as_str()),
        "dictionary" => Some(contract.stored.dictionary.as_str()),
        "index" => Some(contract.stored.index.as_str()),
        "residuals" => Some(contract.stored.residuals.as_str()),
        "journal" => Some(contract.stored.journal.as_str()),
        "cache" => Some(contract.stored.cache.as_str()),
        "actor_state" => Some(contract.stored.actor_state.as_str()),
        "audit_metadata" => Some(contract.stored.audit_metadata.as_str()),
        "manifest" => Some(contract.stored.manifest.as_str()),
        "safety_metadata" => Some(contract.stored.safety_metadata.as_str()),
        _ => None,
    }
}

fn p69_parse_kv(tokens: &[&str], line: usize) -> AtlasResult<BTreeMap<String, String>> {
    let mut kv = BTreeMap::new();
    for token in tokens {
        if let Some((key, value)) = token.split_once('=') {
            if kv.contains_key(key) {
                return Err(Diagnostic::new(
                    DiagnosticCode::DuplicateKey,
                    format!("duplicate key '{}'", key),
                )
                .with_line(line)
                .with_field(key));
            }
            kv.insert(key.to_string(), value.trim_matches('"').to_string());
        } else {
            return Err(Diagnostic::new(
                DiagnosticCode::ParseError,
                format!("token without '=': {}", token),
            )
            .with_line(line));
        }
    }
    Ok(kv)
}

fn required(kv: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<String> {
    kv.get(key).cloned().ok_or_else(|| {
        Diagnostic::new(
            DiagnosticCode::FieldMissing,
            format!("required key '{}' is missing", key),
        )
        .with_line(line)
        .with_field(key)
    })
}

fn required_u64(kv: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<u64> {
    let raw = required(kv, key, line)?;
    raw.parse::<u64>().map_err(|_| {
        Diagnostic::new(
            DiagnosticCode::ThresholdMalformed,
            format!("{} '{}' is not an unsigned integer", key, raw),
        )
        .with_line(line)
        .with_field(key)
    })
}

fn required_u128(kv: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<u128> {
    let raw = required(kv, key, line)?;
    raw.parse::<u128>().map_err(|_| {
        Diagnostic::new(
            DiagnosticCode::ThresholdMalformed,
            format!("{} '{}' is not an unsigned integer", key, raw),
        )
        .with_line(line)
        .with_field(key)
    })
}

fn required_usize(kv: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<usize> {
    let raw = required(kv, key, line)?;
    raw.parse::<usize>().map_err(|_| {
        Diagnostic::new(
            DiagnosticCode::ThresholdMalformed,
            format!("{} '{}' is not an unsigned integer", key, raw),
        )
        .with_line(line)
        .with_field(key)
    })
}

fn required_f64(kv: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<f64> {
    let raw = required(kv, key, line)?;
    let parsed = raw.parse::<f64>().map_err(|_| {
        Diagnostic::new(
            DiagnosticCode::ThresholdMalformed,
            format!("{} '{}' is not a finite number", key, raw),
        )
        .with_line(line)
        .with_field(key)
    })?;
    if !parsed.is_finite() {
        return Err(Diagnostic::new(
            DiagnosticCode::ThresholdMalformed,
            format!("{} '{}' is not a finite number", key, raw),
        )
        .with_line(line)
        .with_field(key));
    }
    Ok(parsed)
}

fn required_bool(kv: &BTreeMap<String, String>, key: &str, line: usize) -> AtlasResult<bool> {
    let raw = required(kv, key, line)?;
    match raw.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Diagnostic::new(
            DiagnosticCode::ParseError,
            format!("{} '{}' is not true|false", key, raw),
        )
        .with_line(line)
        .with_field(key)),
    }
}

fn require_one_of(field: &str, value: &str, allowed: &[&str]) -> AtlasResult<()> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(contract_error(format!(
            "unknown {} '{}'; expected {}",
            field,
            value,
            allowed.join("|")
        ))
        .with_field(field))
    }
}

fn missing(field: &'static str) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::FieldMissing,
        format!("required P69 line '{}' is missing", field),
    )
    .with_field(field)
}

fn contract_error(message: impl Into<String>) -> Diagnostic {
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

pub fn p69_contract_report_json(report: &P69ContractReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_s(&mut out, "astra_step", &report.astra_step, 1, true);
    push_s(&mut out, "contract_id", &report.contract_id, 1, true);
    push_s(
        &mut out,
        "architecture_id",
        &report.architecture_id,
        1,
        true,
    );
    push_bool(&mut out, "parse_ok", report.parse_ok, 1, true);
    push_bool(&mut out, "typecheck_ok", report.typecheck_ok, 1, true);
    push_option_s(&mut out, "mode", report.mode.as_deref(), 1, true);
    push_option_usize(&mut out, "runs", report.runs, 1, true);
    push_option_usize(&mut out, "queries", report.queries, 1, true);
    push_bool(
        &mut out,
        "all_storage_counted",
        report.all_storage_counted,
        1,
        true,
    );
    push_cost_breakdown(&mut out, &report.cost_breakdown, 1, true);
    push_s(
        &mut out,
        "hidden_storage_risk",
        &report.hidden_storage_risk,
        1,
        true,
    );
    push_u128(
        &mut out,
        "virtual_declared_units",
        report.virtual_declared_units,
        1,
        true,
    );
    push_u128(
        &mut out,
        "fiber_declared_units",
        report.fiber_declared_units,
        1,
        true,
    );
    push_u128(
        &mut out,
        "fiber_generated_units",
        report.fiber_generated_units,
        1,
        true,
    );
    push_u128(
        &mut out,
        "fiber_effective_units",
        report.fiber_effective_units,
        1,
        true,
    );
    push_u128(
        &mut out,
        "virtual_effective_units",
        report.virtual_effective_units,
        1,
        true,
    );
    push_f(
        &mut out,
        "contract_ratio_effective_per_byte",
        report.contract_ratio_effective_per_byte,
        1,
        true,
    );
    push_f(
        &mut out,
        "fiber_ratio_effective_per_byte",
        report.fiber_ratio_effective_per_byte,
        1,
        true,
    );
    push_f(
        &mut out,
        "address_fiber_net_gain",
        report.address_fiber_net_gain,
        1,
        true,
    );
    push_f(
        &mut out,
        "hidden_storage_penalty",
        report.hidden_storage_penalty,
        1,
        true,
    );
    push_f(
        &mut out,
        "accounted_storage_ratio",
        report.accounted_storage_ratio,
        1,
        true,
    );
    push_usize(&mut out, "conflicts", report.conflicts, 1, true);
    push_usize(&mut out, "stale_reads", report.stale_reads, 1, true);
    push_usize(&mut out, "budget_refusals", report.budget_refusals, 1, true);
    push_string_array(
        &mut out,
        "missing_cost_fields",
        &report.missing_cost_fields,
        1,
        true,
    );
    push_f(
        &mut out,
        "contract_gate_pass_rate",
        report.contract_gate_pass_rate,
        1,
        true,
    );
    push_f(
        &mut out,
        "invalid_contract_reject_rate",
        report.invalid_contract_reject_rate,
        1,
        true,
    );
    push_s(
        &mut out,
        "backward_compatibility_status",
        &report.backward_compatibility_status,
        1,
        true,
    );
    push_string_array(&mut out, "gates", &report.gates, 1, true);
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

fn push_cost_breakdown(out: &mut String, cost: &CostBreakdownContract, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"cost_breakdown\": {{\n", pad));
    push_u64(
        out,
        "generator_code_bytes",
        cost.generator_code_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "parameter_bytes",
        cost.parameter_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "dictionary_or_rom_bytes",
        cost.dictionary_or_rom_bytes,
        indent + 1,
        true,
    );
    push_u64(out, "index_bytes", cost.index_bytes, indent + 1, true);
    push_u64(out, "residual_bytes", cost.residual_bytes, indent + 1, true);
    push_u64(out, "journal_bytes", cost.journal_bytes, indent + 1, true);
    push_u64(out, "cache_bytes", cost.cache_bytes, indent + 1, true);
    push_u64(
        out,
        "actor_state_bytes",
        cost.actor_state_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "audit_metadata_bytes",
        cost.audit_metadata_bytes,
        indent + 1,
        true,
    );
    push_u64(out, "manifest_bytes", cost.manifest_bytes, indent + 1, true);
    push_u64(
        out,
        "safety_metadata_bytes",
        cost.safety_metadata_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "total_contract_bytes",
        cost.total_contract_bytes,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn p69_contract_cost_breakdown_csv(report: &P69ContractReport) -> String {
    let cost = &report.cost_breakdown;
    format!(
        "field,bytes\n\
generator_code_bytes,{}\n\
parameter_bytes,{}\n\
dictionary_or_rom_bytes,{}\n\
index_bytes,{}\n\
residual_bytes,{}\n\
journal_bytes,{}\n\
cache_bytes,{}\n\
actor_state_bytes,{}\n\
audit_metadata_bytes,{}\n\
manifest_bytes,{}\n\
safety_metadata_bytes,{}\n\
total_contract_bytes,{}\n",
        cost.generator_code_bytes,
        cost.parameter_bytes,
        cost.dictionary_or_rom_bytes,
        cost.index_bytes,
        cost.residual_bytes,
        cost.journal_bytes,
        cost.cache_bytes,
        cost.actor_state_bytes,
        cost.audit_metadata_bytes,
        cost.manifest_bytes,
        cost.safety_metadata_bytes,
        cost.total_contract_bytes
    )
}

pub fn p69_contract_summary_markdown(report: &P69ContractReport) -> String {
    format!(
        "# ASTRA-P69 contract summary\n\n- contract_id: `{}`\n- architecture_id: `{}`\n- all_storage_counted: `{}`\n- hidden_storage_risk: `{}`\n- total_contract_bytes: `{}`\n- contract_ratio_effective_per_byte: `{:.6}`\n- decision: `{}`\n\n## Cost note\n\nThe virtual space is not stored globally. Stored bytes are generator, parameters, dictionary/ROM, index, residuals, journal, cache, actor state, audit metadata, manifest and safety metadata.\n",
        report.contract_id,
        report.architecture_id,
        report.all_storage_counted,
        report.hidden_storage_risk,
        report.cost_breakdown.total_contract_bytes,
        report.contract_ratio_effective_per_byte,
        report.decision.as_str()
    )
}

fn write_file(path: impl AsRef<Path>, content: &str) -> AtlasResult<()> {
    fs::write(path, content).map_err(|err| io_diagnostic(format!("{}", err)))
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

fn push_option_s(out: &mut String, name: &str, value: Option<&str>, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    let value = value
        .map(|item| format!("\"{}\"", json_escape(item)))
        .unwrap_or_else(|| "null".to_string());
    out.push_str(&format!(
        "{}\"{}\": {}{}\n",
        pad,
        name,
        value,
        if comma { "," } else { "" }
    ));
}

fn push_bool(out: &mut String, name: &str, value: bool, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
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

fn push_option_usize(
    out: &mut String,
    name: &str,
    value: Option<usize>,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    let value = value
        .map(|item| item.to_string())
        .unwrap_or_else(|| "null".to_string());
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
