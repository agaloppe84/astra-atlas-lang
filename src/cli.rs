use crate::{
    bench_report_json, export_json_file, metrics_json_file, p57_report_json_file,
    p58_metrics_json_file, p58_report_json_file, p58_report_markdown_file,
    p61_virtual_ratio_report_json_file, p62_real_ratio_report_json_file_with_runs,
    p63_campaign_compare_json_files, p63_campaign_register_json_file,
    p63_campaign_report_file_with_runs, p63_campaign_report_to_json,
    p63_campaign_set_summary_json_file, p63_campaign_summary_json_file, run_workload_file,
    validate_file, write_p63_campaign_exports, write_p64_campaign_exports, DiagnosticCode,
    FiberGenerationStrategy, P63ThresholdProfile, P64GenerationPolicy, P64RatioRealishOptions,
    P64WorkloadKind, P65ActorCalibrationOptions, P65ActorStrategy, P65JournalPolicy,
    P65QueryLocality, P65RatioActorsOptions, P66JournalPolicy, P66RateProfile,
    P66RatioFibersOptions, P67AuditPolicy, P67CachePolicy, P67CompactionPolicy,
    P67FiberCalibrationOptions, P67FiberProjectionDepth, P67QueryLocality, P68PromotionOptions,
    P69ContractRunOptions, P70ContractReplayOptions, P70ReplayFixtureKind, P71FiberStoreOptions,
    RealDataCorpusKind, WorkloadMode,
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
        "ratio-actors" => ratio_actors_command(args),
        "ratio-actors-calibrate" => ratio_actors_calibrate_command(args),
        "ratio-fibers" => ratio_fibers_command(args),
        "ratio-fibers-calibrate" => ratio_fibers_calibrate_command(args),
        "ratio-fibers-promote" => ratio_fibers_promote_command(args),
        "contract-check" => contract_check_command(args),
        "contract-run" => contract_run_command(args),
        "contract-replay" => contract_replay_command(args),
        "fiber-store-bench" => fiber_store_bench_command(args),
        path if args.len() == 1 => check_path(path),
        _ => Err(usage("unknown command")),
    }
}

fn check_path(path: &str) -> Result<(), String> {
    if crate::p69_contract_file_looks_like(path) {
        return match crate::p69_contract_report_file(path) {
            Ok(report) => {
                println!(
                    "OK: p69_contract={} architecture={} all_storage_counted={}",
                    report.contract_id, report.architecture_id, report.all_storage_counted
                );
                Ok(())
            }
            Err(diagnostic) => Err(diagnostic.to_string()),
        };
    }
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

fn ratio_actors_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("ratio-actors requires a .atlas path"))?;
    let options = parse_p65_options(&args[2..])?;
    let report = crate::p65_ratio_actors_report_file(path, options.options)
        .map_err(|diagnostic| diagnostic.to_string())?;
    if let Some(export_dir) = &options.export_dir {
        crate::write_p65_actor_campaign_exports(&report, export_dir)
            .map_err(|diagnostic| diagnostic.to_string())?;
    }
    match options.format {
        OutputFormat::Json => println!("{}", crate::p65_report_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p65_summary_markdown(&report)),
    }
    Ok(())
}

fn ratio_actors_calibrate_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("ratio-actors-calibrate requires a .atlas path"))?;
    let options = parse_p65_calibration_options(&args[2..])?;
    let report = crate::p65_actor_calibration_report_file(path, options.options)
        .map_err(|diagnostic| diagnostic.to_string())?;
    if let Some(export_dir) = &options.export_dir {
        crate::write_p65_actor_calibration_exports(&report, export_dir)
            .map_err(|diagnostic| diagnostic.to_string())?;
    }
    match options.format {
        OutputFormat::Json => println!("{}", crate::p65_actor_calibration_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p65_actor_calibration_markdown(&report)),
    }
    Ok(())
}

fn ratio_fibers_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("ratio-fibers requires a .atlas path"))?;
    let options = parse_p66_options(&args[2..])?;
    let report = crate::p66_ratio_fibers_report_file(path, options.options)
        .map_err(|diagnostic| diagnostic.to_string())?;
    if let Some(export_dir) = &options.export_dir {
        crate::write_p66_fiber_campaign_exports(&report, export_dir)
            .map_err(|diagnostic| diagnostic.to_string())?;
    }
    match options.format {
        OutputFormat::Json => println!("{}", crate::p66_report_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p66_summary_markdown(&report)),
    }
    Ok(())
}

fn ratio_fibers_calibrate_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("ratio-fibers-calibrate requires a .atlas path"))?;
    let options = parse_p67_calibration_options(&args[2..])?;
    let report = crate::p67_fiber_calibration_report_file(path, options.options)
        .map_err(|diagnostic| diagnostic.to_string())?;
    if let Some(export_dir) = &options.export_dir {
        crate::write_p67_fiber_calibration_exports(&report, export_dir)
            .map_err(|diagnostic| diagnostic.to_string())?;
    }
    match options.format {
        OutputFormat::Json => println!("{}", crate::p67_fiber_calibration_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p67_fiber_calibration_markdown(&report)),
    }
    Ok(())
}

fn ratio_fibers_promote_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("ratio-fibers-promote requires a .atlas path"))?;
    let options = parse_p68_promotion_options(&args[2..])?;
    let report = crate::p68_promotion_report_file(path, options.options)
        .map_err(|diagnostic| diagnostic.to_string())?;
    if let Some(export_dir) = &options.export_dir {
        crate::write_p68_promotion_exports(&report, export_dir)
            .map_err(|diagnostic| diagnostic.to_string())?;
    }
    match options.format {
        OutputFormat::Json => println!("{}", crate::p68_promotion_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p68_promotion_markdown(&report)),
    }
    Ok(())
}

fn contract_check_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("contract-check requires a .atlas path"))?;
    if !has_json_format(&args[2..]) {
        return Err(usage("contract-check requires --format json"));
    }
    let json =
        crate::p69_contract_check_json_file(path).map_err(|diagnostic| diagnostic.to_string())?;
    println!("{}", json);
    Ok(())
}

