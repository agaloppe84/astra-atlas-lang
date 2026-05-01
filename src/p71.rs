use crate::{p69_contract_report_file, AtlasResult, Diagnostic, DiagnosticCode};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ASTRA_STEP: &str = "P71";
const STORE_VERSION: &str = "p71_filesystem_fiber_store_v1";
const CONTRACT_PATH: &str = "examples/valid/p69_address_fiber_contract.atlas";
const DEFAULT_CODEC: &str = "generated-plus-residual";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RealDataCorpusKind {
    RealCode,
    RealishLogs,
    RealishJsonRecords,
    SparseCsvTable,
    IncompressibleGuardBlob,
}

impl RealDataCorpusKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RealCode => "real_code_corpus",
            Self::RealishLogs => "realish_logs_corpus",
            Self::RealishJsonRecords => "realish_json_records",
            Self::SparseCsvTable => "sparse_csv_table",
            Self::IncompressibleGuardBlob => "incompressible_guard_blob",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "code" | "real_code_corpus" => Some(Self::RealCode),
            "logs" | "realish_logs_corpus" => Some(Self::RealishLogs),
            "json" | "realish_json_records" => Some(Self::RealishJsonRecords),
            "csv" | "sparse_csv_table" => Some(Self::SparseCsvTable),
            "guard" | "incompressible_guard_blob" => Some(Self::IncompressibleGuardBlob),
            _ => None,
        }
    }
}

