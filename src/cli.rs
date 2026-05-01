use crate::{
    bench_report_json, export_json_file, metrics_json_file, p57_report_json_file,
    p58_metrics_json_file, p58_report_json_file, p58_report_markdown_file,
    p61_virtual_ratio_report_json_file, p62_real_ratio_report_json_file_with_runs,
    p63_campaign_compare_json_files, p63_campaign_register_json_file,
    p63_campaign_report_file_with_runs, p63_campaign_report_to_json,
    p63_campaign_set_summary_json_file, p63_campaign_summary_json_file, run_workload_file,
    validate_file, write_p63_campaign_exports, write_p64_campaign_exports, DiagnosticCode,
    P63ThresholdProfile, P64GenerationPolicy, P64RatioRealishOptions, P64WorkloadKind,
    WorkloadMode,
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
        "ratio" => ratio_command(args),
        "ratio-real" => ratio_real_command(args),
        "ratio-campaign-compare" => ratio_campaign_compare_command(args),
        "ratio-campaign-register" => ratio_campaign_register_command(args),
        "ratio-campaign-summary" => ratio_campaign_summary_command(args),
        "ratio-campaign-set-summary" => ratio_campaign_set_summary_command(args),
        "ratio-realish" => ratio_realish_command(args),
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

fn ratio_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("ratio requires a .atlas path"))?;
    let options = parse_options(&args[2..], "ratio")?;
    let mode = options
        .mode
        .ok_or_else(|| usage("ratio requires --mode smoke|standard|ambitious"))?;
    if options.format != Some(OutputFormat::Json) {
        return Err(usage("ratio requires --format json"));
    }
    let json = p61_virtual_ratio_report_json_file(path, mode)
        .map_err(|diagnostic| diagnostic.to_string())?;
    println!("{}", json);
    Ok(())
}

fn ratio_real_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("ratio-real requires a .atlas path"))?;
    let options = parse_options(&args[2..], "ratio-real")?;
    let mode = options
        .mode
        .ok_or_else(|| usage("ratio-real requires --mode smoke|standard|ambitious"))?;
    if options.format != Some(OutputFormat::Json) {
        return Err(usage("ratio-real requires --format json"));
    }
    let runs = options.runs.unwrap_or(1);
    let json = if let Some(threshold_profile) = options.threshold_profile {
        let report = p63_campaign_report_file_with_runs(path, mode, runs, threshold_profile)
            .map_err(|diagnostic| diagnostic.to_string())?;
        if let Some(export_dir) = &options.export_dir {
            write_p63_campaign_exports(&report, export_dir)
                .map_err(|diagnostic| diagnostic.to_string())?;
        }
        p63_campaign_report_to_json(&report)
    } else {
        if options.export_dir.is_some() {
            return Err(usage(
                "ratio-real --export-dir requires --threshold-profile p63",
            ));
        }
        p62_real_ratio_report_json_file_with_runs(path, mode, runs)
            .map_err(|diagnostic| diagnostic.to_string())?
    };
    println!("{}", json);
    Ok(())
}

fn ratio_campaign_compare_command(args: &[String]) -> Result<(), String> {
    let path_a = args
        .get(1)
        .ok_or_else(|| usage("ratio-campaign-compare requires two campaign report paths"))?;
    let path_b = args
        .get(2)
        .ok_or_else(|| usage("ratio-campaign-compare requires two campaign report paths"))?;
    if !has_json_format(&args[3..]) {
        return Err(usage("ratio-campaign-compare requires --format json"));
    }
    let json = p63_campaign_compare_json_files(path_a, path_b)
        .map_err(|diagnostic| diagnostic.to_string())?;
    println!("{}", json);
    Ok(())
}

fn ratio_campaign_register_command(args: &[String]) -> Result<(), String> {
    let report_path = args
        .get(1)
        .ok_or_else(|| usage("ratio-campaign-register requires a campaign report path"))?;
    let options = parse_campaign_register_options(&args[2..])?;
    let json = p63_campaign_register_json_file(report_path, &options.registry, &options.name)
        .map_err(|diagnostic| diagnostic.to_string())?;
    println!("{}", json);
    Ok(())
}