fn contract_run_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("contract-run requires a .atlas path"))?;
    let options = parse_p69_contract_run_options(&args[2..])?;
    let report = crate::p69_contract_run_report_file(path, options.options)
        .map_err(|diagnostic| diagnostic.to_string())?;
    if let Some(export_dir) = &options.export_dir {
        crate::write_p69_contract_exports(&report, export_dir)
            .map_err(|diagnostic| diagnostic.to_string())?;
    }
    match options.format {
        OutputFormat::Json => println!("{}", crate::p69_contract_report_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p69_contract_summary_markdown(&report)),
    }
    Ok(())
}

fn contract_replay_command(args: &[String]) -> Result<(), String> {
    let path = args
        .get(1)
        .ok_or_else(|| usage("contract-replay requires a .atlas path"))?;
    let options = parse_p70_contract_replay_options(&args[2..])?;
    let report = crate::p70_contract_replay_report_file(path, options.options)
        .map_err(|diagnostic| diagnostic.to_string())?;
    if let Some(export_dir) = &options.export_dir {
        crate::write_p70_contract_replay_exports(&report, export_dir)
            .map_err(|diagnostic| diagnostic.to_string())?;
    }
    match options.format {
        OutputFormat::Json => println!("{}", crate::p70_contract_replay_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p70_contract_replay_markdown(&report)),
    }
    Ok(())
}

