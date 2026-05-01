use crate::{
    bench_report_json, export_json_file, metrics_json_file, p57_report_json_file,
    p58_metrics_json_file, p58_report_json_file, p58_report_markdown_file, run_workload_file,
    validate_file, DiagnosticCode, WorkloadMode,
};
use std::env;

pub fn main() {
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
        "run" => run_command(args),
        "metrics" => metrics_command(args),
        "report" => report_command(args),
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

fn run_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("run requires a .atlas path"))?;
    let mode = parse_required_mode(&args[2..], "run")?;
    let metrics = run_workload_file(path, mode).map_err(|diagnostic| diagnostic.to_string())?;
    println!(
        "OK: runtime {} encoded_segments={} reads={} updates={} \
         snapshots={} rebuilds={} success_rate={:.3}",
        metrics.mode,
        metrics.encoded_segments_total,
        metrics.read_count,
        metrics.update_count,
        metrics.snapshot_count,
        metrics.rebuild_count,
        metrics.query_success_rate
    );
    Ok(())
}

fn metrics_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("metrics requires a .atlas path"))?;
    let options = parse_options(&args[2..], "metrics")?;
    if options.format != Some(OutputFormat::Json) {
        return Err(usage("metrics requires --format json"));
    }
    let json = match options.mode {
        Some(mode) => p58_metrics_json_file(path, mode),
        None => metrics_json_file(path),
    }
    .map_err(|diagnostic| diagnostic.to_string())?;
    println!("{}", json);
    Ok(())
}

fn report_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("report requires a .atlas path"))?;
    let options = parse_options(&args[2..], "report")?;
    let format = options
        .format
        .ok_or_else(|| usage("report requires --format json|markdown"))?;
    match (options.mode, format) {
        (None, OutputFormat::Json) => {
            let json = p57_report_json_file(path).map_err(|diagnostic| diagnostic.to_string())?;
            println!("{}", json);
        }
        (None, OutputFormat::Markdown) => {
            return Err(usage(
                "report --format markdown requires --mode smoke|standard|ambitious",
            ));
        }
        (Some(mode), OutputFormat::Json) => {
            let json =
                p58_report_json_file(path, mode).map_err(|diagnostic| diagnostic.to_string())?;
            println!("{}", json);
        }
        (Some(mode), OutputFormat::Markdown) => {
            let markdown = p58_report_markdown_file(path, mode)
                .map_err(|diagnostic| diagnostic.to_string())?;
            println!("{}", markdown);
        }
    }
    Ok(())
}

fn has_json_format(args: &[String]) -> bool {
    matches!(args, [flag, value] if flag.as_str() == "--format" && value.as_str() == "json")
        || matches!(args, [flag] if flag.as_str() == "--format=json")
}

fn bench_command(args: &[String]) -> Result<(), String> {
    let options = parse_options(&args[1..], "bench")?;
    let mode = options
        .mode
        .ok_or_else(|| usage("bench requires --mode smoke|standard|ambitious"))?;
    let metrics = run_workload_file("examples/p53_strict.atlas", mode)
        .map_err(|diagnostic| diagnostic.to_string())?;
    if let Some(format) = options.format {
        if format != OutputFormat::Json {
            return Err(usage("bench supports --format json"));
        }
        println!("{}", bench_report_json(&metrics));
        return Ok(());
    }
    println!(
        "OK: bench {} encoded_segments={} reads={} updates={} snapshots={} rebuilds={}",
        metrics.mode,
        metrics.encoded_segments_total,
        metrics.read_count,
        metrics.update_count,
        metrics.snapshot_count,
        metrics.rebuild_count
    );
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Json,
    Markdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CommandOptions {
    mode: Option<WorkloadMode>,
    format: Option<OutputFormat>,
}

fn parse_required_mode(args: &[String], command: &str) -> Result<WorkloadMode, String> {
    let mode = match args {
        [flag, value] if flag.as_str() == "--mode" => value.as_str(),
        [flag] if flag.starts_with("--mode=") => flag.trim_start_matches("--mode="),
        _ => {
            return Err(usage(format!(
                "{} requires --mode smoke|standard|ambitious",
                command
            )))
        }
    };
    WorkloadMode::from_str(mode).ok_or_else(|| {
        usage(format!(
            "{} received unsupported mode '{}'; expected smoke|standard|ambitious",
            command, mode
        ))
    })
}

fn parse_options(args: &[String], command: &str) -> Result<CommandOptions, String> {
    let mut mode = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--mode" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage(format!("{} requires a value after --mode", command)))?;
            mode = Some(parse_mode_value(value, command)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--mode=") {
            mode = Some(parse_mode_value(value, command)?);
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage(format!("{} requires a value after --format", command)))?;
            format = Some(parse_format_value(value, command)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, command)?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "{} received unsupported option '{}'",
                command, arg
            )));
        }
    }

    Ok(CommandOptions { mode, format })
}

fn parse_mode_value(value: &str, command: &str) -> Result<WorkloadMode, String> {
    WorkloadMode::from_str(value).ok_or_else(|| {
        usage(format!(
            "{} received unsupported mode '{}'; expected smoke|standard|ambitious",
            command, value
        ))
    })
}

fn parse_format_value(value: &str, command: &str) -> Result<OutputFormat, String> {
    match value {
        "json" => Ok(OutputFormat::Json),
        "markdown" => Ok(OutputFormat::Markdown),
        _ => Err(usage(format!(
            "{} received unsupported format '{}'; expected json|markdown",
            command, value
        ))),
    }
}

fn usage(detail: impl AsRef<str>) -> String {
    let commands = [
        "usage:",
        "  atlas-cli check <file.atlas>",
        "  atlas-cli explain <E_CODE>",
        "  atlas-cli export <file.atlas> --format json",
        "  atlas-cli run <file.atlas> --mode smoke|standard|ambitious",
        "  atlas-cli metrics <file.atlas> [--mode smoke|standard|ambitious] --format json",
        "  atlas-cli report <file.atlas> [--mode smoke|standard|ambitious] --format json|markdown",
        "  atlas-cli bench --mode smoke|standard|ambitious [--format json]",
    ];
    format!("{}\n{}", detail.as_ref(), commands.join("\n"))
}