fn ratio_campaign_summary_command(args: &[String]) -> Result<(), String> {
    let registry_path = args
        .get(1)
        .ok_or_else(|| usage("ratio-campaign-summary requires a registry path"))?;
    if !has_json_format(&args[2..]) {
        return Err(usage("ratio-campaign-summary requires --format json"));
    }
    let json = p63_campaign_summary_json_file(registry_path)
        .map_err(|diagnostic| diagnostic.to_string())?;
    println!("{}", json);
    Ok(())
}

fn ratio_campaign_set_summary_command(args: &[String]) -> Result<(), String> {
    let registry_path = args
        .get(1)
        .ok_or_else(|| usage("ratio-campaign-set-summary requires a registry path"))?;
    let options = parse_campaign_set_options(&args[2..])?;
    let json = p63_campaign_set_summary_json_file(
        registry_path,
        &options.set_name,
        Some(options.mode),
        Some(options.threshold_profile),
    )
    .map_err(|diagnostic| diagnostic.to_string())?;
    println!("{}", json);
    Ok(())
}

fn ratio_realish_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("ratio-realish requires a .atlas path"))?;
    let options = parse_p64_options(&args[2..])?;
    let report = crate::p64_ratio_realish_report_file(path, options.options)
        .map_err(|diagnostic| diagnostic.to_string())?;
    if let Some(export_dir) = &options.export_dir {
        write_p64_campaign_exports(&report, export_dir)
            .map_err(|diagnostic| diagnostic.to_string())?;
    }
    match options.format {
        OutputFormat::Json => println!("{}", crate::p64_report_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p64_summary_markdown(&report)),
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Json,
    Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandOptions {
    mode: Option<WorkloadMode>,
    format: Option<OutputFormat>,
    runs: Option<usize>,
    export_dir: Option<String>,
    threshold_profile: Option<P63ThresholdProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CampaignRegisterOptions {
    registry: String,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CampaignSetOptions {
    set_name: String,
    mode: WorkloadMode,
    threshold_profile: P63ThresholdProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct P64CliOptions {
    options: P64RatioRealishOptions,
    export_dir: Option<String>,
    format: OutputFormat,
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
    let mut runs = None;
    let mut export_dir = None;
    let mut threshold_profile = None;
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
        } else if arg == "--runs" && command == "ratio-real" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-real requires a value after --runs"))?;
            runs = Some(parse_runs_value(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            if command != "ratio-real" {
                return Err(usage(format!(
                    "{} received unsupported option '{}'",
                    command, arg
                )));
            }
            runs = Some(parse_runs_value(value)?);
            idx += 1;
        } else if arg == "--export-dir" && command == "ratio-real" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-real requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            if command != "ratio-real" {
                return Err(usage(format!(
                    "{} received unsupported option '{}'",
                    command, arg
                )));
            }
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--threshold-profile" && command == "ratio-real" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-real requires a value after --threshold-profile"))?;
            threshold_profile = Some(parse_threshold_profile_value(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--threshold-profile=") {
            if command != "ratio-real" {
                return Err(usage(format!(
                    "{} received unsupported option '{}'",
                    command, arg
                )));
            }
            threshold_profile = Some(parse_threshold_profile_value(value)?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "{} received unsupported option '{}'",
                command, arg
            )));
        }
    }

    Ok(CommandOptions {
        mode,
        format,
        runs,
        export_dir,
        threshold_profile,
    })
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

fn parse_runs_value(value: &str) -> Result<usize, String> {
    let runs = value
        .parse::<usize>()
        .map_err(|_| usage(format!("ratio-real received invalid --runs '{}'", value)))?;
    if runs == 0 {
        return Err(usage("ratio-real requires --runs greater than zero"));
    }
    Ok(runs)
}

fn parse_threshold_profile_value(value: &str) -> Result<P63ThresholdProfile, String> {
    P63ThresholdProfile::from_str(value).ok_or_else(|| {
        usage(format!(
            "ratio-real received unsupported threshold profile '{}'; expected p63|p63_conservative_v1",
            value
        ))
    })
}

fn parse_campaign_register_options(args: &[String]) -> Result<CampaignRegisterOptions, String> {
    let mut registry = None;
    let mut name = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--registry" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-campaign-register requires a value after --registry")
            })?;
            registry = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--registry=") {
            registry = Some(value.to_string());
            idx += 1;
        } else if arg == "--name" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-campaign-register requires a value after --name"))?;
            name = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--name=") {
            name = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-campaign-register requires a value after --format"))?;
            format = Some(parse_format_value(value, "ratio-campaign-register")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "ratio-campaign-register")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "ratio-campaign-register received unsupported option '{}'",
                arg
            )));
        }
    }

    if format != Some(OutputFormat::Json) {
        return Err(usage("ratio-campaign-register requires --format json"));
    }
    Ok(CampaignRegisterOptions {
        registry: registry
            .ok_or_else(|| usage("ratio-campaign-register requires --registry <path>"))?,
        name: name.ok_or_else(|| usage("ratio-campaign-register requires --name <name>"))?,
    })
}