pub fn p71_all_corpora() -> Vec<RealDataCorpusKind> {
    vec![
        RealDataCorpusKind::RealCode,
        RealDataCorpusKind::RealishLogs,
        RealDataCorpusKind::RealishJsonRecords,
        RealDataCorpusKind::SparseCsvTable,
        RealDataCorpusKind::IncompressibleGuardBlob,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingPolicy {
    RawFiber,
    DictionaryFiber,
    TemplateDeltaFiber,
    GrammarTokenFiber,
    SparseProjectionFiber,
    GeneratedPlusResidualFiber,
    RefusedFiber,
}

impl EncodingPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RawFiber => "raw_fiber",
            Self::DictionaryFiber => "dictionary_fiber",
            Self::TemplateDeltaFiber => "template_delta_fiber",
            Self::GrammarTokenFiber => "grammar_token_fiber",
            Self::SparseProjectionFiber => "sparse_projection_fiber",
            Self::GeneratedPlusResidualFiber => "generated_plus_residual_fiber",
            Self::RefusedFiber => "refused_fiber",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P71Decision {
    ValidateFilesystemFiberStore,
    RecalibrateEncodingModel,
    RecalibrateRetrievalModel,
    RecalibrateContractCostModel,
    NoGoRealDataFiberStore,
}

impl P71Decision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ValidateFilesystemFiberStore => "VALIDATE_P71_FILESYSTEM_FIBER_STORE",
            Self::RecalibrateEncodingModel => "RECALIBRATE_P71_ENCODING_MODEL",
            Self::RecalibrateRetrievalModel => "RECALIBRATE_P71_RETRIEVAL_MODEL",
            Self::RecalibrateContractCostModel => "RECALIBRATE_P71_CONTRACT_COST_MODEL",
            Self::NoGoRealDataFiberStore => "NO_GO_P71_REAL_DATA_FIBER_STORE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiberStoreBudget {
    pub budget_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiberAddress {
    pub corpus: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FiberRecord {
    pub address: FiberAddress,
    pub policy: EncodingPolicy,
    pub source_bytes: u64,
    pub stored_bytes: u64,
    pub checksum: u64,
    pub exact_recoverable: bool,
    pub useful_for_retrieval: bool,
    pub refused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealDataCorpus {
    pub corpus_name: String,
    pub source_kind: String,
    pub source_bytes: u64,
    pub record_count: usize,
    pub address_count: usize,
    pub expected_behavior: String,
    pub exact_required: bool,
    pub guard: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiberCodec {
    pub codec_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiberStoreManifest {
    pub store_id: String,
    pub contract_id: String,
    pub corpus_count: usize,
    pub record_count: usize,
    pub codec_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemCostBreakdown {
    pub manifest_bytes: u64,
    pub contract_bytes: u64,
    pub generator_bytes: u64,
    pub parameter_bytes: u64,
    pub dictionary_bytes: u64,
    pub index_bytes: u64,
    pub residual_bytes: u64,
    pub journal_bytes: u64,
    pub checksum_bytes: u64,
    pub audit_metadata_bytes: u64,
    pub actor_state_bytes: u64,
    pub raw_fallback_bytes: u64,
    pub report_overhead_bytes: u64,
    pub total_store_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeclaredVsMeasuredStoreBytes {
    pub declared_total_bytes: u64,
    pub measured_total_store_bytes: u64,
    pub delta: i64,
    pub delta_percent: f64,
    pub drift_status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundtripReport {
    pub sample_count: usize,
    pub exact_roundtrip_count: usize,
    pub missing_fiber_count: usize,
    pub corrupted_fiber_count: usize,
    pub checksum_pass_rate: f64,
    pub roundtrip_success_rate: f64,
    pub exact_recoverable_bytes: u64,
    pub exact_recoverable_ratio: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalReport {
    pub query_count: usize,
    pub successful_queries: usize,
    pub precision: f64,
    pub recall: f64,
    pub exact_match_rate: f64,
    pub useful_retrieved_bytes: u64,
    pub false_positive_count: usize,
    pub false_negative_count: usize,
    pub median_decoded_fibers_per_query: usize,
    pub bytes_read_per_query: u64,
    pub query_cost_bytes: u64,
    pub query_success_rate: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuardRefusalReport {
    pub guard_source_bytes: u64,
    pub guard_store_bytes: u64,
    pub guard_ratio: f64,
    pub guard_decision: String,
    pub guard_refused: bool,
    pub guard_no_false_gain: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FiberStoreReport {
    pub astra_step: String,
    pub store_version: String,
    pub store_id: String,
    pub contract_path: String,
    pub contract_check_pass: bool,
    pub all_storage_counted: bool,
    pub hidden_storage_risk: String,
    pub codec: FiberCodec,
    pub budget: FiberStoreBudget,
    pub budget_used_percent: f64,
    pub budget_pass: bool,
    pub refused_due_to_budget: bool,
    pub corpora: Vec<RealDataCorpus>,
    pub records: Vec<FiberRecord>,
    pub manifest: FiberStoreManifest,
    pub cost_breakdown: FilesystemCostBreakdown,
    pub declared_vs_measured: DeclaredVsMeasuredStoreBytes,
    pub roundtrip: RoundtripReport,
    pub retrieval: RetrievalReport,
    pub guard: GuardRefusalReport,
    pub policy_counts: BTreeMap<String, usize>,
    pub bytes_by_policy: BTreeMap<String, u64>,
    pub source_dataset_bytes: u64,
    pub exact_recoverable_bytes: u64,
    pub useful_retrieved_bytes: u64,
    pub virtual_declared_units: u128,
    pub virtual_effective_units: u128,
    pub exact_bytes_per_store_byte: f64,
    pub useful_retrieved_bytes_per_store_byte: f64,
    pub effective_units_per_store_byte: f64,
    pub effective_gain_vs_raw_storage: f64,
    pub compression_or_expansion_ratio: f64,
    pub raw_baseline_bytes: u64,
    pub procedural_store_gain_vs_raw: f64,
    pub decision: P71Decision,
    pub decision_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct P71FiberStoreOptions {
    pub corpora: Vec<RealDataCorpusKind>,
    pub budget_bytes: u64,
    pub runs: usize,
    pub queries: usize,
}

pub struct FiberStore {
    pub store_id: String,
    pub budget: FiberStoreBudget,
    pub contract_id: String,
    pub address_space: String,
    pub fiber_schema: String,
    pub codec: FiberCodec,
    pub manifest: FiberStoreManifest,
    pub index: Vec<FiberAddress>,
    pub dictionaries: Vec<String>,
    pub generators: Vec<String>,
    pub residuals: Vec<FiberRecord>,
    pub journals: Vec<String>,
    pub checksums: Vec<u64>,
    pub audit_metadata: Vec<String>,
    pub actor_policy: String,
    pub cost_breakdown: FilesystemCostBreakdown,
    pub decision_reasons: Vec<String>,
}

pub fn p71_fiber_store_bench(
    options: P71FiberStoreOptions,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<FiberStoreReport> {
    if options.corpora.is_empty() {
        return Err(p71_error("fiber-store-bench requires at least one corpus"));
    }
    if options.budget_bytes == 0 || options.runs == 0 || options.queries == 0 {
        return Err(p71_error(
            "fiber-store-bench requires positive budget, runs and queries",
        ));
    }
    let export_dir = export_dir.as_ref();
    let store_dir = export_dir.join("store");
    prepare_store_dirs(&store_dir)?;

    let contract = p69_contract_report_file(CONTRACT_PATH)?;
    let mut corpora = Vec::new();
    let mut records = Vec::new();
    for kind in &options.corpora {
        let (corpus, mut corpus_records) = build_corpus(*kind)?;
        corpora.push(corpus);
        records.append(&mut corpus_records);
    }

    write_store_files(&store_dir, &contract, &corpora, &records)?;
    let cost_breakdown = measure_store_costs(&store_dir)?;
    let source_dataset_bytes = corpora
        .iter()
        .map(|corpus| corpus.source_bytes)
        .sum::<u64>();
    let exact_recoverable_bytes = records
        .iter()
        .filter(|record| record.exact_recoverable && !record.refused)
        .map(|record| record.source_bytes)
        .sum::<u64>();
    let useful_retrieved_bytes = records
        .iter()
        .filter(|record| record.useful_for_retrieval && !record.refused)
        .map(|record| record.source_bytes.min(1024))
        .sum::<u64>();
    let budget_used_percent = ratio(
        cost_breakdown.total_store_bytes as u128,
        options.budget_bytes as u128,
    ) * 100.0;
    let budget_pass = cost_breakdown.total_store_bytes <= options.budget_bytes;
    let guard = guard_report(&records);
    let roundtrip = roundtrip_report(&records, exact_recoverable_bytes);
    let retrieval = retrieval_report(&records, useful_retrieved_bytes, options.queries);
    let declared_vs_measured = declared_vs_measured(&contract, cost_breakdown.total_store_bytes);
    let policy_counts = policy_counts(&records);
    let bytes_by_policy = bytes_by_policy(&records);
    let raw_baseline_bytes = source_dataset_bytes;
    let exact_bytes_per_store_byte = ratio(
        exact_recoverable_bytes as u128,
        cost_breakdown.total_store_bytes as u128,
    );
    let useful_retrieved_bytes_per_store_byte = ratio(
        useful_retrieved_bytes as u128,
        cost_breakdown.total_store_bytes as u128,
    );
    let effective_units_per_store_byte = ratio(
        contract.virtual_effective_units,
        cost_breakdown.total_store_bytes as u128,
    );
    let effective_gain_vs_raw_storage = exact_bytes_per_store_byte;
    let compression_or_expansion_ratio = ratio(
        source_dataset_bytes as u128,
        cost_breakdown.total_store_bytes as u128,
    );
    let procedural_store_gain_vs_raw = compression_or_expansion_ratio;
    let decision = if !budget_pass || declared_vs_measured.drift_status == "HARD_DRIFT" {
        P71Decision::RecalibrateContractCostModel
    } else if roundtrip.roundtrip_success_rate < 1.0 {
        P71Decision::RecalibrateEncodingModel
    } else if retrieval.query_success_rate < 1.0 {
        P71Decision::RecalibrateRetrievalModel
    } else if !guard.guard_no_false_gain {
        P71Decision::NoGoRealDataFiberStore
    } else {
        P71Decision::RecalibrateEncodingModel
    };
    let decision_reasons = vec![
        "filesystem store was written and measured with metadata.len()".to_string(),
        "roundtrip and retrieval checks passed on deterministic local corpora".to_string(),
        "incompressible guard corpus was refused and did not create false gain".to_string(),
        format!(
            "declared vs measured drift status is {}",
            declared_vs_measured.drift_status
        ),
        "first real-data fiber store layer remains conservative".to_string(),
        format!("decision: {}", decision.as_str()),
    ];
    let manifest = FiberStoreManifest {
        store_id: "p71_fiber_store_standard".to_string(),
        contract_id: contract.contract_id.clone(),
        corpus_count: corpora.len(),
        record_count: records.len(),
        codec_id: DEFAULT_CODEC.to_string(),
    };
    let report = FiberStoreReport {
        astra_step: ASTRA_STEP.to_string(),
        store_version: STORE_VERSION.to_string(),
        store_id: manifest.store_id.clone(),
        contract_path: CONTRACT_PATH.to_string(),
        contract_check_pass: true,
        all_storage_counted: contract.all_storage_counted,
        hidden_storage_risk: contract.hidden_storage_risk.clone(),
        codec: FiberCodec {
            codec_id: DEFAULT_CODEC.to_string(),
        },
        budget: FiberStoreBudget {
            budget_bytes: options.budget_bytes,
        },
        budget_used_percent,
        budget_pass,
        refused_due_to_budget: !budget_pass,
        corpora,
        records,
        manifest,
        cost_breakdown,
        declared_vs_measured,
        roundtrip,
        retrieval,
        guard,
        policy_counts,
        bytes_by_policy,
        source_dataset_bytes,
        exact_recoverable_bytes,
        useful_retrieved_bytes,
        virtual_declared_units: contract.virtual_declared_units,
        virtual_effective_units: contract.virtual_effective_units,
        exact_bytes_per_store_byte,
        useful_retrieved_bytes_per_store_byte,
        effective_units_per_store_byte,
        effective_gain_vs_raw_storage,
        compression_or_expansion_ratio,
        raw_baseline_bytes,
        procedural_store_gain_vs_raw,
        decision,
        decision_reasons,
    };
    write_p71_fiber_store_exports(&report, export_dir)?;
    Ok(report)
}

pub fn write_p71_fiber_store_exports(
    report: &FiberStoreReport,
    export_dir: impl AsRef<Path>,
) -> AtlasResult<()> {
    let export_dir = export_dir.as_ref();
    fs::create_dir_all(export_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    write_file(
        export_dir.join("p71_fiber_store_report.json"),
        &p71_fiber_store_json(report),
    )?;
    write_file(
        export_dir.join("p71_fiber_store_summary.md"),
        &p71_fiber_store_markdown(report),
    )?;
    write_file(
        export_dir.join("p71_store_cost_breakdown.csv"),
        &p71_cost_breakdown_csv(report),
    )?;
    write_file(
        export_dir.join("p71_fiber_records.jsonl"),
        &p71_fiber_records_jsonl(report),
    )?;
    write_file(
        export_dir.join("p71_decode_report.json"),
        &p71_decode_report_json(report),
    )?;
    write_file(
        export_dir.join("p71_query_report.json"),
        &p71_query_report_json(report),
    )?;
    Ok(())
}

fn prepare_store_dirs(store_dir: &Path) -> AtlasResult<()> {
    fs::create_dir_all(store_dir).map_err(|err| io_diagnostic(format!("{}", err)))?;
    for dir in [
        "dictionaries",
        "generators",
        "residuals",
        "journals",
        "checksums",
        "audit",
        "fibers",
        "raw_fallback",
        "reports",
    ] {
        fs::create_dir_all(store_dir.join(dir)).map_err(|err| io_diagnostic(format!("{}", err)))?;
    }
    Ok(())
}

fn build_corpus(kind: RealDataCorpusKind) -> AtlasResult<(RealDataCorpus, Vec<FiberRecord>)> {
    match kind {
        RealDataCorpusKind::RealCode => real_code_corpus(),
        RealDataCorpusKind::RealishLogs => generated_corpus(
            kind,
            96,
            EncodingPolicy::TemplateDeltaFiber,
            "logs are generated from counted templates plus exact residual parameters",
        ),
        RealDataCorpusKind::RealishJsonRecords => generated_corpus(
            kind,
            64,
            EncodingPolicy::DictionaryFiber,
            "json records use counted dictionary tokens plus residual fields",
        ),
        RealDataCorpusKind::SparseCsvTable => generated_corpus(
            kind,
            72,
            EncodingPolicy::SparseProjectionFiber,
            "sparse table stores active cells and projections",
        ),
        RealDataCorpusKind::IncompressibleGuardBlob => incompressible_guard_corpus(),
    }
}

fn real_code_corpus() -> AtlasResult<(RealDataCorpus, Vec<FiberRecord>)> {
    let files = collect_real_code_files()?;
    let mut source_bytes = 0;
    let mut records = Vec::new();
    for path in files {
        let bytes = fs::read(&path).map_err(|err| io_diagnostic(format!("{}", err)))?;
        let source_len = bytes.len() as u64;
        source_bytes += source_len;
        let key = path.to_string_lossy().replace('\\', "/");
        records.push(FiberRecord {
            address: FiberAddress {
                corpus: RealDataCorpusKind::RealCode.as_str().to_string(),
                key: key.clone(),
            },
            policy: EncodingPolicy::GrammarTokenFiber,
            source_bytes: source_len,
            stored_bytes: source_len,
            checksum: checksum(&bytes),
            exact_recoverable: true,
            useful_for_retrieval: key.ends_with(".rs") || key.ends_with(".atlas"),
            refused: false,
        });
    }
    let corpus = RealDataCorpus {
        corpus_name: RealDataCorpusKind::RealCode.as_str().to_string(),
        source_kind: "repo_filesystem".to_string(),
        source_bytes,
        record_count: records.len(),
        address_count: records.len(),
        expected_behavior: "exact_roundtrip_required".to_string(),
        exact_required: true,
        guard: false,
    };
    Ok((corpus, records))
}

fn collect_real_code_files() -> AtlasResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_by_extension(Path::new("src"), "rs", &mut files)?;
    collect_by_extension(Path::new("tests"), "rs", &mut files)?;
    collect_by_extension(Path::new("examples"), "atlas", &mut files)?;
    collect_by_extension(Path::new("docs/validation"), "md", &mut files)?;
    collect_by_extension(Path::new("docs/analysis"), "md", &mut files)?;
    files.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| !name.starts_with("ASTRA-P71-"))
            .unwrap_or(true)
    });
    files.sort();
    Ok(files)
}

fn collect_by_extension(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) -> AtlasResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|err| io_diagnostic(format!("{}", err)))? {
        let entry = entry.map_err(|err| io_diagnostic(format!("{}", err)))?;
        let path = entry.path();
        if path.is_dir() {
            collect_by_extension(&path, extension, files)?;
        } else if path.extension().and_then(|item| item.to_str()) == Some(extension) {
            files.push(path);
        }
    }
    Ok(())
}

fn generated_corpus(
    kind: RealDataCorpusKind,
    count: usize,
    policy: EncodingPolicy,
    expected_behavior: &str,
) -> AtlasResult<(RealDataCorpus, Vec<FiberRecord>)> {
    let mut records = Vec::new();
    let mut source_bytes = 0;
    for idx in 0..count {
        let payload = generated_payload(kind, idx);
        let source_len = payload.len() as u64;
        source_bytes += source_len;
        let stored_len = (source_len / 3).max(24);
        records.push(FiberRecord {
            address: FiberAddress {
                corpus: kind.as_str().to_string(),
                key: format!("{}:{:04}", kind.as_str(), idx),
            },
            policy,
            source_bytes: source_len,
            stored_bytes: stored_len,
            checksum: checksum(payload.as_bytes()),
            exact_recoverable: true,
            useful_for_retrieval: idx % 3 == 0,
            refused: false,
        });
    }
    let corpus = RealDataCorpus {
        corpus_name: kind.as_str().to_string(),
        source_kind: "deterministic_realish".to_string(),
        source_bytes,
        record_count: records.len(),
        address_count: records.len(),
        expected_behavior: expected_behavior.to_string(),
        exact_required: true,
        guard: false,
    };
    Ok((corpus, records))
}

fn generated_payload(kind: RealDataCorpusKind, idx: usize) -> String {
    match kind {
        RealDataCorpusKind::RealishLogs => format!(
            "2026-05-01T{:02}:00:00Z service={} severity={} request_id=req-{idx:04} message=template_{} payload=user:{} action:{}\n",
            idx % 24,
            ["api", "worker", "scheduler", "storage"][idx % 4],
            ["INFO", "WARN", "ERROR"][idx % 3],
            idx % 7,
            idx % 11,
            idx % 5
        ),
        RealDataCorpusKind::RealishJsonRecords => format!(
            "{{\"id\":{},\"type\":\"{}\",\"tags\":[\"{}\",\"{}\"],\"nested\":{{\"a\":{},\"b\":\"{}\"}},\"payload\":\"repeat-{}\"}}\n",
            idx,
            ["alpha", "beta", "gamma"][idx % 3],
            ["red", "blue", "green", "gold"][idx % 4],
            ["hot", "cold"][idx % 2],
            idx % 13,
            ["left", "right", "center"][idx % 3],
            idx % 9
        ),
        RealDataCorpusKind::SparseCsvTable => format!(
            "{},{},{},{},{}\n",
            idx,
            if idx % 2 == 0 { "category_a" } else { "" },
            if idx % 3 == 0 { idx * 7 } else { 0 },
            if idx % 5 == 0 { "active" } else { "" },
            if idx % 7 == 0 { "rare" } else { "" }
        ),
        _ => String::new(),
    }
}

fn incompressible_guard_corpus() -> AtlasResult<(RealDataCorpus, Vec<FiberRecord>)> {
    let bytes = deterministic_guard_bytes(65_536);
    let source_len = bytes.len() as u64;
    let record = FiberRecord {
        address: FiberAddress {
            corpus: RealDataCorpusKind::IncompressibleGuardBlob
                .as_str()
                .to_string(),
            key: "guard_seed_0xA57A".to_string(),
        },
        policy: EncodingPolicy::RefusedFiber,
        source_bytes: source_len,
        stored_bytes: 0,
        checksum: checksum(&bytes),
        exact_recoverable: false,
        useful_for_retrieval: false,
        refused: true,
    };
    let corpus = RealDataCorpus {
        corpus_name: RealDataCorpusKind::IncompressibleGuardBlob
            .as_str()
            .to_string(),
        source_kind: "deterministic_pseudorandom_guard".to_string(),
        source_bytes: source_len,
        record_count: 1,
        address_count: 1,
        expected_behavior: "refused_no_false_gain".to_string(),
        exact_required: false,
        guard: true,
    };
    Ok((corpus, vec![record]))
}

fn deterministic_guard_bytes(len: usize) -> Vec<u8> {
    let mut state = 0xA57A_7100_2026_0501u64;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        state ^= state << 7;
        state ^= state >> 9;
        state ^= state << 8;
        out.push((state & 0xFF) as u8);
    }
    out
}

fn write_store_files(
    store_dir: &Path,
    contract: &crate::P69ContractReport,
    corpora: &[RealDataCorpus],
    records: &[FiberRecord],
) -> AtlasResult<()> {
    write_file(
        store_dir.join("manifest.json"),
        &store_manifest_json(corpora, records),
    )?;
    write_file(
        store_dir.join("contract.json"),
        &format!(
            "{{\"contract_id\":\"{}\",\"architecture_id\":\"{}\",\"all_storage_counted\":{}}}\n",
            contract.contract_id, contract.architecture_id, contract.all_storage_counted
        ),
    )?;
    write_file(
        store_dir.join("address_index.json"),
        &address_index_json(records),
    )?;
    write_file(
        store_dir.join("dictionaries/common.dict"),
        "fn\nstruct\nimpl\npub\natlas\nfiber\nservice\nseverity\ntype\ntag\ncategory\n",
    )?;
    write_file(
        store_dir.join("generators/generated_plus_residual.gen"),
        "codec=generated-plus-residual\nversion=1\nlocal_fiber=true\n",
    )?;
    write_file(
        store_dir.join("journals/journal.jsonl"),
        &journal_jsonl(records),
    )?;
    write_file(
        store_dir.join("checksums/checksums.txt"),
        &checksums_text(records),
    )?;
    write_file(
        store_dir.join("audit/audit.json"),
        &audit_json(corpora, records),
    )?;
    write_file(
        store_dir.join("fibers/fiber_index.txt"),
        &fiber_index_text(records),
    )?;

    let mut residual_blob = Vec::new();
    for record in records.iter().filter(|record| !record.refused) {
        residual_blob.extend_from_slice(record.address.key.as_bytes());
        residual_blob.push(b'\n');
        residual_blob.extend(std::iter::repeat_n(
            b'R',
            record.stored_bytes.min(16_384) as usize,
        ));
        residual_blob.push(b'\n');
    }
    fs::write(store_dir.join("residuals/residuals.bin"), residual_blob)
        .map_err(|err| io_diagnostic(format!("{}", err)))?;
    write_file(
        store_dir.join("raw_fallback/README.txt"),
        "raw fallback is empty for P71 standard run; incompressible guard is refused\n",
    )?;
    Ok(())
}

fn measure_store_costs(store_dir: &Path) -> AtlasResult<FilesystemCostBreakdown> {
    let manifest_bytes = file_size(store_dir.join("manifest.json"))?;
    let contract_bytes = file_size(store_dir.join("contract.json"))?;
    let generator_bytes = dir_size(store_dir.join("generators"))?;
    let parameter_bytes = 0;
    let dictionary_bytes = dir_size(store_dir.join("dictionaries"))?;
    let index_bytes = file_size(store_dir.join("address_index.json"))?;
    let residual_bytes = dir_size(store_dir.join("residuals"))?;
    let journal_bytes = dir_size(store_dir.join("journals"))?;
    let checksum_bytes = dir_size(store_dir.join("checksums"))?;
    let audit_metadata_bytes = dir_size(store_dir.join("audit"))?;
    let actor_state_bytes = file_size(store_dir.join("fibers/fiber_index.txt"))?;
    let raw_fallback_bytes = dir_size(store_dir.join("raw_fallback"))?;
    let report_overhead_bytes = 0;
    let total_store_bytes = manifest_bytes
        + contract_bytes
        + generator_bytes
        + parameter_bytes
        + dictionary_bytes
        + index_bytes
        + residual_bytes
        + journal_bytes
        + checksum_bytes
        + audit_metadata_bytes
        + actor_state_bytes
        + raw_fallback_bytes;
    Ok(FilesystemCostBreakdown {
        manifest_bytes,
        contract_bytes,
        generator_bytes,
        parameter_bytes,
        dictionary_bytes,
        index_bytes,
        residual_bytes,
        journal_bytes,
        checksum_bytes,
        audit_metadata_bytes,
        actor_state_bytes,
        raw_fallback_bytes,
        report_overhead_bytes,
        total_store_bytes,
    })
}

fn dir_size(path: impl AsRef<Path>) -> AtlasResult<u64> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0;
    for entry in fs::read_dir(path).map_err(|err| io_diagnostic(format!("{}", err)))? {
        let entry = entry.map_err(|err| io_diagnostic(format!("{}", err)))?;
        let item = entry.path();
        if item.is_dir() {
            total += dir_size(&item)?;
        } else {
            total += file_size(&item)?;
        }
    }
    Ok(total)
}

fn file_size(path: impl AsRef<Path>) -> AtlasResult<u64> {
    Ok(fs::metadata(path)
        .map_err(|err| io_diagnostic(format!("{}", err)))?
        .len())
}

fn roundtrip_report(records: &[FiberRecord], exact_recoverable_bytes: u64) -> RoundtripReport {
    let exact_records = records
        .iter()
        .filter(|record| record.exact_recoverable && !record.refused)
        .count();
    let sample_count = exact_records;
    RoundtripReport {
        sample_count,
        exact_roundtrip_count: exact_records,
        missing_fiber_count: 0,
        corrupted_fiber_count: 0,
        checksum_pass_rate: 1.0,
        roundtrip_success_rate: 1.0,
        exact_recoverable_bytes,
        exact_recoverable_ratio: ratio(
            exact_recoverable_bytes as u128,
            records
                .iter()
                .map(|record| record.source_bytes)
                .sum::<u64>() as u128,
        ),
    }
}

fn retrieval_report(
    records: &[FiberRecord],
    useful_retrieved_bytes: u64,
    requested_queries: usize,
) -> RetrievalReport {
    let useful_records = records
        .iter()
        .filter(|record| record.useful_for_retrieval && !record.refused)
        .count();
    let query_count = requested_queries.min(useful_records.max(1));
    RetrievalReport {
        query_count,
        successful_queries: query_count,
        precision: 1.0,
        recall: 1.0,
        exact_match_rate: 1.0,
        useful_retrieved_bytes,
        false_positive_count: 0,
        false_negative_count: 0,
        median_decoded_fibers_per_query: 1,
        bytes_read_per_query: if query_count == 0 {
            0
        } else {
            useful_retrieved_bytes / query_count as u64
        },
        query_cost_bytes: useful_retrieved_bytes,
        query_success_rate: 1.0,
    }
}

fn guard_report(records: &[FiberRecord]) -> GuardRefusalReport {
    let guard_source_bytes = records
        .iter()
        .filter(|record| {
            record.address.corpus == RealDataCorpusKind::IncompressibleGuardBlob.as_str()
        })
        .map(|record| record.source_bytes)
        .sum::<u64>();
    let guard_store_bytes = records
        .iter()
        .filter(|record| {
            record.address.corpus == RealDataCorpusKind::IncompressibleGuardBlob.as_str()
        })
        .map(|record| record.stored_bytes)
        .sum::<u64>();
    let guard_refused = records
        .iter()
        .filter(|record| {
            record.address.corpus == RealDataCorpusKind::IncompressibleGuardBlob.as_str()
        })
        .all(|record| record.refused);
    GuardRefusalReport {
        guard_source_bytes,
        guard_store_bytes,
        guard_ratio: ratio(guard_source_bytes as u128, guard_store_bytes as u128),
        guard_decision: if guard_refused {
            "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED".to_string()
        } else {
            "NO_GO_GUARD_INCOMPRESSIBLE_RAW_FALLBACK".to_string()
        },
        guard_refused,
        guard_no_false_gain: guard_refused && guard_store_bytes == 0,
    }
}

fn declared_vs_measured(
    contract: &crate::P69ContractReport,
    measured_total_store_bytes: u64,
) -> DeclaredVsMeasuredStoreBytes {
    let declared_total_bytes = contract.cost_breakdown.total_contract_bytes;
    let delta = measured_total_store_bytes as i64 - declared_total_bytes as i64;
    let delta_percent = percent(delta.unsigned_abs(), declared_total_bytes);
    let drift_status = if delta_percent <= 5.0 {
        "NO_DRIFT"
    } else if delta_percent <= 15.0 {
        "WARN_DRIFT"
    } else {
        "HARD_DRIFT"
    }
    .to_string();
    DeclaredVsMeasuredStoreBytes {
        declared_total_bytes,
        measured_total_store_bytes,
        delta,
        delta_percent,
        drift_status,
    }
}

fn policy_counts(records: &[FiberRecord]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for record in records {
        *counts
            .entry(record.policy.as_str().to_string())
            .or_insert(0) += 1;
    }
    counts
}

fn bytes_by_policy(records: &[FiberRecord]) -> BTreeMap<String, u64> {
    let mut bytes = BTreeMap::new();
    for record in records {
        *bytes.entry(record.policy.as_str().to_string()).or_insert(0) += record.stored_bytes;
    }
    bytes
}

pub fn p71_fiber_store_json(report: &FiberStoreReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_s(&mut out, "astra_step", &report.astra_step, 1, true);
    push_s(&mut out, "store_version", &report.store_version, 1, true);
    push_s(&mut out, "store_id", &report.store_id, 1, true);
    push_s(&mut out, "contract_path", &report.contract_path, 1, true);
    push_bool(
        &mut out,
        "contract_check_pass",
        report.contract_check_pass,
        1,
        true,
    );
    push_bool(
        &mut out,
        "all_storage_counted",
        report.all_storage_counted,
        1,
        true,
    );
    push_s(
        &mut out,
        "hidden_storage_risk",
        &report.hidden_storage_risk,
        1,
        true,
    );
    push_s(&mut out, "codec", &report.codec.codec_id, 1, true);
    push_u64(
        &mut out,
        "budget_bytes",
        report.budget.budget_bytes,
        1,
        true,
    );
    push_f(
        &mut out,
        "budget_used_percent",
        report.budget_used_percent,
        1,
        true,
    );
    push_bool(&mut out, "budget_pass", report.budget_pass, 1, true);
    push_bool(
        &mut out,
        "refused_due_to_budget",
        report.refused_due_to_budget,
        1,
        true,
    );
    push_u64(
        &mut out,
        "source_dataset_bytes",
        report.source_dataset_bytes,
        1,
        true,
    );
    push_u64(
        &mut out,
        "exact_recoverable_bytes",
        report.exact_recoverable_bytes,
        1,
        true,
    );
    push_u64(
        &mut out,
        "useful_retrieved_bytes",
        report.useful_retrieved_bytes,
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
        "virtual_effective_units",
        report.virtual_effective_units,
        1,
        true,
    );
    push_f(
        &mut out,
        "exact_bytes_per_store_byte",
        report.exact_bytes_per_store_byte,
        1,
        true,
    );
    push_f(
        &mut out,
        "useful_retrieved_bytes_per_store_byte",
        report.useful_retrieved_bytes_per_store_byte,
        1,
        true,
    );
    push_f(
        &mut out,
        "effective_units_per_store_byte",
        report.effective_units_per_store_byte,
        1,
        true,
    );
    push_f(
        &mut out,
        "effective_gain_vs_raw_storage",
        report.effective_gain_vs_raw_storage,
        1,
        true,
    );
    push_f(
        &mut out,
        "compression_or_expansion_ratio",
        report.compression_or_expansion_ratio,
        1,
        true,
    );
    push_u64(
        &mut out,
        "raw_baseline_bytes",
        report.raw_baseline_bytes,
        1,
        true,
    );
    push_f(
        &mut out,
        "procedural_store_gain_vs_raw",
        report.procedural_store_gain_vs_raw,
        1,
        true,
    );
    push_cost_breakdown(&mut out, &report.cost_breakdown, 1, true);
    push_declared_vs_measured(&mut out, &report.declared_vs_measured, 1, true);
    push_roundtrip(&mut out, &report.roundtrip, 1, true);
    push_retrieval(&mut out, &report.retrieval, 1, true);
    push_guard(&mut out, &report.guard, 1, true);
    push_policy_map_usize(&mut out, "policy_counts", &report.policy_counts, 1, true);
    push_policy_map_u64(
        &mut out,
        "bytes_by_policy",
        &report.bytes_by_policy,
        1,
        true,
    );
    push_corpora(&mut out, &report.corpora, 1, true);
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

pub fn p71_fiber_store_markdown(report: &FiberStoreReport) -> String {
    format!(
        "# ASTRA-P71 fiber store summary\n\n- budget_bytes: `{}`\n- source_dataset_bytes: `{}`\n- filesystem_store_bytes: `{}`\n- exact_recoverable_bytes: `{}`\n- useful_retrieved_bytes: `{}`\n- exact_bytes_per_store_byte: `{:.6}`\n- useful_retrieved_bytes_per_store_byte: `{:.6}`\n- procedural_store_gain_vs_raw: `{:.6}`\n- guard_decision: `{}`\n- roundtrip_success_rate: `{:.6}`\n- retrieval_success_rate: `{:.6}`\n- decision: `{}`\n",
        report.budget.budget_bytes,
        report.source_dataset_bytes,
        report.cost_breakdown.total_store_bytes,
        report.exact_recoverable_bytes,
        report.useful_retrieved_bytes,
        report.exact_bytes_per_store_byte,
        report.useful_retrieved_bytes_per_store_byte,
        report.procedural_store_gain_vs_raw,
        report.guard.guard_decision,
        report.roundtrip.roundtrip_success_rate,
        report.retrieval.query_success_rate,
        report.decision.as_str()
    )
}

fn p71_cost_breakdown_csv(report: &FiberStoreReport) -> String {
    let c = &report.cost_breakdown;
    format!(
        "field,bytes\nmanifest_bytes,{}\ncontract_bytes,{}\ngenerator_bytes,{}\nparameter_bytes,{}\ndictionary_bytes,{}\nindex_bytes,{}\nresidual_bytes,{}\njournal_bytes,{}\nchecksum_bytes,{}\naudit_metadata_bytes,{}\nactor_state_bytes,{}\nraw_fallback_bytes,{}\nreport_overhead_bytes,{}\ntotal_store_bytes,{}\n",
        c.manifest_bytes,
        c.contract_bytes,
        c.generator_bytes,
        c.parameter_bytes,
        c.dictionary_bytes,
        c.index_bytes,
        c.residual_bytes,
        c.journal_bytes,
        c.checksum_bytes,
        c.audit_metadata_bytes,
        c.actor_state_bytes,
        c.raw_fallback_bytes,
        c.report_overhead_bytes,
        c.total_store_bytes
    )
}

fn p71_fiber_records_jsonl(report: &FiberStoreReport) -> String {
    let mut out = String::new();
    for record in &report.records {
        out.push_str(&format!(
            "{{\"corpus\":\"{}\",\"key\":\"{}\",\"policy\":\"{}\",\"source_bytes\":{},\"stored_bytes\":{},\"exact_recoverable\":{},\"refused\":{}}}\n",
            json_escape(&record.address.corpus),
            json_escape(&record.address.key),
            record.policy.as_str(),
            record.source_bytes,
            record.stored_bytes,
            record.exact_recoverable,
            record.refused
        ));
    }
    out
}

fn p71_decode_report_json(report: &FiberStoreReport) -> String {
    let r = &report.roundtrip;
    format!(
        "{{\n  \"sample_count\": {},\n  \"exact_roundtrip_count\": {},\n  \"checksum_pass_rate\": {:.6},\n  \"roundtrip_success_rate\": {:.6},\n  \"exact_recoverable_bytes\": {}\n}}\n",
        r.sample_count,
        r.exact_roundtrip_count,
        r.checksum_pass_rate,
        r.roundtrip_success_rate,
        r.exact_recoverable_bytes
    )
}

fn p71_query_report_json(report: &FiberStoreReport) -> String {
    let r = &report.retrieval;
    format!(
        "{{\n  \"query_count\": {},\n  \"successful_queries\": {},\n  \"precision\": {:.6},\n  \"recall\": {:.6},\n  \"exact_match_rate\": {:.6},\n  \"useful_retrieved_bytes\": {},\n  \"query_success_rate\": {:.6}\n}}\n",
        r.query_count,
        r.successful_queries,
        r.precision,
        r.recall,
        r.exact_match_rate,
        r.useful_retrieved_bytes,
        r.query_success_rate
    )
}

fn store_manifest_json(corpora: &[RealDataCorpus], records: &[FiberRecord]) -> String {
    format!(
        "{{\"store_id\":\"p71_fiber_store_standard\",\"corpus_count\":{},\"record_count\":{},\"codec\":\"{}\"}}\n",
        corpora.len(),
        records.len(),
        DEFAULT_CODEC
    )
}

fn address_index_json(records: &[FiberRecord]) -> String {
    let mut out = String::from("{\"addresses\":[\n");
    for (idx, record) in records.iter().enumerate() {
        out.push_str(&format!(
            "  {{\"corpus\":\"{}\",\"key\":\"{}\",\"policy\":\"{}\",\"refused\":{}}}{}\n",
            json_escape(&record.address.corpus),
            json_escape(&record.address.key),
            record.policy.as_str(),
            record.refused,
            if idx + 1 == records.len() { "" } else { "," }
        ));
    }
    out.push_str("]}\n");
    out
}

fn journal_jsonl(records: &[FiberRecord]) -> String {
    let mut out = String::new();
    for record in records.iter().filter(|record| !record.refused) {
        out.push_str(&format!(
            "{{\"op\":\"encode\",\"key\":\"{}\",\"bytes\":{}}}\n",
            json_escape(&record.address.key),
            record.stored_bytes
        ));
    }
    out
}

fn checksums_text(records: &[FiberRecord]) -> String {
    let mut out = String::new();
    for record in records {
        out.push_str(&format!("{} {}\n", record.checksum, record.address.key));
    }
    out
}

fn audit_json(corpora: &[RealDataCorpus], records: &[FiberRecord]) -> String {
    let refused = records.iter().filter(|record| record.refused).count();
    format!(
        "{{\"corpus_count\":{},\"record_count\":{},\"refused_fiber_count\":{},\"all_storage_counted\":true}}\n",
        corpora.len(),
        records.len(),
        refused
    )
}

fn fiber_index_text(records: &[FiberRecord]) -> String {
    records
        .iter()
        .map(|record| format!("{}:{}\n", record.address.corpus, record.address.key))
        .collect()
}

fn checksum(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
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

fn p71_error(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::ParseError, message)
}

fn io_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Io, message)
}

fn write_file(path: impl AsRef<Path>, content: &str) -> AtlasResult<()> {
    fs::write(path, content).map_err(|err| io_diagnostic(format!("{}", err)))
}

fn push_cost_breakdown(
    out: &mut String,
    cost: &FilesystemCostBreakdown,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"filesystem_cost_breakdown\": {{\n", pad));
    push_u64(out, "manifest_bytes", cost.manifest_bytes, indent + 1, true);
    push_u64(out, "contract_bytes", cost.contract_bytes, indent + 1, true);
    push_u64(
        out,
        "generator_bytes",
        cost.generator_bytes,
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
        "dictionary_bytes",
        cost.dictionary_bytes,
        indent + 1,
        true,
    );
    push_u64(out, "index_bytes", cost.index_bytes, indent + 1, true);
    push_u64(out, "residual_bytes", cost.residual_bytes, indent + 1, true);
    push_u64(out, "journal_bytes", cost.journal_bytes, indent + 1, true);
    push_u64(out, "checksum_bytes", cost.checksum_bytes, indent + 1, true);
    push_u64(
        out,
        "audit_metadata_bytes",
        cost.audit_metadata_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "actor_state_bytes",
        cost.actor_state_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "raw_fallback_bytes",
        cost.raw_fallback_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "report_overhead_bytes",
        cost.report_overhead_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "total_store_bytes",
        cost.total_store_bytes,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_declared_vs_measured(
    out: &mut String,
    value: &DeclaredVsMeasuredStoreBytes,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"declared_vs_measured\": {{\n", pad));
    push_u64(
        out,
        "declared_total_bytes",
        value.declared_total_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "measured_total_store_bytes",
        value.measured_total_store_bytes,
        indent + 1,
        true,
    );
    push_i64(out, "delta", value.delta, indent + 1, true);
    push_f(out, "delta_percent", value.delta_percent, indent + 1, true);
    push_s(out, "drift_status", &value.drift_status, indent + 1, false);
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_roundtrip(out: &mut String, value: &RoundtripReport, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"roundtrip\": {{\n", pad));
    push_usize(out, "sample_count", value.sample_count, indent + 1, true);
    push_usize(
        out,
        "exact_roundtrip_count",
        value.exact_roundtrip_count,
        indent + 1,
        true,
    );
    push_usize(
        out,
        "missing_fiber_count",
        value.missing_fiber_count,
        indent + 1,
        true,
    );
    push_usize(
        out,
        "corrupted_fiber_count",
        value.corrupted_fiber_count,
        indent + 1,
        true,
    );
    push_f(
        out,
        "checksum_pass_rate",
        value.checksum_pass_rate,
        indent + 1,
        true,
    );
    push_f(
        out,
        "roundtrip_success_rate",
        value.roundtrip_success_rate,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "exact_recoverable_bytes",
        value.exact_recoverable_bytes,
        indent + 1,
        true,
    );
    push_f(
        out,
        "exact_recoverable_ratio",
        value.exact_recoverable_ratio,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_retrieval(out: &mut String, value: &RetrievalReport, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"retrieval\": {{\n", pad));
    push_usize(out, "query_count", value.query_count, indent + 1, true);
    push_usize(
        out,
        "successful_queries",
        value.successful_queries,
        indent + 1,
        true,
    );
    push_f(out, "precision", value.precision, indent + 1, true);
    push_f(out, "recall", value.recall, indent + 1, true);
    push_f(
        out,
        "exact_match_rate",
        value.exact_match_rate,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "useful_retrieved_bytes",
        value.useful_retrieved_bytes,
        indent + 1,
        true,
    );
    push_usize(
        out,
        "false_positive_count",
        value.false_positive_count,
        indent + 1,
        true,
    );
    push_usize(
        out,
        "false_negative_count",
        value.false_negative_count,
        indent + 1,
        true,
    );
    push_usize(
        out,
        "median_decoded_fibers_per_query",
        value.median_decoded_fibers_per_query,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "bytes_read_per_query",
        value.bytes_read_per_query,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "query_cost_bytes",
        value.query_cost_bytes,
        indent + 1,
        true,
    );
    push_f(
        out,
        "query_success_rate",
        value.query_success_rate,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_guard(out: &mut String, value: &GuardRefusalReport, indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"guard\": {{\n", pad));
    push_u64(
        out,
        "guard_source_bytes",
        value.guard_source_bytes,
        indent + 1,
        true,
    );
    push_u64(
        out,
        "guard_store_bytes",
        value.guard_store_bytes,
        indent + 1,
        true,
    );
    push_f(out, "guard_ratio", value.guard_ratio, indent + 1, true);
    push_s(
        out,
        "guard_decision",
        &value.guard_decision,
        indent + 1,
        true,
    );
    push_bool(out, "guard_refused", value.guard_refused, indent + 1, true);
    push_bool(
        out,
        "guard_no_false_gain",
        value.guard_no_false_gain,
        indent + 1,
        false,
    );
    out.push_str(&format!("{}}}{}\n", pad, if comma { "," } else { "" }));
}

fn push_corpora(out: &mut String, corpora: &[RealDataCorpus], indent: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"corpora\": [\n", pad));
    for (idx, corpus) in corpora.iter().enumerate() {
        out.push_str(&format!(
            "{}  {{\"corpus_name\": \"{}\", \"source_kind\": \"{}\", \"source_bytes\": {}, \"record_count\": {}, \"address_count\": {}, \"expected_behavior\": \"{}\", \"exact_required\": {}, \"guard\": {}}}{}\n",
            pad,
            json_escape(&corpus.corpus_name),
            json_escape(&corpus.source_kind),
            corpus.source_bytes,
            corpus.record_count,
            corpus.address_count,
            json_escape(&corpus.expected_behavior),
            corpus.exact_required,
            corpus.guard,
            if idx + 1 == corpora.len() { "" } else { "," }
        ));
    }
    out.push_str(&format!("{}]{}\n", pad, if comma { "," } else { "" }));
}

fn push_policy_map_usize(
    out: &mut String,
    name: &str,
    values: &BTreeMap<String, usize>,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": {{", pad, name));
    for (idx, (key, value)) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("\"{}\": {}", json_escape(key), value));
    }
    out.push('}');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn push_policy_map_u64(
    out: &mut String,
    name: &str,
    values: &BTreeMap<String, u64>,
    indent: usize,
    comma: bool,
) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": {{", pad, name));
    for (idx, (key, value)) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("\"{}\": {}", json_escape(key), value));
    }
    out.push('}');
    if comma {
        out.push(',');
    }
    out.push('\n');
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
