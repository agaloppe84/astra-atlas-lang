use std::env;
use astra_atlas_lang::{parse_atlas_file, typecheck};

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "examples/p53_strict.atlas".to_string());
    match parse_atlas_file(&path).and_then(|p| { typecheck(&p)?; Ok(p) }) {
        Ok(p) => {
            println!("OK: version={} families={}", p.version, p.families.len());
        }
        Err(e) => {
            eprintln!("ERR: {}", e);
            std::process::exit(1);
        }
    }
}
