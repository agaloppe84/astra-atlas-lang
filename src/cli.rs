use crate::{
    bench_report_json, export_json_file, metrics_json_file, p57_report_json_file,
    p58_metrics_json_file, p58_report_json_file, p58_report_markdown_file,
    p61_virtual_ratio_report_json_file, p62_real_ratio_report_json_file_with_runs,
    p63_campaign_compare_json_files, p63_campaign_register_json_file,
    p63_campaign_report_file_with_runs, p63_campaign_report_to_json,
    p63_campaign_set_summary_json_file, p63_campaign_summary_json_file, run_workload_file,
    validate_file, write_p63_campaign_exports, write_p64_campaign_exports, DiagnosticCode,
    FiberGenerationStrategy, Level1TopologyKind, Level1VirtualSpaceEstimateOptions,
    P63ThresholdProfile, P64GenerationPolicy, P64RatioRealishOptions, P64WorkloadKind,
    P65ActorCalibrationOptions, P65ActorStrategy, P65JournalPolicy, P65QueryLocality,
    P65RatioActorsOptions, P66JournalPolicy, P66RateProfile, P66RatioFibersOptions, P67AuditPolicy,
    P67CachePolicy, P67CompactionPolicy, P67FiberCalibrationOptions, P67FiberProjectionDepth,
    P67QueryLocality, P68PromotionOptions, P69ContractRunOptions, P70ContractReplayOptions,
    P70ReplayFixtureKind, P71FiberStoreOptions, P72CompactionPolicy, P72LivingStoreOptions,
    P73CompareP72, P73CubicalStoreOptions, P74CompactionPolicy, P74LocalityProfile,
    P74TopologyLivingOptions, P74UpdatePressure, P76CompareTarget, P76VirtualSpaceEstimateOptions,
    P77CalibrationGridKind, P77RouterCalibrationOptions, P78Level1SpaceOptions, RealDataCorpusKind,
    RouterLivingOptions, RouterPolicy, RoutingOracleOptions, TopologyKind, WorkloadMode,
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
        "living-store-bench" => living_store_bench_command(args),
        "cubical-store-bench" => cubical_store_bench_command(args),
        "topology-living-bench" => topology_living_bench_command(args),
        "mixed-topology-bench" => mixed_topology_bench_command(args),
        "routing-oracle-bench" => routing_oracle_bench_command(args),
        "routing-oracle-calibrate" => routing_oracle_calibrate_command(args),
        "virtual-space-estimate" => virtual_space_estimate_command(args),
        "level1-space-bench" => level1_space_bench_command(args),
        "level1-space-estimate" => level1_space_estimate_command(args),
        path if args.len() == 1 => check_path(path),
        _ => Err(usage("unknown command")),
    }
}