fn fiber_store_bench_command(args: &[String]) -> Result<(), String> {
    let options = parse_p71_fiber_store_options(&args[1..])?;
    let report = crate::p71_fiber_store_bench(options.options, &options.export_dir)
        .map_err(|diagnostic| diagnostic.to_string())?;
    match options.format {
        OutputFormat::Json => println!("{}", crate::p71_fiber_store_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p71_fiber_store_markdown(&report)),
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct P65CliOptions {
    options: P65RatioActorsOptions,
    export_dir: Option<String>,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct P65CalibrationCliOptions {
    options: P65ActorCalibrationOptions,
    export_dir: Option<String>,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct P66CliOptions {
    options: P66RatioFibersOptions,
    export_dir: Option<String>,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct P67CalibrationCliOptions {
    options: P67FiberCalibrationOptions,
    export_dir: Option<String>,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct P68PromotionCliOptions {
    options: P68PromotionOptions,
    export_dir: Option<String>,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct P69ContractRunCliOptions {
    options: P69ContractRunOptions,
    export_dir: Option<String>,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
struct P70ContractReplayCliOptions {
    options: P70ContractReplayOptions,
    export_dir: Option<String>,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
struct P71FiberStoreCliOptions {
    options: P71FiberStoreOptions,
    export_dir: String,
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

fn parse_p65_options(args: &[String]) -> Result<P65CliOptions, String> {
    let mut workload = None;
    let mut workload_all = false;
    let mut actor_strategy = None;
    let mut actor_strategy_all = false;
    let mut mode = None;
    let mut runs = None;
    let mut queries = None;
    let mut neighborhood_radius = None;
    let mut budget_bytes = None;
    let mut cache_enabled = true;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--workload" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors requires a value after --workload"))?;
            let parsed = parse_p64_workload_value(value)?;
            workload_all = parsed.is_none();
            workload = parsed;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--workload=") {
            let parsed = parse_p64_workload_value(value)?;
            workload_all = parsed.is_none();
            workload = parsed;
            idx += 1;
        } else if arg == "--actor-strategy" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors requires a value after --actor-strategy"))?;
            let parsed = parse_p65_actor_strategy_value(value)?;
            actor_strategy_all = parsed.is_none();
            actor_strategy = parsed;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--actor-strategy=") {
            let parsed = parse_p65_actor_strategy_value(value)?;
            actor_strategy_all = parsed.is_none();
            actor_strategy = parsed;
            idx += 1;
        } else if arg == "--mode" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors requires a value after --mode"))?;
            mode = Some(parse_mode_value(value, "ratio-actors")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--mode=") {
            mode = Some(parse_mode_value(value, "ratio-actors")?);
            idx += 1;
        } else if arg == "--runs" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors requires a value after --runs"))?;
            runs = Some(parse_positive_usize(value, "ratio-actors", "--runs")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = Some(parse_positive_usize(value, "ratio-actors", "--runs")?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors requires a value after --queries"))?;
            queries = Some(parse_positive_usize(value, "ratio-actors", "--queries")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(value, "ratio-actors", "--queries")?);
            idx += 1;
        } else if arg == "--neighborhood-radius" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-actors requires a value after --neighborhood-radius")
            })?;
            neighborhood_radius = Some(parse_positive_usize(
                value,
                "ratio-actors",
                "--neighborhood-radius",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--neighborhood-radius=") {
            neighborhood_radius = Some(parse_positive_usize(
                value,
                "ratio-actors",
                "--neighborhood-radius",
            )?);
            idx += 1;
        } else if arg == "--budget-bytes" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors requires a value after --budget-bytes"))?;
            budget_bytes = Some(parse_positive_u64(value, "ratio-actors", "--budget-bytes")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--budget-bytes=") {
            budget_bytes = Some(parse_positive_u64(value, "ratio-actors", "--budget-bytes")?);
            idx += 1;
        } else if arg == "--cache" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors requires a value after --cache"))?;
            cache_enabled = parse_cache_value(value)?;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cache=") {
            cache_enabled = parse_cache_value(value)?;
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors requires a value after --format"))?;
            format = Some(parse_format_value(value, "ratio-actors")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "ratio-actors")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "ratio-actors received unsupported option '{}'",
                arg
            )));
        }
    }

    if !workload_all && workload.is_none() {
        return Err(usage("ratio-actors requires --workload <name>|all"));
    }
    if !actor_strategy_all && actor_strategy.is_none() {
        return Err(usage("ratio-actors requires --actor-strategy <name>|all"));
    }

    Ok(P65CliOptions {
        options: P65RatioActorsOptions {
            workload,
            actor_strategy,
            mode: mode
                .ok_or_else(|| usage("ratio-actors requires --mode smoke|standard|ambitious"))?,
            runs: runs.ok_or_else(|| usage("ratio-actors requires --runs N"))?,
            queries: queries.ok_or_else(|| usage("ratio-actors requires --queries N"))?,
            neighborhood_radius: neighborhood_radius
                .ok_or_else(|| usage("ratio-actors requires --neighborhood-radius N"))?,
            budget_bytes: budget_bytes
                .ok_or_else(|| usage("ratio-actors requires --budget-bytes N"))?,
            cache_enabled,
        },
        export_dir,
        format: format.ok_or_else(|| usage("ratio-actors requires --format json|markdown"))?,
    })
}

fn parse_p65_actor_strategy_value(value: &str) -> Result<Option<P65ActorStrategy>, String> {
    if value == "all" {
        return Ok(None);
    }
    P65ActorStrategy::from_str(value).map(Some).ok_or_else(|| {
        usage(format!(
            "ratio-actors received unsupported actor strategy '{}'; expected no-actor|single-local|specialized-crud|over-agentic|all",
            value
        ))
    })
}

fn parse_p66_options(args: &[String]) -> Result<P66CliOptions, String> {
    let mut workload = None;
    let mut workload_all = false;
    let mut fiber_strategy = None;
    let mut fiber_strategy_all = false;
    let mut mode = None;
    let mut runs = None;
    let mut queries = None;
    let mut neighborhood_radius = None;
    let mut budget_bytes = None;
    let mut cache_enabled = true;
    let mut journal_policy = None;
    let mut update_rate = None;
    let mut audit_rate = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--workload" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --workload"))?;
            let parsed = parse_p64_workload_value(value)?;
            workload_all = parsed.is_none();
            workload = parsed;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--workload=") {
            let parsed = parse_p64_workload_value(value)?;
            workload_all = parsed.is_none();
            workload = parsed;
            idx += 1;
        } else if arg == "--fiber-strategy" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --fiber-strategy"))?;
            let parsed = parse_p66_fiber_strategy_value(value)?;
            fiber_strategy_all = parsed.is_none();
            fiber_strategy = parsed;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--fiber-strategy=") {
            let parsed = parse_p66_fiber_strategy_value(value)?;
            fiber_strategy_all = parsed.is_none();
            fiber_strategy = parsed;
            idx += 1;
        } else if arg == "--mode" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --mode"))?;
            mode = Some(parse_mode_value(value, "ratio-fibers")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--mode=") {
            mode = Some(parse_mode_value(value, "ratio-fibers")?);
            idx += 1;
        } else if arg == "--runs" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --runs"))?;
            runs = Some(parse_positive_usize(value, "ratio-fibers", "--runs")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = Some(parse_positive_usize(value, "ratio-fibers", "--runs")?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --queries"))?;
            queries = Some(parse_positive_usize(value, "ratio-fibers", "--queries")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(value, "ratio-fibers", "--queries")?);
            idx += 1;
        } else if arg == "--neighborhood-radius" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-fibers requires a value after --neighborhood-radius")
            })?;
            neighborhood_radius = Some(parse_positive_usize(
                value,
                "ratio-fibers",
                "--neighborhood-radius",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--neighborhood-radius=") {
            neighborhood_radius = Some(parse_positive_usize(
                value,
                "ratio-fibers",
                "--neighborhood-radius",
            )?);
            idx += 1;
        } else if arg == "--budget-bytes" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --budget-bytes"))?;
            budget_bytes = Some(parse_positive_u64(value, "ratio-fibers", "--budget-bytes")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--budget-bytes=") {
            budget_bytes = Some(parse_positive_u64(value, "ratio-fibers", "--budget-bytes")?);
            idx += 1;
        } else if arg == "--cache" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --cache"))?;
            cache_enabled = parse_cache_value(value)?;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cache=") {
            cache_enabled = parse_cache_value(value)?;
            idx += 1;
        } else if arg == "--journal" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --journal"))?;
            journal_policy = Some(parse_p66_journal_value(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--journal=") {
            journal_policy = Some(parse_p66_journal_value(value)?);
            idx += 1;
        } else if arg == "--update-rate" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --update-rate"))?;
            update_rate = Some(parse_p66_rate_value(value, "--update-rate")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--update-rate=") {
            update_rate = Some(parse_p66_rate_value(value, "--update-rate")?);
            idx += 1;
        } else if arg == "--audit-rate" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --audit-rate"))?;
            audit_rate = Some(parse_p66_rate_value(value, "--audit-rate")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--audit-rate=") {
            audit_rate = Some(parse_p66_rate_value(value, "--audit-rate")?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers requires a value after --format"))?;
            format = Some(parse_format_value(value, "ratio-fibers")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "ratio-fibers")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "ratio-fibers received unsupported option '{}'",
                arg
            )));
        }
    }

    if !workload_all && workload.is_none() {
        return Err(usage("ratio-fibers requires --workload <name>|all"));
    }
    if !fiber_strategy_all && fiber_strategy.is_none() {
        return Err(usage("ratio-fibers requires --fiber-strategy <name>|all"));
    }

    Ok(P66CliOptions {
        options: P66RatioFibersOptions {
            workload,
            fiber_strategy,
            mode: mode
                .ok_or_else(|| usage("ratio-fibers requires --mode smoke|standard|ambitious"))?,
            runs: runs.ok_or_else(|| usage("ratio-fibers requires --runs N"))?,
            queries: queries.ok_or_else(|| usage("ratio-fibers requires --queries N"))?,
            neighborhood_radius: neighborhood_radius
                .ok_or_else(|| usage("ratio-fibers requires --neighborhood-radius N"))?,
            budget_bytes: budget_bytes
                .ok_or_else(|| usage("ratio-fibers requires --budget-bytes N"))?,
            cache_enabled,
            journal_policy: journal_policy
                .ok_or_else(|| usage("ratio-fibers requires --journal eager|lazy|compact"))?,
            update_rate,
            audit_rate,
        },
        export_dir,
        format: format.ok_or_else(|| usage("ratio-fibers requires --format json|markdown"))?,
    })
}

