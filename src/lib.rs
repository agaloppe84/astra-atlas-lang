use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;

pub mod cli;
mod p57;
mod p63;
mod p64;
mod p65;
mod p66;
mod p67;
mod p68;
mod p69;
mod p70;
mod p71;
mod real_ratio;
mod runtime;
mod virtual_ratio;
pub use p57::*;
pub use p63::*;
pub use p64::*;
pub use p65::*;
pub use p66::*;
pub use p67::*;
pub use p68::*;
pub use p69::*;
pub use p70::*;
pub use p71::*;
pub use real_ratio::*;
pub use runtime::*;
pub use virtual_ratio::*;

const MIN_THRESHOLD: f64 = 0.05;
const MAX_THRESHOLD: f64 = 0.50;

#[derive(Debug, Clone, PartialEq)]
pub struct FamilySpec {
    pub name: String,
    pub action: String,
    pub safety: String,
    pub layout: String,
    pub index: String,
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtlasProgram {
    pub version: String,
    pub runtime: BTreeMap<String, String>,
    pub families: Vec<FamilySpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    VersionUnknown,
    GuardActive,
    SnapshotFullStrict,
    ActionUnknown,
    SafetyUnknown,
    UnknownLayout,
    UnknownIndex,
    LayoutIndexMismatch,
    ThresholdMalformed,
    ThresholdOutOfRange,
    ThresholdInvalid,
    MissingFamilies,
    FamilyUnknown,
    FamilyDuplicate,
    DuplicateKey,
    MissingLayout,
    FieldMissing,
    ParseError,
    ActiveWithoutSafety,
    Io,
}

impl DiagnosticCode {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticCode::VersionUnknown => "E_VERSION_UNKNOWN",
            DiagnosticCode::GuardActive => "E_GUARD_ACTIVE",
            DiagnosticCode::SnapshotFullStrict => "E_SNAPSHOT_FULL_STRICT",
            DiagnosticCode::ActionUnknown => "E_ACTION_UNKNOWN",
            DiagnosticCode::SafetyUnknown => "E_SAFETY_UNKNOWN",
            DiagnosticCode::UnknownLayout => "E_UNKNOWN_LAYOUT",
            DiagnosticCode::UnknownIndex => "E_UNKNOWN_INDEX",
            DiagnosticCode::LayoutIndexMismatch => "E_LAYOUT_INDEX_MISMATCH",
            DiagnosticCode::ThresholdMalformed => "E_THRESHOLD_MALFORMED",
            DiagnosticCode::ThresholdOutOfRange => "E_THRESHOLD_OUT_OF_RANGE",
            DiagnosticCode::ThresholdInvalid => "E_THRESHOLD_INVALID",
            DiagnosticCode::MissingFamilies => "E_MISSING_FAMILIES",
            DiagnosticCode::FamilyUnknown => "E_FAMILY_UNKNOWN",
            DiagnosticCode::FamilyDuplicate => "E_FAMILY_DUPLICATE",
            DiagnosticCode::DuplicateKey => "E_DUPLICATE_KEY",
            DiagnosticCode::MissingLayout => "E_MISSING_LAYOUT",
            DiagnosticCode::FieldMissing => "E_FIELD_MISSING",
            DiagnosticCode::ParseError => "E_PARSE",
            DiagnosticCode::ActiveWithoutSafety => "E_ACTIVE_WITHOUT_SAFETY",
            DiagnosticCode::Io => "E_IO",
        }
    }

    pub fn explanation(self) -> &'static str {
        match self {
            DiagnosticCode::VersionUnknown => "The atlas version is not supported by P55.1.",
            DiagnosticCode::GuardActive => "The guard family must remain a refuse-only sentinel.",
            DiagnosticCode::SnapshotFullStrict => {
                "strict_p53 requires snapshot=incremental_manifest; snapshot=full is refused."
            }
            DiagnosticCode::ActionUnknown => "The action is not part of the strict P53 action set.",
            DiagnosticCode::SafetyUnknown => {
                "The safety mode is not part of the strict P53 safety set."
            }
            DiagnosticCode::UnknownLayout => "The layout is not part of the strict P53 layout set.",
            DiagnosticCode::UnknownIndex => "The index is not part of the strict P53 index set.",
            DiagnosticCode::LayoutIndexMismatch => {
                "The family action, layout, and index do not match the strict P53 table."
            }
            DiagnosticCode::ThresholdMalformed => "The threshold is not a finite number.",
            DiagnosticCode::ThresholdOutOfRange => "The threshold is outside the strict P53 range.",
            DiagnosticCode::ThresholdInvalid => {
                "The threshold must be a finite number in the strict P53 range."
            }
            DiagnosticCode::MissingFamilies => {
                "The program does not define every required strict P53 family."
            }
            DiagnosticCode::FamilyUnknown => {
                "The family name is not part of the strict P53 family set."
            }
            DiagnosticCode::FamilyDuplicate => "A strict P53 family appears more than once.",
            DiagnosticCode::DuplicateKey => "A .atlas line repeats the same key.",
            DiagnosticCode::MissingLayout => "A strict P53 family is missing its layout key.",
            DiagnosticCode::FieldMissing => "A required .atlas key is missing.",
            DiagnosticCode::ParseError => {
                "The .atlas source does not match the strict line format."
            }
            DiagnosticCode::ActiveWithoutSafety => "An active action cannot use safety=refuse.",
            DiagnosticCode::Io => "The requested .atlas file could not be read.",
        }
    }

    pub fn from_str(code: &str) -> Option<Self> {
        match code {
            "E_VERSION_UNKNOWN" => Some(DiagnosticCode::VersionUnknown),
            "E_GUARD_ACTIVE" => Some(DiagnosticCode::GuardActive),
            "E_SNAPSHOT_FULL_STRICT" => Some(DiagnosticCode::SnapshotFullStrict),
            "E_ACTION_UNKNOWN" => Some(DiagnosticCode::ActionUnknown),
            "E_SAFETY_UNKNOWN" => Some(DiagnosticCode::SafetyUnknown),
            "E_UNKNOWN_LAYOUT" => Some(DiagnosticCode::UnknownLayout),
            "E_UNKNOWN_INDEX" => Some(DiagnosticCode::UnknownIndex),
            "E_LAYOUT_INDEX_MISMATCH" => Some(DiagnosticCode::LayoutIndexMismatch),
            "E_THRESHOLD_MALFORMED" => Some(DiagnosticCode::ThresholdMalformed),
            "E_THRESHOLD_OUT_OF_RANGE" => Some(DiagnosticCode::ThresholdOutOfRange),
            "E_THRESHOLD_INVALID" => Some(DiagnosticCode::ThresholdInvalid),
            "E_MISSING_FAMILIES" => Some(DiagnosticCode::MissingFamilies),
            "E_FAMILY_UNKNOWN" => Some(DiagnosticCode::FamilyUnknown),
            "E_FAMILY_DUPLICATE" => Some(DiagnosticCode::FamilyDuplicate),
            "E_DUPLICATE_KEY" => Some(DiagnosticCode::DuplicateKey),
            "E_MISSING_LAYOUT" => Some(DiagnosticCode::MissingLayout),
            "E_FIELD_MISSING" => Some(DiagnosticCode::FieldMissing),
            "E_PARSE" => Some(DiagnosticCode::ParseError),
            "E_ACTIVE_WITHOUT_SAFETY" => Some(DiagnosticCode::ActiveWithoutSafety),
            "E_IO" => Some(DiagnosticCode::Io),
            _ => None,
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub line: Option<usize>,
    pub family: Option<String>,
    pub field: Option<String>,
}

impl Diagnostic {
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            line: None,
            family: None,
            field: None,
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_family(mut self, family: impl Into<String>) -> Self {
        self.family = Some(family.into());
        self
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if let Some(line) = self.line {
            write!(f, " (line {})", line)?;
        }
        if let Some(family) = &self.family {
            write!(f, " (family {})", family)?;
        }
        if let Some(field) = &self.field {
            write!(f, " (field {})", field)?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostic {}

pub type AtlasResult<T> = Result<T, Diagnostic>;

#[derive(Debug, Clone, Copy)]
struct FamilyRule {
    name: &'static str,
    action: &'static str,
    safety: &'static str,
    layout: &'static str,
    index: &'static str,
}

const FAMILY_RULES: &[FamilyRule] = &[
    FamilyRule {
        name: "guard",
        action: "refuse",
        safety: "refuse",
        layout: "refuse",
        index: "none",
    },
    FamilyRule {
        name: "stream_processing",
        action: "stream_delta",
        safety: "erasure_light",
        layout: "hotcold_segments",
        index: "stream_time_index",
    },
    FamilyRule {
        name: "sparse_index",
        action: "sparse_csr",
        safety: "erasure_light",
        layout: "sparse_blocks",
        index: "sparse_csr_index",
    },
    FamilyRule {
        name: "image_field_surrogate",
        action: "wavelet_tile",
        safety: "erasure_light",
        layout: "wavelet_tiles",
        index: "tile_pyramid",
    },
    FamilyRule {
        name: "log_request_index",
        action: "log_trie",
        safety: "erasure_light",
        layout: "log_trie_nodes",
        index: "log_trie_index",
    },
    FamilyRule {
        name: "columnar_table",
        action: "columnar_delta",
        safety: "erasure_light",
        layout: "columnar_chunks",
        index: "columnar_chunk_index",
    },
    FamilyRule {
        name: "graph_lowrank_surrogate",
        action: "low_rank",
        safety: "erasure_light",
        layout: "lowrank_factors",
        index: "lowrank_factor_index",
    },
    FamilyRule {
        name: "critical_sparse_archive",
        action: "sparse_csr",
        safety: "mirror_critical",
        layout: "sparse_blocks",
        index: "sparse_csr_index",
    },
    FamilyRule {
        name: "compressible_but_wrong",
        action: "dict_rom",
        safety: "erasure_light",
        layout: "dict_pages",
        index: "dict_rom_index",
    },
    FamilyRule {
        name: "field_surrogate",
        action: "wavelet_tile",
        safety: "erasure_light",
        layout: "wavelet_tiles",
        index: "tile_pyramid",
    },
    FamilyRule {
        name: "topological_field",
        action: "atlas_sheaf",
        safety: "erasure_light",
        layout: "atlas_cells",
        index: "atlas_sheaf_index",
    },
    FamilyRule {
        name: "local_global_conflict",
        action: "atlas_sheaf",
        safety: "erasure_light",
        layout: "atlas_cells",
        index: "atlas_sheaf_index",
    },
];

fn parse_kv(
    tokens: &[&str],
    line: usize,
    family: Option<&str>,
) -> AtlasResult<BTreeMap<String, String>> {
    let mut kv = BTreeMap::new();
    for tok in tokens {
        if let Some((k, v)) = tok.split_once('=') {
            if kv.contains_key(k) {
                let mut diagnostic = Diagnostic::new(
                    DiagnosticCode::DuplicateKey,
                    format!("duplicate key '{}'", k),
                )
                .with_line(line)
                .with_field(k);
                if let Some(family) = family {
                    diagnostic = diagnostic.with_family(family);
                }
                return Err(diagnostic);
            }
            kv.insert(k.to_string(), v.trim_matches('"').to_string());
        } else {
            return Err(Diagnostic::new(
                DiagnosticCode::ParseError,
                format!("token without '=': {}", tok),
            )
            .with_line(line));
        }
    }
    Ok(kv)
}

fn required_value(
    kv: &BTreeMap<String, String>,
    key: &str,
    line: usize,
    family: Option<&str>,
) -> AtlasResult<String> {
    kv.get(key).cloned().ok_or_else(|| {
        let code = if key == "layout" {
            DiagnosticCode::MissingLayout
        } else {
            DiagnosticCode::FieldMissing
        };
        let mut diagnostic = Diagnostic::new(code, format!("required key '{}' is missing", key))
            .with_line(line)
            .with_field(key);
        if let Some(family) = family {
            diagnostic = diagnostic.with_family(family);
        }
        diagnostic
    })
}

pub fn parse_atlas_str(text: &str) -> AtlasResult<AtlasProgram> {
    let mut version: Option<String> = None;
    let mut runtime: BTreeMap<String, String> = BTreeMap::new();
    let mut families: Vec<FamilySpec> = Vec::new();

    for (i, raw) in text.lines().enumerate() {
        let line_number = i + 1;
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
                let kv = parse_kv(&parts[1..], line_number, None)?;
                let v = required_value(&kv, "version", line_number, None)?;
                if v != "0.1" {
                    return Err(Diagnostic::new(
                        DiagnosticCode::VersionUnknown,
                        format!("unsupported atlas version '{}'", v),
                    )
                    .with_line(line_number)
                    .with_field("version"));
                }
                version = Some(v);
            }
            "runtime" => {
                runtime = parse_kv(&parts[1..], line_number, None)?;
            }
            "family" => {
                if parts.len() < 2 {
                    return Err(Diagnostic::new(
                        DiagnosticCode::FieldMissing,
                        "family name is missing",
                    )
                    .with_line(line_number)
                    .with_field("name"));
                }
                let name = parts[1].to_string();
                let kv = parse_kv(&parts[2..], line_number, Some(&name))?;
                let threshold_raw = required_value(&kv, "threshold", line_number, Some(&name))?;
                let threshold = threshold_raw.parse::<f64>().map_err(|_| {
                    Diagnostic::new(
                        DiagnosticCode::ThresholdMalformed,
                        format!("threshold '{}' is not a finite number", threshold_raw),
                    )
                    .with_line(line_number)
                    .with_family(name.clone())
                    .with_field("threshold")
                })?;
                families.push(FamilySpec {
                    action: required_value(&kv, "action", line_number, Some(&name))?,
                    safety: required_value(&kv, "safety", line_number, Some(&name))?,
                    layout: required_value(&kv, "layout", line_number, Some(&name))?,
                    index: required_value(&kv, "index", line_number, Some(&name))?,
                    threshold,
                    name,
                });
            }
            other => {
                return Err(Diagnostic::new(
                    DiagnosticCode::ParseError,
                    format!("unknown block '{}'", other),
                )
                .with_line(line_number));
            }
        }
    }

    Ok(AtlasProgram {
        version: version.ok_or_else(|| {
            Diagnostic::new(DiagnosticCode::FieldMissing, "atlas version is missing")
                .with_field("version")
        })?,
        runtime,
        families,
    })
}

pub fn parse_atlas_file(path: &str) -> AtlasResult<AtlasProgram> {
    let text = fs::read_to_string(path).map_err(|e| {
        Diagnostic::new(
            DiagnosticCode::Io,
            format!("could not read '{}': {}", path, e),
        )
    })?;
    parse_atlas_str(&text)
}

fn family_rule(name: &str) -> Option<&'static FamilyRule> {
    FAMILY_RULES.iter().find(|rule| rule.name == name)
}

fn family_position(name: &str) -> Option<usize> {
    FAMILY_RULES.iter().position(|rule| rule.name == name)
}

fn is_known_action(action: &str) -> bool {
    FAMILY_RULES.iter().any(|rule| rule.action == action)
}

fn is_known_safety(safety: &str) -> bool {
    FAMILY_RULES.iter().any(|rule| rule.safety == safety)
}

fn is_known_layout(layout: &str) -> bool {
    FAMILY_RULES.iter().any(|rule| rule.layout == layout)
}

fn is_known_index(index: &str) -> bool {
    FAMILY_RULES.iter().any(|rule| rule.index == index)
}

pub fn typecheck(program: &AtlasProgram) -> AtlasResult<()> {
    if program.runtime.get("strict_p53").map(|s| s.as_str()) == Some("true")
        && program.runtime.get("snapshot").map(|s| s.as_str()) != Some("incremental_manifest")
    {
        return Err(Diagnostic::new(
            DiagnosticCode::SnapshotFullStrict,
            "strict_p53 requires snapshot=incremental_manifest; snapshot=full is refused",
        )
        .with_field("snapshot"));
    }

    if program.runtime.get("snapshot").map(|s| s.as_str()) == Some("full") {
        return Err(Diagnostic::new(
            DiagnosticCode::SnapshotFullStrict,
            "snapshot=full is refused",
        )
        .with_field("snapshot"));
    }

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for family in &program.families {
        let rule = family_rule(&family.name).ok_or_else(|| {
            Diagnostic::new(
                DiagnosticCode::FamilyUnknown,
                format!("unknown family '{}'", family.name),
            )
            .with_family(family.name.clone())
        })?;

        if !seen.insert(family.name.as_str()) {
            return Err(Diagnostic::new(
                DiagnosticCode::FamilyDuplicate,
                format!("duplicate family '{}'", family.name),
            )
            .with_family(family.name.clone()));
        }

        if family.name == "guard"
            && (family.action != "refuse"
                || family.safety != "refuse"
                || family.layout != "refuse"
                || family.index != "none")
        {
            return Err(Diagnostic::new(
                DiagnosticCode::GuardActive,
                "guard must remain action=refuse safety=refuse layout=refuse index=none",
            )
            .with_family("guard"));
        }

        if !is_known_action(&family.action) {
            return Err(Diagnostic::new(
                DiagnosticCode::ActionUnknown,
                format!("unknown action '{}'", family.action),
            )
            .with_family(family.name.clone())
            .with_field("action"));
        }

        if !is_known_safety(&family.safety) {
            return Err(Diagnostic::new(
                DiagnosticCode::SafetyUnknown,
                format!("unknown safety '{}'", family.safety),
            )
            .with_family(family.name.clone())
            .with_field("safety"));
        }

        if family.action != "refuse" && family.safety == "refuse" {
            return Err(Diagnostic::new(
                DiagnosticCode::ActiveWithoutSafety,
                "active action cannot use safety=refuse",
            )
            .with_family(family.name.clone())
            .with_field("safety"));
        }

        if !family.threshold.is_finite() {
            return Err(Diagnostic::new(
                DiagnosticCode::ThresholdMalformed,
                format!("threshold {} is not a finite number", family.threshold),
            )
            .with_family(family.name.clone())
            .with_field("threshold"));
        }

        if family.threshold < MIN_THRESHOLD || family.threshold > MAX_THRESHOLD {
            return Err(Diagnostic::new(
                DiagnosticCode::ThresholdOutOfRange,
                format!(
                    "threshold {:.3} is outside [{:.2}, {:.2}]",
                    family.threshold, MIN_THRESHOLD, MAX_THRESHOLD
                ),
            )
            .with_family(family.name.clone())
            .with_field("threshold"));
        }

        if family.safety != rule.safety {
            return Err(Diagnostic::new(
                DiagnosticCode::SafetyUnknown,
                format!("family '{}' expects safety='{}'", family.name, rule.safety),
            )
            .with_family(family.name.clone())
            .with_field("safety"));
        }

        if !is_known_layout(&family.layout) {
            return Err(Diagnostic::new(
                DiagnosticCode::UnknownLayout,
                format!("unknown layout '{}'", family.layout),
            )
            .with_family(family.name.clone())
            .with_field("layout"));
        }

        if !is_known_index(&family.index) {
            return Err(Diagnostic::new(
                DiagnosticCode::UnknownIndex,
                format!("unknown index '{}'", family.index),
            )
            .with_family(family.name.clone())
            .with_field("index"));
        }

        if family.action != rule.action
            || family.layout != rule.layout
            || family.index != rule.index
        {
            let field = if family.action != rule.action {
                "action"
            } else if family.layout != rule.layout {
                "layout"
            } else {
                "index"
            };
            return Err(Diagnostic::new(
                DiagnosticCode::LayoutIndexMismatch,
                format!(
                    "family '{}' expects action='{}' layout='{}' index='{}'",
                    family.name, rule.action, rule.layout, rule.index
                ),
            )
            .with_family(family.name.clone())
            .with_field(field));
        }
    }

    let missing: Vec<&str> = FAMILY_RULES
        .iter()
        .filter_map(|rule| {
            if seen.contains(rule.name) {
                None
            } else {
                Some(rule.name)
            }
        })
        .collect();
    if !missing.is_empty() {
        return Err(Diagnostic::new(
            DiagnosticCode::MissingFamilies,
            format!("missing families: {}", missing.join(", ")),
        ));
    }

    Ok(())
}

pub fn validate(text: &str) -> AtlasResult<AtlasProgram> {
    let program = parse_atlas_str(text)?;
    typecheck(&program)?;
    Ok(program)
}

pub fn validate_file(path: &str) -> AtlasResult<AtlasProgram> {
    let program = parse_atlas_file(path)?;
    typecheck(&program)?;
    Ok(program)
}

pub fn export_json(text: &str) -> AtlasResult<String> {
    let program = validate(text)?;
    Ok(canonical_json(&program))
}

pub fn export_json_file(path: &str) -> AtlasResult<String> {
    let program = validate_file(path)?;
    Ok(canonical_json(&program))
}

pub fn canonical_json(program: &AtlasProgram) -> String {
    let mut families: Vec<&FamilySpec> = program.families.iter().collect();
    families.sort_by(|a, b| {
        family_position(&a.name)
            .unwrap_or(usize::MAX)
            .cmp(&family_position(&b.name).unwrap_or(usize::MAX))
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!(
        "  \"version\": \"{}\",\n",
        escape_json(&program.version)
    ));
    out.push_str("  \"runtime\": {\n");
    for (idx, (key, value)) in program.runtime.iter().enumerate() {
        let comma = if idx + 1 == program.runtime.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!(
            "    \"{}\": \"{}\"{}\n",
            escape_json(key),
            escape_json(value),
            comma
        ));
    }
    out.push_str("  },\n");
    out.push_str("  \"families\": [\n");
    for (idx, family) in families.iter().enumerate() {
        let comma = if idx + 1 == families.len() { "" } else { "," };
        out.push_str("    {\n");
        out.push_str(&format!(
            "      \"name\": \"{}\",\n",
            escape_json(&family.name)
        ));
        out.push_str(&format!(
            "      \"action\": \"{}\",\n",
            escape_json(&family.action)
        ));
        out.push_str(&format!(
            "      \"safety\": \"{}\",\n",
            escape_json(&family.safety)
        ));
        out.push_str(&format!(
            "      \"layout\": \"{}\",\n",
            escape_json(&family.layout)
        ));
        out.push_str(&format!(
            "      \"index\": \"{}\",\n",
            escape_json(&family.index)
        ));
        out.push_str(&format!("      \"threshold\": {:.3}\n", family.threshold));
        out.push_str(&format!("    }}{}\n", comma));
    }
    out.push_str("  ]\n");
    out.push('}');
    out
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