fn check_path(path: &str) -> Result<(), String> {
    if crate::p78_level1_file_looks_like(path) {
        return match crate::p78_level1_contract_report_file(path) {
            Ok(report) => {
                println!(
                    "OK: p78_level1_space={} topology={} address_bits={} local_on_address={} guard={}",
                    report.space_id,
                    report.topology,
                    report.address_bits,
                    report.local_on_address,
                    report.guard_no_false_gain
                );
                Ok(())
            }
            Err(diagnostic) => Err(diagnostic.to_string()),
        };
    }
    if crate::p77_router_policy_file_looks_like(path) {
        return match crate::p77_router_policy_report_file(path) {
            Ok(report) => {
                println!(
                    "OK: p77_policy={} confidence={:.2} fallback={:.2} living={} guard={}",
                    report.policy_id,
                    report.confidence_threshold,
                    report.fallback_threshold,
                    report.living_memory_only,
                    report.guard_no_false_gain
                );
                Ok(())
            }
            Err(diagnostic) => Err(diagnostic.to_string()),
        };
    }
    if crate::p76_process_file_looks_like(path) {
        return match crate::p76_process_contract_report_file(path) {
            Ok(report) => {
                println!(
                    "OK: p76_oracle={} compare={} living={} virtual_metrics={} guard={}",
                    report.oracle_id,
                    report.compare_count,
                    report.living_memory_only,
                    report.virtual_space_metrics_required,
                    report.guard_no_false_gain
                );
                Ok(())
            }
            Err(diagnostic) => Err(diagnostic.to_string()),
        };
    }
    if crate::p75_router_file_looks_like(path) {
        return match crate::p75_router_contract_report_file(path) {
            Ok(report) => {
                println!(
                    "OK: p75_router={} default={} living={} guard={}",
                    report.router_id,
                    report.default_topology,
                    report.living_memory_only,
                    report.guard_no_false_gain
                );
                Ok(())
            }
            Err(diagnostic) => Err(diagnostic.to_string()),
        };
    }
    if crate::p74_topology_file_looks_like(path) {
        return match crate::p74_topology_contract_report_file(path) {
            Ok(report) => {
                println!(
                    "OK: p74_topology={} kind={} reopen={} guard={}",
                    report.topology_name,
                    report.topology_kind,
                    report.reopen_equivalence_gate,
                    report.guard_no_false_gain
                );
                Ok(())
            }
            Err(diagnostic) => Err(diagnostic.to_string()),
        };
    }
    if crate::p73_cubical_file_looks_like(path) {
        return match crate::p73_cubical_lifecycle_report_file(path) {
            Ok(report) => {
                println!(
                    "OK: p73_topology={} faces={} gluing={} reopen={}",
                    report.topology_id,
                    report.faces,
                    report.face_gluing_consistency,
                    report.cubical_reopen_equivalence
                );
                Ok(())
            }
            Err(diagnostic) => Err(diagnostic.to_string()),
        };
    }
    if crate::p72_lifecycle_file_looks_like(path) {
        return match crate::p72_lifecycle_report_file(path) {
            Ok(report) => {
                println!(
                    "OK: p72_lifecycle={} reopen_equivalence={} journal_replay_bounded={}",
                    report.lifecycle_id,
                    report.reopen_equivalence_gate,
                    report.journal_replay_bounded
                );
                Ok(())
            }
            Err(diagnostic) => Err(diagnostic.to_string()),
        };
    }
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

fn living_store_bench_command(args: &[String]) -> Result<(), String> {
    let options = parse_p72_living_store_options(&args[1..])?;
    let report = crate::p72_living_store_bench(options.options, &options.export_dir)
        .map_err(|diagnostic| diagnostic.to_string())?;
    match options.format {
        OutputFormat::Json => println!("{}", crate::p72_living_store_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p72_living_store_markdown(&report)),
    }
    Ok(())
}

fn cubical_store_bench_command(args: &[String]) -> Result<(), String> {
    let options = parse_p73_cubical_store_options(&args[1..])?;
    let report = crate::p73_cubical_store_bench(options.options, &options.export_dir)
        .map_err(|diagnostic| diagnostic.to_string())?;
    match options.format {
        OutputFormat::Json => println!("{}", crate::p73_cubical_store_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p73_cubical_store_markdown(&report)),
    }
    Ok(())
}

fn topology_living_bench_command(args: &[String]) -> Result<(), String> {
    let options = parse_p74_topology_living_options(&args[1..])?;
    let report = crate::p74_topology_living_bench(options.options, &options.export_dir)
        .map_err(|diagnostic| diagnostic.to_string())?;
    match options.format {
        OutputFormat::Json => println!("{}", crate::p74_topology_living_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p74_topology_living_markdown(&report)),
    }
    Ok(())
}

fn mixed_topology_bench_command(args: &[String]) -> Result<(), String> {
    let options = parse_p75_router_living_options(&args[1..])?;
    let report = crate::p75_mixed_topology_bench(options.options, &options.export_dir)
        .map_err(|diagnostic| diagnostic.to_string())?;
    match options.format {
        OutputFormat::Json => println!("{}", crate::p75_mixed_topology_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p75_mixed_topology_markdown(&report)),
    }
    Ok(())
}

fn routing_oracle_bench_command(args: &[String]) -> Result<(), String> {
    let options = parse_p76_routing_oracle_options(&args[1..])?;
    let report = crate::p76_routing_oracle_bench(options.options, &options.export_dir)
        .map_err(|diagnostic| diagnostic.to_string())?;
    match options.format {
        OutputFormat::Json => println!("{}", crate::p76_routing_oracle_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p76_routing_oracle_markdown(&report)),
    }
    Ok(())
}

fn routing_oracle_calibrate_command(args: &[String]) -> Result<(), String> {
    let options = parse_p77_router_calibration_options(&args[1..])?;
    let report = crate::p77_calibrate_router(options.options, &options.export_dir)
        .map_err(|diagnostic| diagnostic.to_string())?;
    match options.format {
        OutputFormat::Json => println!("{}", crate::p77_calibration_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p77_calibration_markdown(&report)),
    }
    Ok(())
}

fn virtual_space_estimate_command(args: &[String]) -> Result<(), String> {
    let options = parse_p76_virtual_space_estimate_options(&args[1..])?;
    let metrics = crate::p76_virtual_space_estimate(options.options);
    match options.format {
        OutputFormat::Json => println!("{}", crate::p76_virtual_space_metrics_json(&metrics)),
        OutputFormat::Markdown => println!(
            "- virtual_cell_count: `{}`\n- virtual_fiber_count: `{}`\n- virtual_effective_bytes_equivalent: `{}`\n- bytes_are_equivalent_not_stored: `{}`",
            metrics.virtual_cell_count,
            metrics.virtual_fiber_count,
            metrics.virtual_effective_bytes_equivalent,
            metrics.bytes_are_equivalent_not_stored
        ),
    }
    Ok(())
}

fn level1_space_bench_command(args: &[String]) -> Result<(), String> {
    let options = parse_p78_level1_space_options(&args[1..])?;
    let report = crate::p78_level1_space_bench(options.options, &options.export_dir)
        .map_err(|diagnostic| diagnostic.to_string())?;
    match options.format {
        OutputFormat::Json => println!("{}", crate::p78_level1_space_json(&report)),
        OutputFormat::Markdown => println!("{}", crate::p78_level1_space_markdown(&report)),
    }
    Ok(())
}

fn level1_space_estimate_command(args: &[String]) -> Result<(), String> {
    let options = parse_p78_level1_space_estimate_options(&args[1..])?;
    let metrics = crate::p78_level1_space_estimate(options.options);
    match options.format {
        OutputFormat::Json => println!("{}", crate::p78_virtual_space_metrics_json(&metrics)),
        OutputFormat::Markdown => println!(
            "- level1_effective_address_count: `{}`\n- virtual_cell_count: `{}`\n- virtual_fiber_count: `{}`\n- virtual_effective_bytes_equivalent: `{}`\n- limiting_factor: `{}`\n- bytes_are_equivalent_not_stored: `{}`",
            metrics.level1_effective_address_count,
            metrics.virtual_cell_count,
            metrics.virtual_fiber_count,
            metrics.virtual_effective_bytes_equivalent,
            metrics.limiting_factor,
            metrics.bytes_are_equivalent_not_stored
        ),
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

#[derive(Debug, Clone, PartialEq)]
struct P72LivingStoreCliOptions {
    options: P72LivingStoreOptions,
    export_dir: String,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
struct P73CubicalStoreCliOptions {
    options: P73CubicalStoreOptions,
    export_dir: String,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
struct P74TopologyLivingCliOptions {
    options: P74TopologyLivingOptions,
    export_dir: String,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
struct P75RouterLivingCliOptions {
    options: RouterLivingOptions,
    export_dir: String,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
struct P76RoutingOracleCliOptions {
    options: RoutingOracleOptions,
    export_dir: String,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
struct P76VirtualSpaceEstimateCliOptions {
    options: P76VirtualSpaceEstimateOptions,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
struct P77RouterCalibrationCliOptions {
    options: P77RouterCalibrationOptions,
    export_dir: String,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
struct P78Level1SpaceCliOptions {
    options: P78Level1SpaceOptions,
    export_dir: String,
    format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq)]
struct P78Level1EstimateCliOptions {
    options: Level1VirtualSpaceEstimateOptions,
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

fn parse_p72_living_store_options(args: &[String]) -> Result<P72LivingStoreCliOptions, String> {
    let mut corpora = None;
    let mut budget_bytes = None;
    let mut runs = None;
    let mut queries = None;
    let mut updates = None;
    let mut deletes = None;
    let mut compact = None;
    let mut adaptive = None;
    let mut reopen_check = Some(true);
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--corpus" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --corpus"))?;
            corpora = Some(parse_p71_corpora(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--corpus=") {
            corpora = Some(parse_p71_corpora(value)?);
            idx += 1;
        } else if arg == "--budget-bytes" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --budget-bytes"))?;
            budget_bytes = Some(parse_positive_u64(
                value,
                "living-store-bench",
                "--budget-bytes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--budget-bytes=") {
            budget_bytes = Some(parse_positive_u64(
                value,
                "living-store-bench",
                "--budget-bytes",
            )?);
            idx += 1;
        } else if arg == "--runs" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --runs"))?;
            runs = Some(parse_positive_usize(value, "living-store-bench", "--runs")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = Some(parse_positive_usize(value, "living-store-bench", "--runs")?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --queries"))?;
            queries = Some(parse_positive_usize(
                value,
                "living-store-bench",
                "--queries",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(
                value,
                "living-store-bench",
                "--queries",
            )?);
            idx += 1;
        } else if arg == "--updates" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --updates"))?;
            updates = Some(parse_nonnegative_usize(
                value,
                "living-store-bench",
                "--updates",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--updates=") {
            updates = Some(parse_nonnegative_usize(
                value,
                "living-store-bench",
                "--updates",
            )?);
            idx += 1;
        } else if arg == "--deletes" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --deletes"))?;
            deletes = Some(parse_nonnegative_usize(
                value,
                "living-store-bench",
                "--deletes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--deletes=") {
            deletes = Some(parse_nonnegative_usize(
                value,
                "living-store-bench",
                "--deletes",
            )?);
            idx += 1;
        } else if arg == "--compact" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --compact"))?;
            compact = Some(parse_p72_compact_value(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--compact=") {
            compact = Some(parse_p72_compact_value(value)?);
            idx += 1;
        } else if arg == "--adaptive" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --adaptive"))?;
            adaptive = Some(parse_bool_value(value, "living-store-bench", "--adaptive")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--adaptive=") {
            adaptive = Some(parse_bool_value(value, "living-store-bench", "--adaptive")?);
            idx += 1;
        } else if arg == "--reopen-check" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --reopen-check"))?;
            reopen_check = Some(parse_bool_value(
                value,
                "living-store-bench",
                "--reopen-check",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--reopen-check=") {
            reopen_check = Some(parse_bool_value(
                value,
                "living-store-bench",
                "--reopen-check",
            )?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("living-store-bench requires a value after --format"))?;
            format = Some(parse_format_value(value, "living-store-bench")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "living-store-bench")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "living-store-bench received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P72LivingStoreCliOptions {
        options: P72LivingStoreOptions {
            corpora: corpora.ok_or_else(|| {
                usage("living-store-bench requires --corpus all|code|logs|json|csv|guard")
            })?,
            budget_bytes: budget_bytes
                .ok_or_else(|| usage("living-store-bench requires --budget-bytes N"))?,
            runs: runs.ok_or_else(|| usage("living-store-bench requires --runs N"))?,
            queries: queries.ok_or_else(|| usage("living-store-bench requires --queries N"))?,
            updates: updates.ok_or_else(|| usage("living-store-bench requires --updates N"))?,
            deletes: deletes.ok_or_else(|| usage("living-store-bench requires --deletes N"))?,
            compact: compact.ok_or_else(|| {
                usage("living-store-bench requires --compact off|threshold|aggressive")
            })?,
            adaptive: adaptive
                .ok_or_else(|| usage("living-store-bench requires --adaptive on|off"))?,
            reopen_check: reopen_check.unwrap_or(true),
        },
        export_dir: export_dir
            .ok_or_else(|| usage("living-store-bench requires --export-dir <path>"))?,
        format: format
            .ok_or_else(|| usage("living-store-bench requires --format json|markdown"))?,
    })
}

fn parse_p72_compact_value(value: &str) -> Result<P72CompactionPolicy, String> {
    P72CompactionPolicy::from_str(value).ok_or_else(|| {
        usage(format!(
            "living-store-bench received unsupported compact policy '{}'; expected off|threshold|aggressive",
            value
        ))
    })
}

fn parse_p73_cubical_store_options(args: &[String]) -> Result<P73CubicalStoreCliOptions, String> {
    let mut corpora = None;
    let mut budget_bytes = None;
    let mut cycles = None;
    let mut queries = None;
    let mut updates = None;
    let mut deletes = None;
    let mut corruptions = None;
    let mut compact = None;
    let mut adaptive = None;
    let mut compare_p72 = Some(P73CompareP72::Off);
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--corpus" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --corpus"))?;
            corpora = Some(parse_p71_corpora(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--corpus=") {
            corpora = Some(parse_p71_corpora(value)?);
            idx += 1;
        } else if arg == "--budget-bytes" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("cubical-store-bench requires a value after --budget-bytes")
            })?;
            budget_bytes = Some(parse_positive_u64(
                value,
                "cubical-store-bench",
                "--budget-bytes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--budget-bytes=") {
            budget_bytes = Some(parse_positive_u64(
                value,
                "cubical-store-bench",
                "--budget-bytes",
            )?);
            idx += 1;
        } else if arg == "--cycles" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --cycles"))?;
            cycles = Some(parse_positive_usize(
                value,
                "cubical-store-bench",
                "--cycles",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cycles=") {
            cycles = Some(parse_positive_usize(
                value,
                "cubical-store-bench",
                "--cycles",
            )?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --queries"))?;
            queries = Some(parse_positive_usize(
                value,
                "cubical-store-bench",
                "--queries",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(
                value,
                "cubical-store-bench",
                "--queries",
            )?);
            idx += 1;
        } else if arg == "--updates" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --updates"))?;
            updates = Some(parse_nonnegative_usize(
                value,
                "cubical-store-bench",
                "--updates",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--updates=") {
            updates = Some(parse_nonnegative_usize(
                value,
                "cubical-store-bench",
                "--updates",
            )?);
            idx += 1;
        } else if arg == "--deletes" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --deletes"))?;
            deletes = Some(parse_nonnegative_usize(
                value,
                "cubical-store-bench",
                "--deletes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--deletes=") {
            deletes = Some(parse_nonnegative_usize(
                value,
                "cubical-store-bench",
                "--deletes",
            )?);
            idx += 1;
        } else if arg == "--corruptions" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --corruptions"))?;
            corruptions = Some(parse_nonnegative_usize(
                value,
                "cubical-store-bench",
                "--corruptions",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--corruptions=") {
            corruptions = Some(parse_nonnegative_usize(
                value,
                "cubical-store-bench",
                "--corruptions",
            )?);
            idx += 1;
        } else if arg == "--compact" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --compact"))?;
            compact = Some(parse_p72_compact_value(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--compact=") {
            compact = Some(parse_p72_compact_value(value)?);
            idx += 1;
        } else if arg == "--adaptive" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --adaptive"))?;
            adaptive = Some(parse_bool_value(
                value,
                "cubical-store-bench",
                "--adaptive",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--adaptive=") {
            adaptive = Some(parse_bool_value(
                value,
                "cubical-store-bench",
                "--adaptive",
            )?);
            idx += 1;
        } else if arg == "--compare-p72" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --compare-p72"))?;
            compare_p72 = Some(parse_p73_compare_p72(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--compare-p72=") {
            compare_p72 = Some(parse_p73_compare_p72(value)?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("cubical-store-bench requires a value after --format"))?;
            format = Some(parse_format_value(value, "cubical-store-bench")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "cubical-store-bench")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "cubical-store-bench received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P73CubicalStoreCliOptions {
        options: P73CubicalStoreOptions {
            corpora: corpora.ok_or_else(|| {
                usage("cubical-store-bench requires --corpus all|code|logs|json|csv|guard")
            })?,
            budget_bytes: budget_bytes
                .ok_or_else(|| usage("cubical-store-bench requires --budget-bytes N"))?,
            cycles: cycles.ok_or_else(|| usage("cubical-store-bench requires --cycles N"))?,
            queries: queries.ok_or_else(|| usage("cubical-store-bench requires --queries N"))?,
            updates: updates.ok_or_else(|| usage("cubical-store-bench requires --updates N"))?,
            deletes: deletes.ok_or_else(|| usage("cubical-store-bench requires --deletes N"))?,
            corruptions: corruptions
                .ok_or_else(|| usage("cubical-store-bench requires --corruptions N"))?,
            compact: compact.ok_or_else(|| {
                usage("cubical-store-bench requires --compact off|threshold|aggressive")
            })?,
            adaptive: adaptive
                .ok_or_else(|| usage("cubical-store-bench requires --adaptive on|off"))?,
            compare_p72: compare_p72.unwrap_or(P73CompareP72::Off),
        },
        export_dir: export_dir
            .ok_or_else(|| usage("cubical-store-bench requires --export-dir <path>"))?,
        format: format
            .ok_or_else(|| usage("cubical-store-bench requires --format json|markdown"))?,
    })
}

fn parse_p73_compare_p72(value: &str) -> Result<P73CompareP72, String> {
    P73CompareP72::from_str(value).ok_or_else(|| {
        usage(format!(
            "cubical-store-bench received unsupported --compare-p72 '{}'; expected baseline|off",
            value
        ))
    })
}

fn parse_p74_topology_living_options(
    args: &[String],
) -> Result<P74TopologyLivingCliOptions, String> {
    let mut corpora = None;
    let mut topologies = None;
    let mut target_source_bytes = None;
    let mut cycles = None;
    let mut queries = None;
    let mut updates = None;
    let mut deletes = None;
    let mut compact = None;
    let mut adaptive = None;
    let mut locality = None;
    let mut update_pressure = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--corpus" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("topology-living-bench requires a value after --corpus"))?;
            corpora = Some(parse_p71_corpora(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--corpus=") {
            corpora = Some(parse_p71_corpora(value)?);
            idx += 1;
        } else if arg == "--topology" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("topology-living-bench requires a value after --topology"))?;
            topologies = Some(parse_p74_topologies(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--topology=") {
            topologies = Some(parse_p74_topologies(value)?);
            idx += 1;
        } else if arg == "--target-source-bytes" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("topology-living-bench requires a value after --target-source-bytes")
            })?;
            target_source_bytes = Some(parse_positive_u64(
                value,
                "topology-living-bench",
                "--target-source-bytes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--target-source-bytes=") {
            target_source_bytes = Some(parse_positive_u64(
                value,
                "topology-living-bench",
                "--target-source-bytes",
            )?);
            idx += 1;
        } else if arg == "--cycles" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("topology-living-bench requires a value after --cycles"))?;
            cycles = Some(parse_positive_usize(
                value,
                "topology-living-bench",
                "--cycles",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cycles=") {
            cycles = Some(parse_positive_usize(
                value,
                "topology-living-bench",
                "--cycles",
            )?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("topology-living-bench requires a value after --queries"))?;
            queries = Some(parse_positive_usize(
                value,
                "topology-living-bench",
                "--queries",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(
                value,
                "topology-living-bench",
                "--queries",
            )?);
            idx += 1;
        } else if arg == "--updates" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("topology-living-bench requires a value after --updates"))?;
            updates = Some(parse_nonnegative_usize(
                value,
                "topology-living-bench",
                "--updates",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--updates=") {
            updates = Some(parse_nonnegative_usize(
                value,
                "topology-living-bench",
                "--updates",
            )?);
            idx += 1;
        } else if arg == "--deletes" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("topology-living-bench requires a value after --deletes"))?;
            deletes = Some(parse_nonnegative_usize(
                value,
                "topology-living-bench",
                "--deletes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--deletes=") {
            deletes = Some(parse_nonnegative_usize(
                value,
                "topology-living-bench",
                "--deletes",
            )?);
            idx += 1;
        } else if arg == "--compact" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("topology-living-bench requires a value after --compact"))?;
            compact = Some(parse_p74_compact_value(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--compact=") {
            compact = Some(parse_p74_compact_value(value)?);
            idx += 1;
        } else if arg == "--adaptive" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("topology-living-bench requires a value after --adaptive"))?;
            adaptive = Some(parse_bool_value(
                value,
                "topology-living-bench",
                "--adaptive",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--adaptive=") {
            adaptive = Some(parse_bool_value(
                value,
                "topology-living-bench",
                "--adaptive",
            )?);
            idx += 1;
        } else if arg == "--locality" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("topology-living-bench requires a value after --locality"))?;
            locality = Some(parse_p74_locality(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--locality=") {
            locality = Some(parse_p74_locality(value)?);
            idx += 1;
        } else if arg == "--update-pressure" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("topology-living-bench requires a value after --update-pressure")
            })?;
            update_pressure = Some(parse_p74_update_pressure(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--update-pressure=") {
            update_pressure = Some(parse_p74_update_pressure(value)?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("topology-living-bench requires a value after --export-dir")
            })?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("topology-living-bench requires a value after --format"))?;
            format = Some(parse_format_value(value, "topology-living-bench")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "topology-living-bench")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "topology-living-bench received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P74TopologyLivingCliOptions {
        options: P74TopologyLivingOptions {
            corpora: corpora.ok_or_else(|| {
                usage("topology-living-bench requires --corpus all|code|logs|json|csv|guard")
            })?,
            topologies: topologies.ok_or_else(|| {
                usage(
                    "topology-living-bench requires --topology all|linear|cubical|trie|graph|hypergraph|hierarchical",
                )
            })?,
            target_source_bytes: target_source_bytes
                .ok_or_else(|| usage("topology-living-bench requires --target-source-bytes N"))?,
            cycles: cycles.ok_or_else(|| usage("topology-living-bench requires --cycles N"))?,
            queries: queries.ok_or_else(|| usage("topology-living-bench requires --queries N"))?,
            updates: updates.ok_or_else(|| usage("topology-living-bench requires --updates N"))?,
            deletes: deletes.ok_or_else(|| usage("topology-living-bench requires --deletes N"))?,
            compact: compact.ok_or_else(|| {
                usage("topology-living-bench requires --compact off|threshold|aggressive|adaptive")
            })?,
            adaptive: adaptive
                .ok_or_else(|| usage("topology-living-bench requires --adaptive on|off"))?,
            locality: locality.ok_or_else(|| {
                usage("topology-living-bench requires --locality clustered|random|mixed|hotspot")
            })?,
            update_pressure: update_pressure.ok_or_else(|| {
                usage("topology-living-bench requires --update-pressure low|medium|high")
            })?,
        },
        export_dir: export_dir
            .ok_or_else(|| usage("topology-living-bench requires --export-dir <path>"))?,
        format: format
            .ok_or_else(|| usage("topology-living-bench requires --format json|markdown"))?,
    })
}

fn parse_p74_topologies(value: &str) -> Result<Vec<TopologyKind>, String> {
    if value == "all" {
        return Ok(crate::p74_all_topologies());
    }
    let mut topologies = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        let topology = TopologyKind::from_str(item).ok_or_else(|| {
            usage(format!(
                "topology-living-bench received unsupported topology '{}'; expected all|linear|cubical|trie|graph|hypergraph|hierarchical",
                item
            ))
        })?;
        topologies.push(topology);
    }
    if topologies.is_empty() {
        return Err(usage("topology-living-bench requires non-empty --topology"));
    }
    Ok(topologies)
}

fn parse_p74_compact_value(value: &str) -> Result<P74CompactionPolicy, String> {
    P74CompactionPolicy::from_str(value).ok_or_else(|| {
        usage(format!(
            "topology-living-bench received unsupported compact policy '{}'; expected off|threshold|aggressive|adaptive",
            value
        ))
    })
}

fn parse_p74_locality(value: &str) -> Result<P74LocalityProfile, String> {
    P74LocalityProfile::from_str(value).ok_or_else(|| {
        usage(format!(
            "topology-living-bench received unsupported locality '{}'; expected clustered|random|mixed|hotspot",
            value
        ))
    })
}

fn parse_p74_update_pressure(value: &str) -> Result<P74UpdatePressure, String> {
    P74UpdatePressure::from_str(value).ok_or_else(|| {
        usage(format!(
            "topology-living-bench received unsupported update pressure '{}'; expected low|medium|high",
            value
        ))
    })
}

fn parse_p75_router_living_options(args: &[String]) -> Result<P75RouterLivingCliOptions, String> {
    let mut corpora = None;
    let mut router = None;
    let mut target_source_bytes = None;
    let mut cycles = None;
    let mut queries = None;
    let mut updates = None;
    let mut deletes = None;
    let mut compact = None;
    let mut adaptive = None;
    let mut locality = None;
    let mut update_pressure = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--corpus" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --corpus"))?;
            corpora = Some(parse_p71_corpora(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--corpus=") {
            corpora = Some(parse_p71_corpora(value)?);
            idx += 1;
        } else if arg == "--router" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --router"))?;
            router = Some(parse_p75_router_policy(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--router=") {
            router = Some(parse_p75_router_policy(value)?);
            idx += 1;
        } else if arg == "--target-source-bytes" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("mixed-topology-bench requires a value after --target-source-bytes")
            })?;
            target_source_bytes = Some(parse_positive_u64(
                value,
                "mixed-topology-bench",
                "--target-source-bytes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--target-source-bytes=") {
            target_source_bytes = Some(parse_positive_u64(
                value,
                "mixed-topology-bench",
                "--target-source-bytes",
            )?);
            idx += 1;
        } else if arg == "--cycles" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --cycles"))?;
            cycles = Some(parse_positive_usize(
                value,
                "mixed-topology-bench",
                "--cycles",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cycles=") {
            cycles = Some(parse_positive_usize(
                value,
                "mixed-topology-bench",
                "--cycles",
            )?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --queries"))?;
            queries = Some(parse_positive_usize(
                value,
                "mixed-topology-bench",
                "--queries",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(
                value,
                "mixed-topology-bench",
                "--queries",
            )?);
            idx += 1;
        } else if arg == "--updates" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --updates"))?;
            updates = Some(parse_nonnegative_usize(
                value,
                "mixed-topology-bench",
                "--updates",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--updates=") {
            updates = Some(parse_nonnegative_usize(
                value,
                "mixed-topology-bench",
                "--updates",
            )?);
            idx += 1;
        } else if arg == "--deletes" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --deletes"))?;
            deletes = Some(parse_nonnegative_usize(
                value,
                "mixed-topology-bench",
                "--deletes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--deletes=") {
            deletes = Some(parse_nonnegative_usize(
                value,
                "mixed-topology-bench",
                "--deletes",
            )?);
            idx += 1;
        } else if arg == "--compact" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --compact"))?;
            compact = Some(parse_p74_compact_value(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--compact=") {
            compact = Some(parse_p74_compact_value(value)?);
            idx += 1;
        } else if arg == "--adaptive" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --adaptive"))?;
            adaptive = Some(parse_bool_value(
                value,
                "mixed-topology-bench",
                "--adaptive",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--adaptive=") {
            adaptive = Some(parse_bool_value(
                value,
                "mixed-topology-bench",
                "--adaptive",
            )?);
            idx += 1;
        } else if arg == "--locality" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --locality"))?;
            locality = Some(parse_p74_locality(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--locality=") {
            locality = Some(parse_p74_locality(value)?);
            idx += 1;
        } else if arg == "--update-pressure" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("mixed-topology-bench requires a value after --update-pressure")
            })?;
            update_pressure = Some(parse_p74_update_pressure(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--update-pressure=") {
            update_pressure = Some(parse_p74_update_pressure(value)?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("mixed-topology-bench requires a value after --format"))?;
            format = Some(parse_format_value(value, "mixed-topology-bench")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "mixed-topology-bench")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "mixed-topology-bench received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P75RouterLivingCliOptions {
        options: RouterLivingOptions {
            corpora: corpora.ok_or_else(|| {
                usage("mixed-topology-bench requires --corpus all|code|logs|json|csv|guard")
            })?,
            router: router.ok_or_else(|| {
                usage(
                    "mixed-topology-bench requires --router mixed|hierarchical-only|linear-only|cubical-only|trie-only|graph-only|hypergraph-only",
                )
            })?,
            target_source_bytes: target_source_bytes
                .ok_or_else(|| usage("mixed-topology-bench requires --target-source-bytes N"))?,
            cycles: cycles.ok_or_else(|| usage("mixed-topology-bench requires --cycles N"))?,
            queries: queries.ok_or_else(|| usage("mixed-topology-bench requires --queries N"))?,
            updates: updates.ok_or_else(|| usage("mixed-topology-bench requires --updates N"))?,
            deletes: deletes.ok_or_else(|| usage("mixed-topology-bench requires --deletes N"))?,
            compact: compact.ok_or_else(|| {
                usage("mixed-topology-bench requires --compact off|threshold|aggressive|adaptive")
            })?,
            adaptive: adaptive
                .ok_or_else(|| usage("mixed-topology-bench requires --adaptive on|off"))?,
            locality: locality.ok_or_else(|| {
                usage("mixed-topology-bench requires --locality clustered|random|mixed|hotspot")
            })?,
            update_pressure: update_pressure.ok_or_else(|| {
                usage("mixed-topology-bench requires --update-pressure low|medium|high")
            })?,
        },
        export_dir: export_dir
            .ok_or_else(|| usage("mixed-topology-bench requires --export-dir <path>"))?,
        format: format
            .ok_or_else(|| usage("mixed-topology-bench requires --format json|markdown"))?,
    })
}

fn parse_p75_router_policy(value: &str) -> Result<RouterPolicy, String> {
    RouterPolicy::from_str(value).ok_or_else(|| {
        usage(format!(
            "mixed-topology-bench received unsupported router '{}'; expected mixed|hierarchical-only|linear-only|cubical-only|trie-only|graph-only|hypergraph-only",
            value
        ))
    })
}

fn parse_p76_routing_oracle_options(args: &[String]) -> Result<P76RoutingOracleCliOptions, String> {
    let mut corpora = None;
    let mut target_source_bytes = None;
    let mut cycles = None;
    let mut queries = None;
    let mut updates = None;
    let mut deletes = None;
    let mut locality_profiles = None;
    let mut update_pressures = None;
    let mut compare = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--corpus" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-bench requires a value after --corpus"))?;
            corpora = Some(parse_p71_corpora(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--corpus=") {
            corpora = Some(parse_p71_corpora(value)?);
            idx += 1;
        } else if arg == "--target-source-bytes" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("routing-oracle-bench requires a value after --target-source-bytes")
            })?;
            target_source_bytes = Some(parse_positive_u64(
                value,
                "routing-oracle-bench",
                "--target-source-bytes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--target-source-bytes=") {
            target_source_bytes = Some(parse_positive_u64(
                value,
                "routing-oracle-bench",
                "--target-source-bytes",
            )?);
            idx += 1;
        } else if arg == "--cycles" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-bench requires a value after --cycles"))?;
            cycles = Some(parse_positive_usize(
                value,
                "routing-oracle-bench",
                "--cycles",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cycles=") {
            cycles = Some(parse_positive_usize(
                value,
                "routing-oracle-bench",
                "--cycles",
            )?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-bench requires a value after --queries"))?;
            queries = Some(parse_positive_usize(
                value,
                "routing-oracle-bench",
                "--queries",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(
                value,
                "routing-oracle-bench",
                "--queries",
            )?);
            idx += 1;
        } else if arg == "--updates" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-bench requires a value after --updates"))?;
            updates = Some(parse_nonnegative_usize(
                value,
                "routing-oracle-bench",
                "--updates",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--updates=") {
            updates = Some(parse_nonnegative_usize(
                value,
                "routing-oracle-bench",
                "--updates",
            )?);
            idx += 1;
        } else if arg == "--deletes" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-bench requires a value after --deletes"))?;
            deletes = Some(parse_nonnegative_usize(
                value,
                "routing-oracle-bench",
                "--deletes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--deletes=") {
            deletes = Some(parse_nonnegative_usize(
                value,
                "routing-oracle-bench",
                "--deletes",
            )?);
            idx += 1;
        } else if arg == "--locality" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-bench requires a value after --locality"))?;
            locality_profiles = Some(parse_p76_localities(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--locality=") {
            locality_profiles = Some(parse_p76_localities(value)?);
            idx += 1;
        } else if arg == "--update-pressure" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("routing-oracle-bench requires a value after --update-pressure")
            })?;
            update_pressures = Some(parse_p76_update_pressures(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--update-pressure=") {
            update_pressures = Some(parse_p76_update_pressures(value)?);
            idx += 1;
        } else if arg == "--compare" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-bench requires a value after --compare"))?;
            compare = Some(parse_p76_compare_targets(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--compare=") {
            compare = Some(parse_p76_compare_targets(value)?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-bench requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-bench requires a value after --format"))?;
            format = Some(parse_format_value(value, "routing-oracle-bench")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "routing-oracle-bench")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "routing-oracle-bench received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P76RoutingOracleCliOptions {
        options: RoutingOracleOptions {
            corpora: corpora.ok_or_else(|| {
                usage("routing-oracle-bench requires --corpus all|code|logs|json|csv|guard")
            })?,
            target_source_bytes: target_source_bytes
                .ok_or_else(|| usage("routing-oracle-bench requires --target-source-bytes N"))?,
            cycles: cycles.ok_or_else(|| usage("routing-oracle-bench requires --cycles N"))?,
            queries: queries.ok_or_else(|| usage("routing-oracle-bench requires --queries N"))?,
            updates: updates.ok_or_else(|| usage("routing-oracle-bench requires --updates N"))?,
            deletes: deletes.ok_or_else(|| usage("routing-oracle-bench requires --deletes N"))?,
            locality_profiles: locality_profiles
                .ok_or_else(|| usage("routing-oracle-bench requires --locality all|clustered|random|mixed|hotspot"))?,
            update_pressures: update_pressures
                .ok_or_else(|| usage("routing-oracle-bench requires --update-pressure all|low|medium|high"))?,
            compare: compare.ok_or_else(|| {
                usage("routing-oracle-bench requires --compare oracle,mixed,hierarchical,linear,cubical,trie,graph,hypergraph")
            })?,
        },
        export_dir: export_dir
            .ok_or_else(|| usage("routing-oracle-bench requires --export-dir <path>"))?,
        format: format
            .ok_or_else(|| usage("routing-oracle-bench requires --format json|markdown"))?,
    })
}

fn parse_p77_router_calibration_options(
    args: &[String],
) -> Result<P77RouterCalibrationCliOptions, String> {
    let mut corpora = None;
    let mut target_source_bytes = None;
    let mut cycles = None;
    let mut queries = None;
    let mut updates = None;
    let mut deletes = None;
    let mut locality_profiles = None;
    let mut update_pressures = None;
    let mut grid_kind = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--corpus" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-calibrate requires a value after --corpus"))?;
            corpora = Some(parse_p71_corpora(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--corpus=") {
            corpora = Some(parse_p71_corpora(value)?);
            idx += 1;
        } else if arg == "--target-source-bytes" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("routing-oracle-calibrate requires a value after --target-source-bytes")
            })?;
            target_source_bytes = Some(parse_positive_u64(
                value,
                "routing-oracle-calibrate",
                "--target-source-bytes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--target-source-bytes=") {
            target_source_bytes = Some(parse_positive_u64(
                value,
                "routing-oracle-calibrate",
                "--target-source-bytes",
            )?);
            idx += 1;
        } else if arg == "--cycles" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-calibrate requires a value after --cycles"))?;
            cycles = Some(parse_positive_usize(
                value,
                "routing-oracle-calibrate",
                "--cycles",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cycles=") {
            cycles = Some(parse_positive_usize(
                value,
                "routing-oracle-calibrate",
                "--cycles",
            )?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("routing-oracle-calibrate requires a value after --queries")
            })?;
            queries = Some(parse_positive_usize(
                value,
                "routing-oracle-calibrate",
                "--queries",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(
                value,
                "routing-oracle-calibrate",
                "--queries",
            )?);
            idx += 1;
        } else if arg == "--updates" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("routing-oracle-calibrate requires a value after --updates")
            })?;
            updates = Some(parse_nonnegative_usize(
                value,
                "routing-oracle-calibrate",
                "--updates",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--updates=") {
            updates = Some(parse_nonnegative_usize(
                value,
                "routing-oracle-calibrate",
                "--updates",
            )?);
            idx += 1;
        } else if arg == "--deletes" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("routing-oracle-calibrate requires a value after --deletes")
            })?;
            deletes = Some(parse_nonnegative_usize(
                value,
                "routing-oracle-calibrate",
                "--deletes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--deletes=") {
            deletes = Some(parse_nonnegative_usize(
                value,
                "routing-oracle-calibrate",
                "--deletes",
            )?);
            idx += 1;
        } else if arg == "--locality" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("routing-oracle-calibrate requires a value after --locality")
            })?;
            locality_profiles = Some(parse_p76_localities(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--locality=") {
            locality_profiles = Some(parse_p76_localities(value)?);
            idx += 1;
        } else if arg == "--update-pressure" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("routing-oracle-calibrate requires a value after --update-pressure")
            })?;
            update_pressures = Some(parse_p76_update_pressures(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--update-pressure=") {
            update_pressures = Some(parse_p76_update_pressures(value)?);
            idx += 1;
        } else if arg == "--grid" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-calibrate requires a value after --grid"))?;
            grid_kind = Some(parse_p77_grid_kind(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--grid=") {
            grid_kind = Some(parse_p77_grid_kind(value)?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("routing-oracle-calibrate requires a value after --export-dir")
            })?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("routing-oracle-calibrate requires a value after --format"))?;
            format = Some(parse_format_value(value, "routing-oracle-calibrate")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "routing-oracle-calibrate")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "routing-oracle-calibrate received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P77RouterCalibrationCliOptions {
        options: P77RouterCalibrationOptions {
            corpora: corpora.ok_or_else(|| {
                usage("routing-oracle-calibrate requires --corpus all|code|logs|json|csv|guard")
            })?,
            target_source_bytes: target_source_bytes.ok_or_else(|| {
                usage("routing-oracle-calibrate requires --target-source-bytes N")
            })?,
            cycles: cycles
                .ok_or_else(|| usage("routing-oracle-calibrate requires --cycles N"))?,
            queries: queries
                .ok_or_else(|| usage("routing-oracle-calibrate requires --queries N"))?,
            updates: updates
                .ok_or_else(|| usage("routing-oracle-calibrate requires --updates N"))?,
            deletes: deletes
                .ok_or_else(|| usage("routing-oracle-calibrate requires --deletes N"))?,
            locality_profiles: locality_profiles.ok_or_else(|| {
                usage("routing-oracle-calibrate requires --locality all|clustered|random|mixed|hotspot")
            })?,
            update_pressures: update_pressures.ok_or_else(|| {
                usage("routing-oracle-calibrate requires --update-pressure all|low|medium|high")
            })?,
            grid_kind: grid_kind.ok_or_else(|| {
                usage("routing-oracle-calibrate requires --grid smoke|standard|focused|wide")
            })?,
        },
        export_dir: export_dir
            .ok_or_else(|| usage("routing-oracle-calibrate requires --export-dir <path>"))?,
        format: format
            .ok_or_else(|| usage("routing-oracle-calibrate requires --format json|markdown"))?,
    })
}

fn parse_p76_virtual_space_estimate_options(
    args: &[String],
) -> Result<P76VirtualSpaceEstimateCliOptions, String> {
    let mut topology = None;
    let mut target_source_bytes = None;
    let mut cells = None;
    let mut fibers_per_cell = None;
    let mut hierarchy_depth = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--topology" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("virtual-space-estimate requires a value after --topology"))?;
            topology = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--topology=") {
            topology = Some(value.to_string());
            idx += 1;
        } else if arg == "--target-source-bytes" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("virtual-space-estimate requires a value after --target-source-bytes")
            })?;
            target_source_bytes = Some(parse_positive_u64(
                value,
                "virtual-space-estimate",
                "--target-source-bytes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--target-source-bytes=") {
            target_source_bytes = Some(parse_positive_u64(
                value,
                "virtual-space-estimate",
                "--target-source-bytes",
            )?);
            idx += 1;
        } else if arg == "--cells" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("virtual-space-estimate requires a value after --cells"))?;
            cells = Some(parse_positive_u64(
                value,
                "virtual-space-estimate",
                "--cells",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cells=") {
            cells = Some(parse_positive_u64(
                value,
                "virtual-space-estimate",
                "--cells",
            )?);
            idx += 1;
        } else if arg == "--fibers-per-cell" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("virtual-space-estimate requires a value after --fibers-per-cell")
            })?;
            fibers_per_cell = Some(parse_positive_u64(
                value,
                "virtual-space-estimate",
                "--fibers-per-cell",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--fibers-per-cell=") {
            fibers_per_cell = Some(parse_positive_u64(
                value,
                "virtual-space-estimate",
                "--fibers-per-cell",
            )?);
            idx += 1;
        } else if arg == "--hierarchy-depth" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("virtual-space-estimate requires a value after --hierarchy-depth")
            })?;
            hierarchy_depth = Some(parse_positive_u64(
                value,
                "virtual-space-estimate",
                "--hierarchy-depth",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--hierarchy-depth=") {
            hierarchy_depth = Some(parse_positive_u64(
                value,
                "virtual-space-estimate",
                "--hierarchy-depth",
            )?);
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("virtual-space-estimate requires a value after --format"))?;
            format = Some(parse_format_value(value, "virtual-space-estimate")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "virtual-space-estimate")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "virtual-space-estimate received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P76VirtualSpaceEstimateCliOptions {
        options: P76VirtualSpaceEstimateOptions {
            topology: topology
                .ok_or_else(|| usage("virtual-space-estimate requires --topology <name>"))?,
            target_source_bytes: target_source_bytes
                .ok_or_else(|| usage("virtual-space-estimate requires --target-source-bytes N"))?,
            cells: cells.ok_or_else(|| usage("virtual-space-estimate requires --cells N"))?,
            fibers_per_cell: fibers_per_cell
                .ok_or_else(|| usage("virtual-space-estimate requires --fibers-per-cell N"))?,
            hierarchy_depth: hierarchy_depth
                .ok_or_else(|| usage("virtual-space-estimate requires --hierarchy-depth N"))?,
        },
        format: format
            .ok_or_else(|| usage("virtual-space-estimate requires --format json|markdown"))?,
    })
}

fn parse_p78_level1_space_options(args: &[String]) -> Result<P78Level1SpaceCliOptions, String> {
    let mut corpora = None;
    let mut level1_topologies = None;
    let mut fiber_router = None;
    let mut target_source_bytes = None;
    let mut cycles = None;
    let mut queries = None;
    let mut updates = None;
    let mut deletes = None;
    let mut compact = None;
    let mut adaptive = None;
    let mut export_dir = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--corpus" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-bench requires a value after --corpus"))?;
            corpora = Some(parse_p71_corpora(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--corpus=") {
            corpora = Some(parse_p71_corpora(value)?);
            idx += 1;
        } else if arg == "--level1-topology" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("level1-space-bench requires a value after --level1-topology")
            })?;
            level1_topologies = Some(parse_p78_level1_topologies(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--level1-topology=") {
            level1_topologies = Some(parse_p78_level1_topologies(value)?);
            idx += 1;
        } else if arg == "--fiber-router" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-bench requires a value after --fiber-router"))?;
            fiber_router = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--fiber-router=") {
            fiber_router = Some(value.to_string());
            idx += 1;
        } else if arg == "--target-source-bytes" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("level1-space-bench requires a value after --target-source-bytes")
            })?;
            target_source_bytes = Some(parse_positive_u64(
                value,
                "level1-space-bench",
                "--target-source-bytes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--target-source-bytes=") {
            target_source_bytes = Some(parse_positive_u64(
                value,
                "level1-space-bench",
                "--target-source-bytes",
            )?);
            idx += 1;
        } else if arg == "--cycles" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-bench requires a value after --cycles"))?;
            cycles = Some(parse_positive_usize(
                value,
                "level1-space-bench",
                "--cycles",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--cycles=") {
            cycles = Some(parse_positive_usize(
                value,
                "level1-space-bench",
                "--cycles",
            )?);
            idx += 1;
        } else if arg == "--queries" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-bench requires a value after --queries"))?;
            queries = Some(parse_positive_usize(
                value,
                "level1-space-bench",
                "--queries",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--queries=") {
            queries = Some(parse_positive_usize(
                value,
                "level1-space-bench",
                "--queries",
            )?);
            idx += 1;
        } else if arg == "--updates" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-bench requires a value after --updates"))?;
            updates = Some(parse_nonnegative_usize(
                value,
                "level1-space-bench",
                "--updates",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--updates=") {
            updates = Some(parse_nonnegative_usize(
                value,
                "level1-space-bench",
                "--updates",
            )?);
            idx += 1;
        } else if arg == "--deletes" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-bench requires a value after --deletes"))?;
            deletes = Some(parse_nonnegative_usize(
                value,
                "level1-space-bench",
                "--deletes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--deletes=") {
            deletes = Some(parse_nonnegative_usize(
                value,
                "level1-space-bench",
                "--deletes",
            )?);
            idx += 1;
        } else if arg == "--compact" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-bench requires a value after --compact"))?;
            compact = Some(parse_p74_compact_value(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--compact=") {
            compact = Some(parse_p74_compact_value(value)?);
            idx += 1;
        } else if arg == "--adaptive" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-bench requires a value after --adaptive"))?;
            adaptive = Some(parse_bool_value(value, "level1-space-bench", "--adaptive")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--adaptive=") {
            adaptive = Some(parse_bool_value(value, "level1-space-bench", "--adaptive")?);
            idx += 1;
        } else if arg == "--export-dir" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-bench requires a value after --export-dir"))?;
            export_dir = Some(value.to_string());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--export-dir=") {
            export_dir = Some(value.to_string());
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-bench requires a value after --format"))?;
            format = Some(parse_format_value(value, "level1-space-bench")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "level1-space-bench")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "level1-space-bench received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P78Level1SpaceCliOptions {
        options: P78Level1SpaceOptions {
            corpora: corpora.ok_or_else(|| {
                usage("level1-space-bench requires --corpus all|code|logs|json|csv|guard")
            })?,
            level1_topologies: level1_topologies.ok_or_else(|| {
                usage("level1-space-bench requires --level1-topology all|grid2d|grid3d|tree|path-trie|content-dag|graph|product-typed|hybrid-multi-index")
            })?,
            fiber_router: fiber_router
                .ok_or_else(|| usage("level1-space-bench requires --fiber-router <id>"))?,
            target_source_bytes: target_source_bytes
                .ok_or_else(|| usage("level1-space-bench requires --target-source-bytes N"))?,
            cycles: cycles.ok_or_else(|| usage("level1-space-bench requires --cycles N"))?,
            queries: queries.ok_or_else(|| usage("level1-space-bench requires --queries N"))?,
            updates: updates.ok_or_else(|| usage("level1-space-bench requires --updates N"))?,
            deletes: deletes.ok_or_else(|| usage("level1-space-bench requires --deletes N"))?,
            compact: compact
                .ok_or_else(|| usage("level1-space-bench requires --compact off|threshold|aggressive|adaptive"))?,
            adaptive: adaptive.ok_or_else(|| usage("level1-space-bench requires --adaptive on|off"))?,
        },
        export_dir: export_dir
            .ok_or_else(|| usage("level1-space-bench requires --export-dir <path>"))?,
        format: format.ok_or_else(|| usage("level1-space-bench requires --format json|markdown"))?,
    })
}

fn parse_p78_level1_space_estimate_options(
    args: &[String],
) -> Result<P78Level1EstimateCliOptions, String> {
    let mut topology_kind = None;
    let mut target_source_bytes = None;
    let mut address_bits = None;
    let mut file_type_count = None;
    let mut object_count = None;
    let mut chunk_count = None;
    let mut version_count = None;
    let mut fibers_per_object = None;
    let mut format = None;
    let mut idx = 0;

    while idx < args.len() {
        let arg = args[idx].as_str();
        if arg == "--level1-topology" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("level1-space-estimate requires a value after --level1-topology")
            })?;
            topology_kind = Some(parse_p78_level1_topology(value)?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--level1-topology=") {
            topology_kind = Some(parse_p78_level1_topology(value)?);
            idx += 1;
        } else if arg == "--target-source-bytes" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("level1-space-estimate requires a value after --target-source-bytes")
            })?;
            target_source_bytes = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--target-source-bytes",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--target-source-bytes=") {
            target_source_bytes = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--target-source-bytes",
            )?);
            idx += 1;
        } else if arg == "--address-bits" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("level1-space-estimate requires a value after --address-bits")
            })?;
            address_bits = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--address-bits",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--address-bits=") {
            address_bits = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--address-bits",
            )?);
            idx += 1;
        } else if arg == "--file-type-count" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("level1-space-estimate requires a value after --file-type-count")
            })?;
            file_type_count = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--file-type-count",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--file-type-count=") {
            file_type_count = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--file-type-count",
            )?);
            idx += 1;
        } else if arg == "--object-count" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("level1-space-estimate requires a value after --object-count")
            })?;
            object_count = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--object-count",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--object-count=") {
            object_count = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--object-count",
            )?);
            idx += 1;
        } else if arg == "--chunk-count" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("level1-space-estimate requires a value after --chunk-count")
            })?;
            chunk_count = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--chunk-count",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--chunk-count=") {
            chunk_count = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--chunk-count",
            )?);
            idx += 1;
        } else if arg == "--version-count" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("level1-space-estimate requires a value after --version-count")
            })?;
            version_count = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--version-count",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--version-count=") {
            version_count = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--version-count",
            )?);
            idx += 1;
        } else if arg == "--fibers-per-object" {
            let value = args.get(idx + 1).ok_or_else(|| {
                usage("level1-space-estimate requires a value after --fibers-per-object")
            })?;
            fibers_per_object = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--fibers-per-object",
            )?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--fibers-per-object=") {
            fibers_per_object = Some(parse_positive_u64(
                value,
                "level1-space-estimate",
                "--fibers-per-object",
            )?);
            idx += 1;
        } else if arg == "--format" {
            let value = args
                .get(idx + 1)
                .ok_or_else(|| usage("level1-space-estimate requires a value after --format"))?;
            format = Some(parse_format_value(value, "level1-space-estimate")?);
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(parse_format_value(value, "level1-space-estimate")?);
            idx += 1;
        } else {
            return Err(usage(format!(
                "level1-space-estimate received unsupported option '{}'",
                arg
            )));
        }
    }

    Ok(P78Level1EstimateCliOptions {
        options: Level1VirtualSpaceEstimateOptions {
            topology_kind: topology_kind
                .ok_or_else(|| usage("level1-space-estimate requires --level1-topology <kind>"))?,
            target_source_bytes: target_source_bytes
                .ok_or_else(|| usage("level1-space-estimate requires --target-source-bytes N"))?,
            address_bits: address_bits
                .ok_or_else(|| usage("level1-space-estimate requires --address-bits N"))?,
            file_type_count: file_type_count
                .ok_or_else(|| usage("level1-space-estimate requires --file-type-count N"))?,
            object_count: object_count
                .ok_or_else(|| usage("level1-space-estimate requires --object-count N"))?,
            chunk_count: chunk_count
                .ok_or_else(|| usage("level1-space-estimate requires --chunk-count N"))?,
            version_count: version_count
                .ok_or_else(|| usage("level1-space-estimate requires --version-count N"))?,
            fibers_per_object: fibers_per_object
                .ok_or_else(|| usage("level1-space-estimate requires --fibers-per-object N"))?,
        },
        format: format
            .ok_or_else(|| usage("level1-space-estimate requires --format json|markdown"))?,
    })
}