fn parse_p66_fiber_strategy_value(value: &str) -> Result<Option<FiberGenerationStrategy>, String> {
    if value == "all" {
        return Ok(None);
    }
    FiberGenerationStrategy::from_str(value)
        .map(Some)
        .ok_or_else(|| {
            usage(format!(
                "ratio-fibers received unsupported fiber strategy '{}'; expected point-fiber|neighborhood-fiber|actor-fiber|actor-neighborhood-fiber|all",
                value
            ))
        })
}

fn parse_p66_journal_value(value: &str) -> Result<P66JournalPolicy, String> {
    P66JournalPolicy::from_str(value).ok_or_else(|| {
        usage(format!(
            "ratio-fibers received unsupported journal policy '{}'; expected eager|lazy|compact",
            value
        ))
    })
}

fn parse_p66_rate_value(value: &str, option: &str) -> Result<P66RateProfile, String> {
    P66RateProfile::from_str(value).ok_or_else(|| {
        usage(format!(
            "ratio-fibers received unsupported {} '{}'; expected low|medium|high",
            option, value
        ))
    })
}

fn parse_p67_calibration_options(args: &[String]) -> Result<P67CalibrationCliOptions, String> {
    let mut workload = None;
    let mut workload_all = false;
    let mut mode = None;
    let mut runs = None;
    let mut queries = None;
    let mut radius_grid = None;
    let mut budget_grid = None;
    let mut cache_grid = None;
    let mut journal_grid = None;
    let mut audit_grid = None;
    let mut compaction_grid = None;
    let mut query_locality_grid = None;
    let mut fiber_projection_grid = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--workload" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers-calibrate requires a value after --workload"))?;
            let parsed = parse_p64_workload_value(value)?;
            workload_all = parsed.is_none();
            workload = parsed;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--workload=") {
            let parsed = parse_p64_workload_value(value)?;
            workload_all = parsed.is_none();
            workload = parsed;
            idx += 1;
        } else if arg == "--mode" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers-calibrate requires a value after --mode"))?;
            mode = Some(parse_mode_value(value, "ratio-fibers-calibrate")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--mode=") {
            mode = Some(parse_mode_value(value, "ratio-fibers-calibrate")?);
            idx += 1;
        } else if arg == "--runs" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers-calibrate requires a value after --runs"))?;
            runs = Some(parse_positive_usize(
                value,
                "ratio-fibers-calibrate",
                "--runs",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = Some(parse_positive_usize(
                value,
                "ratio-fibers-calibrate",
                "--runs",
            )?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers-calibrate requires a value after --queries"))?;
            queries = Some(parse_positive_usize(
                value,
                "ratio-fibers-calibrate",
                "--queries",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(
                value,
                "ratio-fibers-calibrate",
                "--queries",
            )?);
            idx += 1;
        } else if arg == "--radius-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-fibers-calibrate requires a value after --radius-grid")
            })?;
            radius_grid = Some(parse_p67_usize_grid(value, "radius-grid")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--radius-grid=") {
            radius_grid = Some(parse_p67_usize_grid(value, "radius-grid")?);
            idx += 1;
        } else if arg == "--budget-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-fibers-calibrate requires a value after --budget-grid")
            })?;
            budget_grid = Some(parse_p67_u64_grid(value, "budget-grid")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--budget-grid=") {
            budget_grid = Some(parse_p67_u64_grid(value, "budget-grid")?);
            idx += 1;
        } else if arg == "--cache-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-fibers-calibrate requires a value after --cache-grid")
            })?;
            cache_grid = Some(parse_p67_cache_grid(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cache-grid=") {
            cache_grid = Some(parse_p67_cache_grid(value)?);
            idx += 1;
        } else if arg == "--journal-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-fibers-calibrate requires a value after --journal-grid")
            })?;
            journal_grid = Some(parse_p67_journal_grid(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--journal-grid=") {
            journal_grid = Some(parse_p67_journal_grid(value)?);
            idx += 1;
        } else if arg == "--audit-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-fibers-calibrate requires a value after --audit-grid")
            })?;
            audit_grid = Some(parse_p67_audit_grid(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--audit-grid=") {
            audit_grid = Some(parse_p67_audit_grid(value)?);
            idx += 1;
        } else if arg == "--compaction-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-fibers-calibrate requires a value after --compaction-grid")
            })?;
            compaction_grid = Some(parse_p67_compaction_grid(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--compaction-grid=") {
            compaction_grid = Some(parse_p67_compaction_grid(value)?);
            idx += 1;
        } else if arg == "--query-locality-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-fibers-calibrate requires a value after --query-locality-grid")
            })?;
            query_locality_grid = Some(parse_p67_query_locality_grid(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--query-locality-grid=") {
            query_locality_grid = Some(parse_p67_query_locality_grid(value)?);
            idx += 1;
        } else if arg == "--fiber-projection-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-fibers-calibrate requires a value after --fiber-projection-grid")
            })?;
            fiber_projection_grid = Some(parse_p67_projection_grid(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--fiber-projection-grid=") {
            fiber_projection_grid = Some(parse_p67_projection_grid(value)?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-fibers-calibrate requires a value after --export-dir")
            })?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers-calibrate requires a value after --format"))?;
            format = Some(parse_format_value(value, "ratio-fibers-calibrate")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "ratio-fibers-calibrate")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "ratio-fibers-calibrate received unsupported option '{}'",
                arg
            )));
        }
    }

    if !workload_all && workload.is_none() {
        return Err(usage(
            "ratio-fibers-calibrate requires --workload <name>|all",
        ));
    }

    Ok(P67CalibrationCliOptions {
        options: P67FiberCalibrationOptions {
            workload,
            mode: mode.ok_or_else(|| {
                usage("ratio-fibers-calibrate requires --mode standard|ambitious")
            })?,
            runs: runs.ok_or_else(|| usage("ratio-fibers-calibrate requires --runs N"))?,
            queries: queries.ok_or_else(|| usage("ratio-fibers-calibrate requires --queries N"))?,
            radius_grid: radius_grid
                .ok_or_else(|| usage("ratio-fibers-calibrate requires --radius-grid"))?,
            budget_grid: budget_grid
                .ok_or_else(|| usage("ratio-fibers-calibrate requires --budget-grid"))?,
            cache_grid: cache_grid
                .ok_or_else(|| usage("ratio-fibers-calibrate requires --cache-grid"))?,
            journal_grid: journal_grid
                .ok_or_else(|| usage("ratio-fibers-calibrate requires --journal-grid"))?,
            audit_grid: audit_grid
                .ok_or_else(|| usage("ratio-fibers-calibrate requires --audit-grid"))?,
            compaction_grid: compaction_grid
                .ok_or_else(|| usage("ratio-fibers-calibrate requires --compaction-grid"))?,
            query_locality_grid: query_locality_grid
                .ok_or_else(|| usage("ratio-fibers-calibrate requires --query-locality-grid"))?,
            fiber_projection_grid: fiber_projection_grid
                .ok_or_else(|| usage("ratio-fibers-calibrate requires --fiber-projection-grid"))?,
        },
        export_dir,
        format: format
            .ok_or_else(|| usage("ratio-fibers-calibrate requires --format json|markdown"))?,
    })
}