fn parse_campaign_set_options(args: &[String]) -> Result<CampaignSetOptions, String> {
    let mut set_name = "standard_local_set".to_string();
    let mut mode = None;
    let mut threshold_profile = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--set-name" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-campaign-set-summary requires a value after --set-name")
            })?;
            set_name = value.to_string();
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--set-name=") {
            set_name = value.to_string();
            idx += 1;
        } else if arg == "--mode" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-campaign-set-summary requires a value after --mode"))?;
            mode = Some(parse_mode_value(value, "ratio-campaign-set-summary")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--mode=") {
            mode = Some(parse_mode_value(value, "ratio-campaign-set-summary")?);
            idx += 1;
        } else if arg == "--threshold-profile" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-campaign-set-summary requires a value after --threshold-profile")
            })?;
            threshold_profile = Some(parse_threshold_profile_value(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--threshold-profile=") {
            threshold_profile = Some(parse_threshold_profile_value(value)?);
            idx += 1;
        } else if arg == "--format" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-campaign-set-summary requires a value after --format")
            })?;
            format = Some(parse_format_value(value, "ratio-campaign-set-summary")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "ratio-campaign-set-summary")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "ratio-campaign-set-summary received unsupported option '{}'",
                arg
            )));
        }
    }

    if format != Some(OutputFormat::Json) {
        return Err(usage("ratio-campaign-set-summary requires --format json"));
    }
    Ok(CampaignSetOptions {
        set_name,
        mode: mode.ok_or_else(|| {
            usage("ratio-campaign-set-summary requires --mode smoke|standard|ambitious")
        })?,
        threshold_profile: threshold_profile
            .ok_or_else(|| usage("ratio-campaign-set-summary requires --threshold-profile p63"))?,
    })
}

