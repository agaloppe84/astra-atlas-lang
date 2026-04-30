use astra_atlas_lang::{validate, parse_atlas_str, typecheck};

#[test]
fn valid_program_passes() {
    let text = include_str!("../examples/p53_strict.atlas");
    let p = validate(text).expect("valid atlas should pass");
    assert_eq!(p.families.len(), 12);
}

#[test]
fn snapshot_full_refused() {
    let text = include_str!("../examples/invalid/snapshot_full.atlas");
    assert!(validate(text).is_err());
}

#[test]
fn guard_active_refused() {
    let text = include_str!("../examples/invalid/guard_active.atlas");
    assert!(validate(text).is_err());
}

#[test]
fn bad_version_refused() {
    let text = include_str!("../examples/invalid/bad_version.atlas");
    assert!(parse_atlas_str(text).is_err());
}