fn parse_p67_usize_grid(value: &str, name: &str) -> Result<Vec<usize>, String> {
    let parsed: Result<Vec<_>, _> = value
        .split(',')
        .map(|item| parse_positive_usize(item.trim(), "ratio-fibers-calibrate", name))
        .collect();
    let parsed = parsed?;
    if parsed.is_empty() {
        return Err(usage(format!(
            "ratio-fibers-calibrate requires non-empty {}",
            name
        )));
    }
    Ok(parsed)
}

fn parse_p67_u64_grid(value: &str, name: &str) -> Result<Vec<u64>, String> {
    let parsed: Result<Vec<_>, _> = value
        .split(',')
        .map(|item| parse_positive_u64(item.trim(), "ratio-fibers-calibrate", name))
        .collect();
    let parsed = parsed?;
    if parsed.is_empty() {
        return Err(usage(format!(
            "ratio-fibers-calibrate requires non-empty {}",
            name
        )));
    }
    Ok(parsed)
}

fn parse_p67_cache_grid(value: &str) -> Result<Vec<P67CachePolicy>, String> {
    value
        .split(',')
        .map(|item| {
            P67CachePolicy::from_str(item.trim()).ok_or_else(|| {
                usage(format!(
                    "ratio-fibers-calibrate received unsupported cache policy '{}'; expected off|on|compact",
                    item.trim()
                ))
            })
        })
        .collect()
}

fn parse_p67_journal_grid(value: &str) -> Result<Vec<P66JournalPolicy>, String> {
    value
        .split(',')
        .map(|item| {
            P66JournalPolicy::from_str(item.trim()).ok_or_else(|| {
                usage(format!(
                    "ratio-fibers-calibrate received unsupported journal policy '{}'; expected eager|lazy|compact",
                    item.trim()
                ))
            })
        })
        .collect()
}

fn parse_p67_audit_grid(value: &str) -> Result<Vec<P67AuditPolicy>, String> {
    value
        .split(',')
        .map(|item| {
            P67AuditPolicy::from_str(item.trim()).ok_or_else(|| {
                usage(format!(
                    "ratio-fibers-calibrate received unsupported audit policy '{}'; expected minimal|sampled|full",
                    item.trim()
                ))
            })
        })
        .collect()
}

fn parse_p67_compaction_grid(value: &str) -> Result<Vec<P67CompactionPolicy>, String> {
    value
        .split(',')
        .map(|item| {
            P67CompactionPolicy::from_str(item.trim()).ok_or_else(|| {
                usage(format!(
                    "ratio-fibers-calibrate received unsupported compaction policy '{}'; expected off|threshold|aggressive",
                    item.trim()
                ))
            })
        })
        .collect()
}

fn parse_p67_query_locality_grid(value: &str) -> Result<Vec<P67QueryLocality>, String> {
    value
        .split(',')
        .map(|item| {
            P67QueryLocality::from_str(item.trim()).ok_or_else(|| {
                usage(format!(
                    "ratio-fibers-calibrate received unsupported query locality '{}'; expected clustered|random|mixed",
                    item.trim()
                ))
            })
        })
        .collect()
}

fn parse_p67_projection_grid(value: &str) -> Result<Vec<P67FiberProjectionDepth>, String> {
    value
        .split(',')
        .map(|item| {
            P67FiberProjectionDepth::from_str(item.trim()).ok_or_else(|| {
                usage(format!(
                    "ratio-fibers-calibrate received unsupported fiber projection depth '{}'; expected shallow|medium|full",
                    item.trim()
                ))
            })
        })
        .collect()
}

fn parse_p68_promotion_options(args: &[String]) -> Result<P68PromotionCliOptions, String> {
    let mut run_ablations = false;
    let mut run_stress = false;
    let mut phase_map = false;
    let mut strict = true;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--run-ablations" {
            run_ablations = true;
            idx += 1;
        } else if arg == "--run-stress" {
            run_stress = true;
            idx += 1;
        } else if arg == "--phase-map" {
            phase_map = true;
            idx += 1;
        } else if arg == "--strict" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers-promote requires a value after --strict"))?;
            strict = parse_bool_value(value, "ratio-fibers-promote", "--strict")?;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--strict=") {
            strict = parse_bool_value(value, "ratio-fibers-promote", "--strict")?;
            idx += 1;
        } else if arg == "--mode-pair" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers-promote requires a value after --mode-pair"))?;
            parse_p68_mode_pair(value)?;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--mode-pair=") {
            parse_p68_mode_pair(value)?;
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers-promote requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-fibers-promote requires a value after --format"))?;
            format = Some(parse_format_value(value, "ratio-fibers-promote")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "ratio-fibers-promote")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "ratio-fibers-promote received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P68PromotionCliOptions {
        options: P68PromotionOptions {
            run_ablations,
            run_stress,
            phase_map,
            strict,
        },
        export_dir,
        format: format
            .ok_or_else(|| usage("ratio-fibers-promote requires --format json|markdown"))?,
    })
}

