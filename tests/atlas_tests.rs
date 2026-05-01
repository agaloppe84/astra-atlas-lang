use astra_atlas_lang::{canonical_json, export_json, validate, DiagnosticCode};
use std::fs;
use std::path::Path;

struct InvalidCase {
    name: &'static str,
    text: &'static str,
    code: DiagnosticCode,
    family: Option<&'static str>,
    field: Option<&'static str>,
    message: &'static str,
}

fn invalid_cases() -> [InvalidCase; 38] {
    [
        InvalidCase {
            name: "bad_version",
            text: include_str!("../examples/invalid/bad_version.atlas"),
            code: DiagnosticCode::VersionUnknown,
            family: None,
            field: Some("version"),
            message: "unsupported atlas version",
        },
        InvalidCase {
            name: "guard_active",
            text: include_str!("../examples/invalid/guard_active.atlas"),
            code: DiagnosticCode::GuardActive,
            family: Some("guard"),
            field: None,
            message: "guard must remain",
        },
        InvalidCase {
            name: "snapshot_full",
            text: include_str!("../examples/invalid/snapshot_full.atlas"),
            code: DiagnosticCode::SnapshotFullStrict,
            family: None,
            field: Some("snapshot"),
            message: "strict_p53 requires snapshot=incremental_manifest",
        },
        InvalidCase {
            name: "bad_action",
            text: include_str!("../examples/invalid/bad_action.atlas"),
            code: DiagnosticCode::ActionUnknown,
            family: Some("stream_processing"),
            field: Some("action"),
            message: "unknown action",
        },
        InvalidCase {
            name: "bad_safety",
            text: include_str!("../examples/invalid/bad_safety.atlas"),
            code: DiagnosticCode::SafetyUnknown,
            family: Some("stream_processing"),
            field: Some("safety"),
            message: "unknown safety",
        },
        InvalidCase {
            name: "layout_mismatch",
            text: include_str!("../examples/invalid/layout_mismatch.atlas"),
            code: DiagnosticCode::LayoutIndexMismatch,
            family: Some("stream_processing"),
            field: Some("layout"),
            message: "expects action",
        },
        InvalidCase {
            name: "index_mismatch",
            text: include_str!("../examples/invalid/index_mismatch.atlas"),
            code: DiagnosticCode::LayoutIndexMismatch,
            family: Some("stream_processing"),
            field: Some("index"),
            message: "expects action",
        },
        InvalidCase {
            name: "threshold_bad",
            text: include_str!("../examples/invalid/threshold_bad.atlas"),
            code: DiagnosticCode::ThresholdOutOfRange,
            family: Some("stream_processing"),
            field: Some("threshold"),
            message: "outside",
        },
        InvalidCase {
            name: "missing_families",
            text: include_str!("../examples/invalid/missing_families.atlas"),
            code: DiagnosticCode::MissingFamilies,
            family: None,
            field: None,
            message: "missing families",
        },
        InvalidCase {
            name: "unknown_family",
            text: include_str!("../examples/invalid/unknown_family.atlas"),
            code: DiagnosticCode::FamilyUnknown,
            family: Some("alien"),
            field: None,
            message: "unknown family",
        },
        InvalidCase {
            name: "missing_semicolon",
            text: include_str!("../examples/invalid/missing_semicolon.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "missing terminating ';'",
        },
        InvalidCase {
            name: "active_without_safety",
            text: include_str!("../examples/invalid/active_without_safety.atlas"),
            code: DiagnosticCode::ActiveWithoutSafety,
            family: Some("sparse_index"),
            field: Some("safety"),
            message: "active action cannot use safety=refuse",
        },
        InvalidCase {
            name: "dangerous_encoded",
            text: include_str!("../examples/invalid/dangerous_encoded.atlas"),
            code: DiagnosticCode::ActionUnknown,
            family: Some("stream_processing"),
            field: Some("action"),
            message: "unknown action",
        },
        InvalidCase {
            name: "malformed_family",
            text: include_str!("../examples/invalid/malformed_family.atlas"),
            code: DiagnosticCode::FieldMissing,
            family: None,
            field: Some("name"),
            message: "family name is missing",
        },
        InvalidCase {
            name: "duplicate_family",
            text: include_str!("../examples/invalid/duplicate_family.atlas"),
            code: DiagnosticCode::FamilyDuplicate,
            family: Some("stream_processing"),
            field: None,
            message: "duplicate family",
        },
        InvalidCase {
            name: "duplicate_key",
            text: include_str!("../examples/invalid/duplicate_key.atlas"),
            code: DiagnosticCode::DuplicateKey,
            family: Some("stream_processing"),
            field: Some("action"),
            message: "duplicate key 'action'",
        },
        InvalidCase {
            name: "missing_layout",
            text: include_str!("../examples/invalid/missing_layout.atlas"),
            code: DiagnosticCode::MissingLayout,
            family: Some("stream_processing"),
            field: Some("layout"),
            message: "required key 'layout' is missing",
        },
        InvalidCase {
            name: "unknown_layout",
            text: include_str!("../examples/invalid/unknown_layout.atlas"),
            code: DiagnosticCode::UnknownLayout,
            family: Some("stream_processing"),
            field: Some("layout"),
            message: "unknown layout",
        },
        InvalidCase {
            name: "unknown_index",
            text: include_str!("../examples/invalid/unknown_index.atlas"),
            code: DiagnosticCode::UnknownIndex,
            family: Some("stream_processing"),
            field: Some("index"),
            message: "unknown index",
        },
        InvalidCase {
            name: "malformed_threshold",
            text: include_str!("../examples/invalid/malformed_threshold.atlas"),
            code: DiagnosticCode::ThresholdMalformed,
            family: Some("stream_processing"),
            field: Some("threshold"),
            message: "not a finite number",
        },
        InvalidCase {
            name: "threshold_low",
            text: include_str!("../examples/invalid/threshold_low.atlas"),
            code: DiagnosticCode::ThresholdOutOfRange,
            family: Some("stream_processing"),
            field: Some("threshold"),
            message: "outside",
        },
        InvalidCase {
            name: "p69_missing_fiber_schema",
            text: include_str!("../examples/invalid/p69_missing_fiber_schema.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p69_unknown_generator",
            text: include_str!("../examples/invalid/p69_unknown_generator.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p69_unaccounted_actor_state",
            text: include_str!("../examples/invalid/p69_unaccounted_actor_state.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p69_missing_all_storage_counted_gate",
            text: include_str!("../examples/invalid/p69_missing_all_storage_counted_gate.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p69_zero_budget_actor",
            text: include_str!("../examples/invalid/p69_zero_budget_actor.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p69_unknown_projection",
            text: include_str!("../examples/invalid/p69_unknown_projection.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p69_contract_unknown_reference",
            text: include_str!("../examples/invalid/p69_contract_unknown_reference.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p70_cache_unaccounted",
            text: include_str!("../examples/invalid/p70_cache_unaccounted.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p70_journal_unaccounted",
            text: include_str!("../examples/invalid/p70_journal_unaccounted.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p70_actor_state_unaccounted",
            text: include_str!("../examples/invalid/p70_actor_state_unaccounted.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p70_missing_audit_metadata",
            text: include_str!("../examples/invalid/p70_missing_audit_metadata.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p70_hidden_storage_risk_high",
            text: include_str!("../examples/invalid/p70_hidden_storage_risk_high.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p71_hidden_raw_fallback",
            text: include_str!("../examples/invalid/p71_hidden_raw_fallback.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p71_missing_checksum_store",
            text: include_str!("../examples/invalid/p71_missing_checksum_store.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p71_unaccounted_dictionary",
            text: include_str!("../examples/invalid/p71_unaccounted_dictionary.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p71_budget_exceeded_contract",
            text: include_str!("../examples/invalid/p71_budget_exceeded_contract.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
        InvalidCase {
            name: "p71_false_guard_gain",
            text: include_str!("../examples/invalid/p71_false_guard_gain.atlas"),
            code: DiagnosticCode::ParseError,
            family: None,
            field: None,
            message: "unknown block 'p69_contract'",
        },
    ]
}

#[test]
fn valid_programs_are_accepted() {
    let cases = [
        (
            "root p53 strict",
            include_str!("../examples/p53_strict.atlas"),
        ),
        (
            "valid corpus p53 strict",
            include_str!("../examples/valid/p53_strict.atlas"),
        ),
    ];

    for (name, text) in cases {
        let program = validate(text).unwrap_or_else(|err| panic!("{} should pass: {}", name, err));
        assert_eq!(program.version, "0.1", "{}", name);
        assert_eq!(program.families.len(), 12, "{}", name);
        assert_eq!(
            program
                .runtime
                .get("strict_p53")
                .map(|value| value.as_str()),
            Some("true"),
            "{}",
            name
        );
    }
}

#[test]
fn invalid_programs_are_refused_with_expected_codes() {
    for case in invalid_cases() {
        let err = validate(case.text).expect_err(case.name);
        assert_eq!(err.code, case.code, "{}", case.name);
        assert_eq!(err.family.as_deref(), case.family, "{}", case.name);
        assert_eq!(err.field.as_deref(), case.field, "{}", case.name);
        assert!(
            err.message.contains(case.message),
            "{}: '{}' did not contain '{}'",
            case.name,
            err.message,
            case.message
        );
    }
}

#[test]
fn all_invalid_corpus_files_are_refused() {
    let invalid_dir = Path::new("examples/invalid");
    let mut paths: Vec<_> = fs::read_dir(invalid_dir)
        .expect("invalid examples dir")
        .map(|entry| entry.expect("invalid dir entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("atlas"))
        .collect();
    paths.sort();

    assert_eq!(
        paths.len(),
        invalid_cases().len(),
        "stable diagnostic coverage should track every invalid corpus file"
    );

    for path in paths {
        let text = fs::read_to_string(&path).expect("invalid example text");
        if validate(&text).is_ok() {
            panic!("{} should be refused", path.display());
        }
    }
}

#[test]
fn required_diagnostic_codes_have_stable_strings() {
    let cases = [
        (DiagnosticCode::VersionUnknown, "E_VERSION_UNKNOWN"),
        (DiagnosticCode::GuardActive, "E_GUARD_ACTIVE"),
        (DiagnosticCode::SnapshotFullStrict, "E_SNAPSHOT_FULL_STRICT"),
        (DiagnosticCode::ActionUnknown, "E_ACTION_UNKNOWN"),
        (DiagnosticCode::SafetyUnknown, "E_SAFETY_UNKNOWN"),
        (DiagnosticCode::UnknownLayout, "E_UNKNOWN_LAYOUT"),
        (DiagnosticCode::UnknownIndex, "E_UNKNOWN_INDEX"),
        (
            DiagnosticCode::LayoutIndexMismatch,
            "E_LAYOUT_INDEX_MISMATCH",
        ),
        (DiagnosticCode::ThresholdMalformed, "E_THRESHOLD_MALFORMED"),
        (
            DiagnosticCode::ThresholdOutOfRange,
            "E_THRESHOLD_OUT_OF_RANGE",
        ),
        (DiagnosticCode::ThresholdInvalid, "E_THRESHOLD_INVALID"),
        (DiagnosticCode::MissingFamilies, "E_MISSING_FAMILIES"),
        (DiagnosticCode::FamilyUnknown, "E_FAMILY_UNKNOWN"),
        (DiagnosticCode::FamilyDuplicate, "E_FAMILY_DUPLICATE"),
        (DiagnosticCode::DuplicateKey, "E_DUPLICATE_KEY"),
        (DiagnosticCode::MissingLayout, "E_MISSING_LAYOUT"),
        (DiagnosticCode::FieldMissing, "E_FIELD_MISSING"),
        (DiagnosticCode::ParseError, "E_PARSE"),
        (
            DiagnosticCode::ActiveWithoutSafety,
            "E_ACTIVE_WITHOUT_SAFETY",
        ),
    ];

    for (code, expected) in cases {
        assert_eq!(code.as_str(), expected);
        assert_eq!(DiagnosticCode::from_str(expected), Some(code));
    }
}

#[test]
fn canonical_json_export_is_stable() {
    let program = validate(include_str!("../examples/p53_strict.atlas")).expect("valid program");
    let expected = include_str!("golden/p53_strict.json").trim_end_matches('\n');
    assert_eq!(canonical_json(&program), expected);
    assert_eq!(
        export_json(include_str!("../examples/p53_strict.atlas")).expect("valid export"),
        expected
    );
}

#[test]
fn canonical_json_export_orders_families_by_p53_table() {
    let text = include_str!("../examples/p53_strict.atlas");
    let mut prefix_lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("family "))
        .collect();
    let mut family_lines: Vec<&str> = text
        .lines()
        .filter(|line| line.trim_start().starts_with("family "))
        .collect();
    family_lines.reverse();

    prefix_lines.extend(family_lines);
    let reordered = format!("{}\n", prefix_lines.join("\n"));
    let expected = include_str!("golden/p53_strict.json").trim_end_matches('\n');
    assert_eq!(export_json(&reordered).expect("reordered export"), expected);
}

#[test]
fn canonical_json_export_refuses_invalid_programs() {
    for case in invalid_cases() {
        let err = export_json(case.text).expect_err(case.name);
        assert_eq!(err.code, case.code, "{}", case.name);
    }
}

#[test]
fn strict_p53_is_preserved() {
    let program = validate(include_str!("../examples/p53_strict.atlas")).expect("valid program");
    assert_eq!(
        program.runtime.get("snapshot").map(|value| value.as_str()),
        Some("incremental_manifest")
    );
    assert_eq!(
        program
            .runtime
            .get("strict_p53")
            .map(|value| value.as_str()),
        Some("true")
    );
}

#[test]
fn guard_active_is_refused() {
    let err = validate(include_str!("../examples/invalid/guard_active.atlas"))
        .expect_err("guard_active must be refused");
    assert_eq!(err.code, DiagnosticCode::GuardActive);
    assert_eq!(err.family.as_deref(), Some("guard"));
    assert!(err.message.contains("guard must remain"));
}

#[test]
fn snapshot_full_is_refused() {
    let err = validate(include_str!("../examples/invalid/snapshot_full.atlas"))
        .expect_err("snapshot_full must be refused");
    assert_eq!(err.code, DiagnosticCode::SnapshotFullStrict);
    assert_eq!(err.field.as_deref(), Some("snapshot"));
    assert!(err
        .message
        .contains("strict_p53 requires snapshot=incremental_manifest"));
}