fn parse_p64_options(args: &[String]) -> Result<P64CliOptions, String> {
    let mut workload = None;
    let mut workload_all = false;
    let mut policy = None;
    let mut policy_all = false;
    let mut mode = None;
    let mut runs = None;
    let mut queries = None;
    let mut neighborhood_radius = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--workload" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-realish requires a value after --workload"))?;
            let parsed = parse_p64_workload_value(value)?;
            workload_all = parsed.is_none();
            workload = parsed;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--workload=") {
            let parsed = parse_p64_workload_value(value)?;
            workload_all = parsed.is_none();
            workload = parsed;
            idx += 1;
        } else if arg == "--policy" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-realish requires a value after --policy"))?;
            let parsed = parse_p64_policy_value(value)?;
            policy_all = parsed.is_none();
            policy = parsed;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--policy=") {
            let parsed = parse_p64_policy_value(value)?;
            policy_all = parsed.is_none();
            policy = parsed;
            idx += 1;
        } else if arg == "--mode" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-realish requires a value after --mode"))?;
            mode = Some(parse_mode_value(value, "ratio-realish")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--mode=") {
            mode = Some(parse_mode_value(value, "ratio-realish")?);
            idx += 1;
        } else if arg == "--runs" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-realish requires a value after --runs"))?;
            runs = Some(parse_positive_usize(value, "ratio-realish", "--runs")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = Some(parse_positive_usize(value, "ratio-realish", "--runs")?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-realish requires a value after --queries"))?;
            queries = Some(parse_positive_usize(value, "ratio-realish", "--queries")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(value, "ratio-realish", "--queries")?);
            idx += 1;
        } else if arg == "--neighborhood-radius" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-realish requires a value after --neighborhood-radius")
            })?;
            neighborhood_radius = Some(parse_positive_usize(
                value,
                "ratio-realish",
                "--neighborhood-radius",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--neighborhood-radius=") {
            neighborhood_radius = Some(parse_positive_usize(
                value,
                "ratio-realish",
                "--neighborhood-radius",
            )?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-realish requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-realish requires a value after --format"))?;
            format = Some(parse_format_value(value, "ratio-realish")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "ratio-realish")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "ratio-realish received unsupported option '{}'",
                arg
            )));
        }
    }

    if !workload_all && workload.is_none() {
        return Err(usage("ratio-realish requires --workload <name>|all"));
    }
    if !policy_all && policy.is_none() {
        return Err(usage("ratio-realish requires --policy <name>|all"));
    }

    Ok(P64CliOptions {
        options: P64RatioRealishOptions {
            workload,
            policy,
            mode: mode
                .ok_or_else(|| usage("ratio-realish requires --mode smoke|standard|ambitious"))?,
            runs: runs.ok_or_else(|| usage("ratio-realish requires --runs N"))?,
            queries: queries.ok_or_else(|| usage("ratio-realish requires --queries N"))?,
            neighborhood_radius: neighborhood_radius
                .ok_or_else(|| usage("ratio-realish requires --neighborhood-radius N"))?,
        },
        export_dir,
        format: format.ok_or_else(|| usage("ratio-realish requires --format json|markdown"))?,
    })
}

fn parse_p64_workload_value(value: &str) -> Result<Option<P64WorkloadKind>, String> {
    if value == "all" {
        return Ok(None);
    }
    P64WorkloadKind::from_str(value).map(Some).ok_or_else(|| {
        usage(format!(
            "ratio-realish received unsupported workload '{}'; expected realish_log_events|realish_sparse_csv|realish_json_records|realish_hybrid_field_fixture|all",
            value
        ))
    })
}

fn parse_p64_policy_value(value: &str) -> Result<Option<P64GenerationPolicy>, String> {
    if value == "all" {
        return Ok(None);
    }
    P64GenerationPolicy::from_str(value).map(Some).ok_or_else(|| {
        usage(format!(
            "ratio-realish received unsupported policy '{}'; expected full-materialization|global-indexed|address-local|all",
            value
        ))
    })
}

fn parse_positive_usize(value: &str, command: &str, option: &str) -> Result<usize, String> {
    let parsed = value.parse::<usize>().map_err(|_| {
        usage(format!(
            "{} received invalid {} '{}'",
            command, option, value
        ))
    })?;
    if parsed == 0 {
        return Err(usage(format!(
            "{} requires {} greater than zero",
            command, option
        )));
    }
    Ok(parsed)
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
        "  atlas-cli ratio <file.atlas> --mode smoke|standard|ambitious --format json",
        "  atlas-cli ratio-real <file.atlas> --mode smoke|standard|ambitious --format json [--runs N] [--export-dir <path> --threshold-profile p63]",
        "  atlas-cli ratio-campaign-compare <campaign-a.json> <campaign-b.json> --format json",
        "  atlas-cli ratio-campaign-register <campaign.json> --registry <registry.json> --name <campaign-name> --format json",
        "  atlas-cli ratio-campaign-summary <registry.json> --format json",
        "  atlas-cli ratio-campaign-set-summary <registry.json> --mode smoke|standard|ambitious --threshold-profile p63 --format json [--set-name <name>]",
        "  atlas-cli ratio-realish <file.atlas> --workload realish_log_events|realish_sparse_csv|realish_json_records|realish_hybrid_field_fixture|all --policy full-materialization|global-indexed|address-local|all --mode smoke|standard|ambitious --runs N --queries N --neighborhood-radius N [--export-dir <path>] --format json|markdown",
    ];
    format!("{}\n{}", detail.as_ref(), commands.join("\n"))
}