fn parse_p76_localities(value: &str) -> Result<Vec<P74LocalityProfile>, String> {
    if value == "all" {
        return Ok(P74LocalityProfile::all());
    }
    let mut parsed = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        parsed.push(P74LocalityProfile::from_str(item).ok_or_else(|| {
            usage(format!(
                "routing-oracle-bench received unsupported locality '{}'; expected all|clustered|random|mixed|hotspot",
                item
            ))
        })?);
    }
    Ok(parsed)
}

fn parse_p78_level1_topologies(value: &str) -> Result<Vec<Level1TopologyKind>, String> {
    if value == "all" {
        return Ok(crate::p78_all_level1_topologies());
    }
    let mut topologies = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        topologies.push(parse_p78_level1_topology(item)?);
    }
    if topologies.is_empty() {
        return Err(usage(
            "level1-space-bench requires non-empty --level1-topology",
        ));
    }
    Ok(topologies)
}

fn parse_p78_level1_topology(value: &str) -> Result<Level1TopologyKind, String> {
    Level1TopologyKind::from_str(value).ok_or_else(|| {
        usage(format!(
            "level1-space received unsupported topology '{}'; expected all|grid2d|grid3d|tree|path-trie|content-dag|graph|product-typed|hybrid-multi-index",
            value
        ))
    })
}