fn parse_p68_mode_pair(value: &str) -> Result<(), String> {
    if value == "standard,ambitious" {
        Ok(())
    } else {
        Err(usage(format!(
            "ratio-fibers-promote supports --mode-pair standard,ambitious, got '{}'",
            value
        )))
    }
}

fn parse_p69_contract_run_options(args: &[String]) -> Result<P69ContractRunCliOptions, String> {
    let mut mode = None;
    let mut runs = None;
    let mut queries = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--mode" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-run requires a value after --mode"))?;
            mode = Some(parse_mode_value(value, "contract-run")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--mode=") {
            mode = Some(parse_mode_value(value, "contract-run")?);
            idx += 1;
        } else if arg == "--runs" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-run requires a value after --runs"))?;
            runs = Some(parse_positive_usize(value, "contract-run", "--runs")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = Some(parse_positive_usize(value, "contract-run", "--runs")?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-run requires a value after --queries"))?;
            queries = Some(parse_positive_usize(value, "contract-run", "--queries")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(value, "contract-run", "--queries")?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-run requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-run requires a value after --format"))?;
            format = Some(parse_format_value(value, "contract-run")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "contract-run")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "contract-run received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P69ContractRunCliOptions {
        options: P69ContractRunOptions {
            mode: mode
                .ok_or_else(|| usage("contract-run requires --mode smoke|standard|ambitious"))?,
            runs: runs.ok_or_else(|| usage("contract-run requires --runs N"))?,
            queries: queries.ok_or_else(|| usage("contract-run requires --queries N"))?,
        },
        export_dir,
        format: format.ok_or_else(|| usage("contract-run requires --format json|markdown"))?,
    })
}

fn parse_p70_contract_replay_options(
    args: &[String],
) -> Result<P70ContractReplayCliOptions, String> {
    let mut fixtures = None;
    let mut mode = None;
    let mut runs = None;
    let mut queries = None;
    let mut tolerance_percent = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--fixtures" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-replay requires a value after --fixtures"))?;
            fixtures = Some(parse_p70_fixtures(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--fixtures=") {
            fixtures = Some(parse_p70_fixtures(value)?);
            idx += 1;
        } else if arg == "--mode" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-replay requires a value after --mode"))?;
            mode = Some(parse_mode_value(value, "contract-replay")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--mode=") {
            mode = Some(parse_mode_value(value, "contract-replay")?);
            idx += 1;
        } else if arg == "--runs" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-replay requires a value after --runs"))?;
            runs = Some(parse_positive_usize(value, "contract-replay", "--runs")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = Some(parse_positive_usize(value, "contract-replay", "--runs")?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-replay requires a value after --queries"))?;
            queries = Some(parse_positive_usize(value, "contract-replay", "--queries")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(value, "contract-replay", "--queries")?);
            idx += 1;
        } else if arg == "--tolerance-percent" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("contract-replay requires a value after --tolerance-percent")
            })?;
            tolerance_percent = Some(parse_positive_f64(
                value,
                "contract-replay",
                "--tolerance-percent",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--tolerance-percent=") {
            tolerance_percent = Some(parse_positive_f64(
                value,
                "contract-replay",
                "--tolerance-percent",
            )?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-replay requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("contract-replay requires a value after --format"))?;
            format = Some(parse_format_value(value, "contract-replay")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "contract-replay")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "contract-replay received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P70ContractReplayCliOptions {
        options: P70ContractReplayOptions {
            fixtures: fixtures.ok_or_else(|| {
                usage("contract-replay requires --fixtures all|log|sparse|json|hybrid")
            })?,
            mode: mode
                .ok_or_else(|| usage("contract-replay requires --mode smoke|standard|ambitious"))?,
            runs: runs.ok_or_else(|| usage("contract-replay requires --runs N"))?,
            queries: queries.ok_or_else(|| usage("contract-replay requires --queries N"))?,
            tolerance_percent: tolerance_percent
                .ok_or_else(|| usage("contract-replay requires --tolerance-percent N"))?,
        },
        export_dir,
        format: format.ok_or_else(|| usage("contract-replay requires --format json|markdown"))?,
    })
}

fn parse_p70_fixtures(value: &str) -> Result<Vec<P70ReplayFixtureKind>, String> {
    if value == "all" {
        return Ok(crate::p70_all_fixture_kinds());
    }
    let mut fixtures = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        let fixture = P70ReplayFixtureKind::from_str(item).ok_or_else(|| {
            usage(format!(
                "contract-replay received unsupported fixture '{}'; expected all|log|sparse|json|hybrid",
                item
            ))
        })?;
        fixtures.push(fixture);
    }
    if fixtures.is_empty() {
        return Err(usage("contract-replay requires non-empty --fixtures"));
    }
    Ok(fixtures)
}

