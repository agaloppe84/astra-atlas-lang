use astra_atlas_lang::{export_json_file, validate, validate_file, DiagnosticCode};
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Err(message) = run(&args) {
        eprintln!("{}", message);
        std::process::exit(1);
    }
}

fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return check_path("examples/p53_strict.atlas");
    }

    match args[0].as_str() {
        "check" => {
            let path = args
                .get(1)
                .ok_or_else(|| usage("check requires a .atlas path"))?;
            if args.len() != 2 {
                return Err(usage("check accepts exactly one .atlas path"));
            }
            check_path(path)
        }
        "explain" => {
            let code = args
                .get(1)
                .ok_or_else(|| usage("explain requires a diagnostic code"))?;
            if args.len() != 2 {
                return Err(usage("explain accepts exactly one diagnostic code"));
            }
            explain_code(code)
        }
        "export" => export_json_command(args),
        "bench" => bench_command(args),
        path if args.len() == 1 => check_path(path),
        _ => Err(usage("unknown command")),
    }
}

fn check_path(path: &str) -> Result<(), String> {
    match validate_file(path) {
        Ok(program) => {
            println!(
                "OK: version={} families={}",
                program.version,
                program.families.len()
            );
            Ok(())
        }
        Err(diagnostic) => Err(diagnostic.to_string()),
    }
}

fn explain_code(code: &str) -> Result<(), String> {
    let diagnostic_code = DiagnosticCode::from_str(code)
        .ok_or_else(|| format!("unknown diagnostic code: {}", code))?;
    println!("{}: {}", diagnostic_code, diagnostic_code.explanation());
    Ok(())
}

fn export_json_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("export requires a .atlas path"))?;
    if !has_json_format(&args[2..]) {
        return Err(usage("export requires --format json"));
    }
    let json = export_json_file(path).map_err(|diagnostic| diagnostic.to_string())?;
    println!("{}", json);
    Ok(())
}

fn has_json_format(args: &[String]) -> bool {
    matches!(args, [flag, value] if flag.as_str() == "--format" && value.as_str() == "json")
        || matches!(args, [flag] if flag.as_str() == "--format=json")
}

fn bench_command(args: &[String]) -> Result<(), String> {
    if !has_smoke_mode(&args[1..]) {
        return Err(usage("bench requires --mode smoke"));
    }
    let program = validate(include_str!("../examples/p53_strict.atlas"))
        .map_err(|diagnostic| diagnostic.to_string())?;
    println!("OK: bench smoke families={}", program.families.len());
    Ok(())
}

fn has_smoke_mode(args: &[String]) -> bool {
    matches!(args, [flag, value] if flag.as_str() == "--mode" && value.as_str() == "smoke")
        || matches!(args, [flag] if flag.as_str() == "--mode=smoke")
}

fn usage(detail: &str) -> String {
    let commands = [
        "usage:",
        "  atlas-cli check <file.atlas>",
        "  atlas-cli explain <E_CODE>",
        "  atlas-cli export <file.atlas> --format json",
        "  atlas-cli bench --mode smoke",
    ];
    format!("{}\n{}", detail, commands.join("\n"))
}