fn parse_p76_update_pressures(value: &str) -> Result<Vec<P74UpdatePressure>, String> {
    if value == "all" {
        return Ok(P74UpdatePressure::all());
    }
    let mut parsed = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        parsed.push(P74UpdatePressure::from_str(item).ok_or_else(|| {
            usage(format!(
                "routing-oracle-bench received unsupported update pressure '{}'; expected all|low|medium|high",
                item
            ))
        })?);
    }
    Ok(parsed)
}

fn parse_p76_compare_targets(value: &str) -> Result<Vec<P76CompareTarget>, String> {
    let mut parsed = Vec::new();
    for item in value.split(',') {
        let item = item.trim();
        parsed.push(P76CompareTarget::from_str(item).ok_or_else(|| {
            usage(format!(
                "routing-oracle-bench received unsupported compare target '{}'; expected oracle|mixed|hierarchical|linear|cubical|trie|graph|hypergraph",
                item
            ))
        })?);
    }
    if parsed.is_empty() {
        return Err(usage("routing-oracle-bench requires non-empty --compare"));
    }
    Ok(parsed)
}

fn parse_p77_grid_kind(value: &str) -> Result<P77CalibrationGridKind, String> {
    P77CalibrationGridKind::from_str(value).ok_or_else(|| {
        usage(format!(
            "routing-oracle-calibrate received unsupported grid '{}'; expected smoke|standard|focused|wide",
            value
        ))
    })
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

fn parse_nonnegative_usize(value: &str, command: &str, option: &str) -> Result<usize, String> {
    value.parse::<usize>().map_err(|_| {
        usage(format!(
            "{} received invalid {} '{}'",
            command, option, value
        ))
    })
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
        "  atlas-cli living-store-bench --corpus all|code|logs|json|csv|guard --budget-bytes N --runs N --queries N --updates N --deletes N --compact off|threshold|aggressive --adaptive on|off [--reopen-check on|off] --export-dir <path> --format json|markdown",
        "  atlas-cli cubical-store-bench --corpus all|code|logs|json|csv|guard --budget-bytes N --cycles N --queries N --updates N --deletes N --corruptions N --compact off|threshold|aggressive --adaptive on|off --compare-p72 baseline|off --export-dir <path> --format json|markdown",
        "  atlas-cli topology-living-bench --corpus all|code|logs|json|csv|guard --topology all|linear|cubical|trie|graph|hypergraph|hierarchical --target-source-bytes N --cycles N --queries N --updates N --deletes N --compact off|threshold|aggressive|adaptive --adaptive on|off --locality clustered|random|mixed|hotspot --update-pressure low|medium|high --export-dir <path> --format json|markdown",
        "  atlas-cli mixed-topology-bench --corpus all|code|logs|json|csv|guard --router mixed|hierarchical-only|linear-only|cubical-only|trie-only|graph-only|hypergraph-only --target-source-bytes N --cycles N --queries N --updates N --deletes N --compact off|threshold|aggressive|adaptive --adaptive on|off --locality clustered|random|mixed|hotspot --update-pressure low|medium|high --export-dir <path> --format json|markdown",
        "  atlas-cli routing-oracle-bench --corpus all|code|logs|json|csv|guard --target-source-bytes N --cycles N --queries N --updates N --deletes N --locality all|clustered|random|mixed|hotspot --update-pressure all|low|medium|high --compare oracle,mixed,hierarchical,linear,cubical,trie,graph,hypergraph --export-dir <path> --format json|markdown",
        "  atlas-cli routing-oracle-calibrate --corpus all|code|logs|json|csv|guard --target-source-bytes N --cycles N --queries N --updates N --deletes N --locality all|clustered|random|mixed|hotspot --update-pressure all|low|medium|high --grid smoke|standard|focused|wide --export-dir <path> --format json|markdown",
        "  atlas-cli virtual-space-estimate --topology mixed|linear|cubical|trie|graph|hypergraph|hierarchical --target-source-bytes N --cells N --fibers-per-cell N --hierarchy-depth N --format json|markdown",
        "  atlas-cli level1-space-bench --corpus all|code|logs|json|csv|guard --level1-topology all|grid2d|grid3d|tree|path-trie|content-dag|graph|product-typed|hybrid-multi-index --fiber-router p77-calibrated --target-source-bytes N --cycles N --queries N --updates N --deletes N --compact off|threshold|aggressive|adaptive --adaptive on|off --export-dir <path> --format json|markdown",
        "  atlas-cli level1-space-estimate --level1-topology grid2d|grid3d|tree|path-trie|content-dag|graph|product-typed|hybrid-multi-index --target-source-bytes N --address-bits N --file-type-count N --object-count N --chunk-count N --version-count N --fibers-per-object N --format json|markdown",
    ];
    format!("{}\n{}", detail.as_ref(), commands.join("\n"))
}