fn parse_p71_fiber_store_options(args: &[String]) -> Result<P71FiberStoreCliOptions, String> {
    let mut corpora = None;
    let mut budget_bytes = None;
    let mut runs = None;
    let mut queries = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--corpus" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("fiber-store-bench requires a value after --corpus"))?;
            corpora = Some(parse_p71_corpora(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--corpus=") {
            corpora = Some(parse_p71_corpora(value)?);
            idx += 1;
        } else if arg == "--budget-bytes" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("fiber-store-bench requires a value after --budget-bytes"))?;
            budget_bytes = Some(parse_positive_u64(
                value,
                "fiber-store-bench",
                "--budget-bytes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--budget-bytes=") {
            budget_bytes = Some(parse_positive_u64(
                value,
                "fiber-store-bench",
                "--budget-bytes",
            )?);
            idx += 1;
        } else if arg == "--runs" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("fiber-store-bench requires a value after --runs"))?;
            runs = Some(parse_positive_usize(value, "fiber-store-bench", "--runs")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = Some(parse_positive_usize(value, "fiber-store-bench", "--runs")?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("fiber-store-bench requires a value after --queries"))?;
            queries = Some(parse_positive_usize(
                value,
                "fiber-store-bench",
                "--queries",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(
                value,
                "fiber-store-bench",
                "--queries",
            )?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("fiber-store-bench requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("fiber-store-bench requires a value after --format"))?;
            format = Some(parse_format_value(value, "fiber-store-bench")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "fiber-store-bench")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "fiber-store-bench received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P71FiberStoreCliOptions {
        options: P71FiberStoreOptions {
            corpora: corpora.ok_or_else(|| {
                usage("fiber-store-bench requires --corpus all|code|logs|json|csv|guard")
            })?,
            budget_bytes: budget_bytes
                .ok_or_else(|| usage("fiber-store-bench requires --budget-bytes N"))?,
            runs: runs.ok_or_else(|| usage("fiber-store-bench requires --runs N"))?,
            queries: queries.ok_or_else(|| usage("fiber-store-bench requires --queries N"))?,
        },
        export_dir: export_dir
            .ok_or_else(|| usage("fiber-store-bench requires --export-dir <path>"))?,
        format: format.ok_or_else(|| usage("fiber-store-bench requires --format json|markdown"))?,
    })
}

fn parse_p71_corpora(value: &str) -> Result<Vec<RealDataCorpusKind>, String> {
    if value == "all" {
        return Ok(crate::p71_all_corpora());
    }
    let mut corpora = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        let corpus = RealDataCorpusKind::from_str(item).ok_or_else(|| {
            usage(format!(
                "fiber-store-bench received unsupported corpus '{}'; expected all|code|logs|json|csv|guard",
                item
            ))
        })?;
        corpora.push(corpus);
    }
    if corpora.is_empty() {
        return Err(usage("fiber-store-bench requires non-empty --corpus"));
    }
    Ok(corpora)
}

fn parse_bool_value(value: &str, command: &str, option: &str) -> Result<bool, String> {
    match value {
        "true" | "on" | "yes" => Ok(true),
        "false" | "off" | "no" => Ok(false),
        _ => Err(usage(format!(
            "{} received invalid {} '{}'",
            command, option, value
        ))),
    }
}

fn parse_positive_u64(value: &str, command: &str, option: &str) -> Result<u64, String> {
    let parsed = value.parse::<u64>().map_err(|_| {
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

fn parse_positive_f64(value: &str, command: &str, option: &str) -> Result<f64, String> {
    let parsed = value.parse::<f64>().map_err(|_| {
        usage(format!(
            "{} received invalid {} '{}'",
            command, option, value
        ))
    })?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(usage(format!(
            "{} requires {} greater than zero",
            command, option
        )));
    }
    Ok(parsed)
}

fn parse_cache_value(value: &str) -> Result<bool, String> {
    match value {
        "on" => Ok(true),
        "off" => Ok(false),
        _ => Err(usage(format!(
            "ratio-actors received unsupported cache value '{}'; expected on|off",
            value
        ))),
    }
}

fn parse_p65_calibration_options(args: &[String]) -> Result<P65CalibrationCliOptions, String> {
    let mut workload = None;
    let mut workload_all = false;
    let mut mode = None;
    let mut runs = None;
    let mut queries = None;
    let mut radius_grid = None;
    let mut budget_grid = None;
    let mut cache_grid = None;
    let mut journal_grid = None;
    let mut query_locality_grid = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--workload" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors-calibrate requires a value after --workload"))?;
            let parsed = parse_p64_workload_value(value)?;
            workload_all = parsed.is_none();
            workload = parsed;
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--workload=") {
            let parsed = parse_p64_workload_value(value)?;
            workload_all = parsed.is_none();
            workload = parsed;
            idx += 1;
        } else if arg == "--mode" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors-calibrate requires a value after --mode"))?;
            mode = Some(parse_mode_value(value, "ratio-actors-calibrate")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--mode=") {
            mode = Some(parse_mode_value(value, "ratio-actors-calibrate")?);
            idx += 1;
        } else if arg == "--runs" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors-calibrate requires a value after --runs"))?;
            runs = Some(parse_positive_usize(
                value,
                "ratio-actors-calibrate",
                "--runs",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = Some(parse_positive_usize(
                value,
                "ratio-actors-calibrate",
                "--runs",
            )?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors-calibrate requires a value after --queries"))?;
            queries = Some(parse_positive_usize(
                value,
                "ratio-actors-calibrate",
                "--queries",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(
                value,
                "ratio-actors-calibrate",
                "--queries",
            )?);
            idx += 1;
        } else if arg == "--radius-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-actors-calibrate requires a value after --radius-grid")
            })?;
            radius_grid = Some(parse_usize_grid(value, "radius-grid")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--radius-grid=") {
            radius_grid = Some(parse_usize_grid(value, "radius-grid")?);
            idx += 1;
        } else if arg == "--budget-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-actors-calibrate requires a value after --budget-grid")
            })?;
            budget_grid = Some(parse_u64_grid(value, "budget-grid")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--budget-grid=") {
            budget_grid = Some(parse_u64_grid(value, "budget-grid")?);
            idx += 1;
        } else if arg == "--cache-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-actors-calibrate requires a value after --cache-grid")
            })?;
            cache_grid = Some(parse_cache_grid(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cache-grid=") {
            cache_grid = Some(parse_cache_grid(value)?);
            idx += 1;
        } else if arg == "--journal-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-actors-calibrate requires a value after --journal-grid")
            })?;
            journal_grid = Some(parse_journal_grid(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--journal-grid=") {
            journal_grid = Some(parse_journal_grid(value)?);
            idx += 1;
        } else if arg == "--query-locality-grid" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-actors-calibrate requires a value after --query-locality-grid")
            })?;
            query_locality_grid = Some(parse_query_locality_grid(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--query-locality-grid=") {
            query_locality_grid = Some(parse_query_locality_grid(value)?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("ratio-actors-calibrate requires a value after --export-dir")
            })?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("ratio-actors-calibrate requires a value after --format"))?;
            format = Some(parse_format_value(value, "ratio-actors-calibrate")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "ratio-actors-calibrate")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "ratio-actors-calibrate received unsupported option '{}'",
                arg
            )));
        }
    }

    if !workload_all && workload.is_none() {
        return Err(usage(
            "ratio-actors-calibrate requires --workload <name>|all",
        ));
    }

    Ok(P65CalibrationCliOptions {
        options: P65ActorCalibrationOptions {
            workload,
            mode: mode.ok_or_else(|| {
                usage("ratio-actors-calibrate requires --mode standard|ambitious")
            })?,
            runs: runs.ok_or_else(|| usage("ratio-actors-calibrate requires --runs N"))?,
            queries: queries.ok_or_else(|| usage("ratio-actors-calibrate requires --queries N"))?,
            radius_grid: radius_grid
                .ok_or_else(|| usage("ratio-actors-calibrate requires --radius-grid"))?,
            budget_grid: budget_grid
                .ok_or_else(|| usage("ratio-actors-calibrate requires --budget-grid"))?,
            cache_grid: cache_grid
                .ok_or_else(|| usage("ratio-actors-calibrate requires --cache-grid"))?,
            journal_grid: journal_grid
                .ok_or_else(|| usage("ratio-actors-calibrate requires --journal-grid"))?,
            query_locality_grid: query_locality_grid
                .ok_or_else(|| usage("ratio-actors-calibrate requires --query-locality-grid"))?,
        },
        export_dir,
        format: format
            .ok_or_else(|| usage("ratio-actors-calibrate requires --format json|markdown"))?,
    })
}

fn parse_usize_grid(value: &str, name: &str) -> Result<Vec<usize>, String> {
    let mut parsed = Vec::new();
    for item in value.split(',') {
        let number = parse_positive_usize(item.trim(), "ratio-actors-calibrate", name)?;
        parsed.push(number);
    }
    if parsed.is_empty() {
        return Err(usage(format!(
            "ratio-actors-calibrate requires non-empty {}",
            name
        )));
    }
    Ok(parsed)
}

fn parse_u64_grid(value: &str, name: &str) -> Result<Vec<u64>, String> {
    let mut parsed = Vec::new();
    for item in value.split(',') {
        let number = parse_positive_u64(item.trim(), "ratio-actors-calibrate", name)?;
        parsed.push(number);
    }
    if parsed.is_empty() {
        return Err(usage(format!(
            "ratio-actors-calibrate requires non-empty {}",
            name
        )));
    }
    Ok(parsed)
}

fn parse_cache_grid(value: &str) -> Result<Vec<bool>, String> {
    value
        .split(',')
        .map(|item| parse_cache_value(item.trim()))
        .collect()
}

fn parse_journal_grid(value: &str) -> Result<Vec<P65JournalPolicy>, String> {
    value
        .split(',')
        .map(|item| {
            P65JournalPolicy::from_str(item.trim()).ok_or_else(|| {
                usage(format!(
                    "ratio-actors-calibrate received unsupported journal policy '{}'; expected lazy|compact",
                    item.trim()
                ))
            })
        })
        .collect()
}

fn parse_query_locality_grid(value: &str) -> Result<Vec<P65QueryLocality>, String> {
    value
        .split(',')
        .map(|item| {
            P65QueryLocality::from_str(item.trim()).ok_or_else(|| {
                usage(format!(
                    "ratio-actors-calibrate received unsupported query locality '{}'; expected clustered|random|mixed",
                    item.trim()
                ))
            })
        })
        .collect()
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
        "  atlas-cli ratio-actors <file.atlas> --workload realish_log_events|realish_sparse_csv|realish_json_records|realish_hybrid_field_fixture|all --actor-strategy no-actor|single-local|specialized-crud|over-agentic|all --mode smoke|standard|ambitious --runs N --queries N --neighborhood-radius N --budget-bytes N [--cache on|off] [--export-dir <path>] --format json|markdown",
        "  atlas-cli ratio-actors-calibrate <file.atlas> --workload realish_log_events|realish_sparse_csv|realish_json_records|realish_hybrid_field_fixture|all --mode standard|ambitious --runs N --queries N --radius-grid a,b --budget-grid a,b --cache-grid off,on --journal-grid lazy,compact --query-locality-grid clustered,random,mixed [--export-dir <path>] --format json|markdown",
        "  atlas-cli ratio-fibers <file.atlas> --workload realish_log_events|realish_sparse_csv|realish_json_records|realish_hybrid_field_fixture|all --fiber-strategy point-fiber|neighborhood-fiber|actor-fiber|actor-neighborhood-fiber|all --mode smoke|standard|ambitious --runs N --queries N --neighborhood-radius N --budget-bytes N --journal eager|lazy|compact [--cache on|off] [--update-rate low|medium|high] [--audit-rate low|medium|high] [--export-dir <path>] --format json|markdown",
        "  atlas-cli ratio-fibers-calibrate <file.atlas> --workload realish_log_events|realish_sparse_csv|realish_json_records|realish_hybrid_field_fixture|all --mode standard|ambitious --runs N --queries N --radius-grid a,b --budget-grid a,b --cache-grid off,on,compact --journal-grid eager,lazy,compact --audit-grid minimal,sampled,full --compaction-grid off,threshold,aggressive --query-locality-grid clustered,random,mixed --fiber-projection-grid shallow,medium,full [--export-dir <path>] --format json|markdown",
        "  atlas-cli ratio-fibers-promote <file.atlas> [--run-ablations] [--run-stress] [--phase-map] [--mode-pair standard,ambitious] [--strict true|false] [--export-dir <path>] --format json|markdown",
        "  atlas-cli contract-check <file.atlas> --format json",
        "  atlas-cli contract-run <file.atlas> --mode smoke|standard|ambitious --runs N --queries N [--export-dir <path>] --format json|markdown",
        "  atlas-cli contract-replay <file.atlas> --fixtures all|log|sparse|json|hybrid --mode smoke|standard|ambitious --runs N --queries N --tolerance-percent N [--export-dir <path>] --format json|markdown",
        "  atlas-cli fiber-store-bench --corpus all|code|logs|json|csv|guard --budget-bytes N --runs N --queries N --export-dir <path> --format json|markdown",
    ];
    format!("{}\n{}", detail.as_ref(), commands.join("\n"))
}
