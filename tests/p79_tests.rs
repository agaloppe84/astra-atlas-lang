use astra_atlas_lang::{
    p71_all_corpora, p79_all_level1_router_topologies, p79_default_route_policy,
    p79_level1_router_bench, p79_level1_router_estimate, p79_parse_level1_router_contract_file,
    Level1AddressRouter, Level1FeatureExtractor, Level1TopologyKind, P74CompactionPolicy,
    P79CompareTarget, P79Decision, P79Level1RouterOptions,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID: &str = "examples/valid/p79_level1_router.atlas";

#[test]
fn p79_level1_router_exists_and_all_topologies_are_routable() {
    let topologies = p79_all_level1_router_topologies();
    assert_eq!(topologies.len(), 8);
    assert!(topologies.contains(&Level1TopologyKind::Grid2D));
    assert!(topologies.contains(&Level1TopologyKind::Grid3D));
    assert!(topologies.contains(&Level1TopologyKind::HierarchicalTree));
    assert!(topologies.contains(&Level1TopologyKind::PathTrie));
    assert!(topologies.contains(&Level1TopologyKind::ContentAddressedDag));
    assert!(topologies.contains(&Level1TopologyKind::GraphAddressSpace));
    assert!(topologies.contains(&Level1TopologyKind::ProductTypedSpace));
    assert!(topologies.contains(&Level1TopologyKind::HybridMultiIndexSpace));

    let policy = p79_default_route_policy();
    assert_eq!(policy.path_like_topology, Level1TopologyKind::PathTrie);
    assert_eq!(
        policy.chunked_binary_topology,
        Level1TopologyKind::ContentAddressedDag
    );
    assert_eq!(
        policy.typed_namespace_topology,
        Level1TopologyKind::ProductTypedSpace
    );
    assert_eq!(
        policy.relation_heavy_topology,
        Level1TopologyKind::GraphAddressSpace
    );
    assert_eq!(
        policy.multi_access_topology,
        Level1TopologyKind::HybridMultiIndexSpace
    );
}

#[test]
fn p79_router_routes_expected_feature_families() {
    let router = Level1AddressRouter::new(p79_default_route_policy());
    let extractor = Level1FeatureExtractor::p79();
    let features = extractor.extract(&p71_all_corpora(), 1_048_576);

    let path = features
        .iter()
        .find(|feature| feature.feature_id == "code_path_modules")
        .expect("path-like feature");
    let json = features
        .iter()
        .find(|feature| feature.feature_id == "json_typed_namespace")
        .expect("typed namespace feature");
    let csv = features
        .iter()
        .find(|feature| feature.feature_id == "csv_regular_projection")
        .expect("regular grid feature");
    let guard = features
        .iter()
        .find(|feature| feature.guard_flag)
        .expect("guard feature");

    assert_eq!(
        router.route(path).selected_level1_topology,
        Some(Level1TopologyKind::PathTrie)
    );
    assert_eq!(
        router.route(json).selected_level1_topology,
        Some(Level1TopologyKind::ProductTypedSpace)
    );
    assert_eq!(
        router.route(csv).selected_level1_topology,
        Some(Level1TopologyKind::Grid3D)
    );
    let guard_route = router.route(guard);
    assert!(guard_route.guard_refused);
    assert_eq!(guard_route.selected_level1_topology, None);
}

#[test]
fn p79_valid_contract_parses_and_invalids_are_refused() {
    let contract = p79_parse_level1_router_contract_file(VALID).expect("valid P79 parses");
    assert_eq!(contract.policy.policy_id, "p79-router");
    assert!(contract.living_memory_only);
    assert!(contract.local_on_address);
    assert_eq!(contract.virtual_bytes_claim, "equivalent");

    let invalids = [
        "examples/invalid/p79_unknown_level1_route.atlas",
        "examples/invalid/p79_missing_guard_policy.atlas",
        "examples/invalid/p79_hidden_level1_index_storage.atlas",
        "examples/invalid/p79_non_living_level1_decision.atlas",
        "examples/invalid/p79_missing_router_oracle_gate.atlas",
        "examples/invalid/p79_unbounded_address_lookup.atlas",
        "examples/invalid/p79_virtual_bytes_claim_stored.atlas",
        "examples/invalid/p79_missing_local_on_address.atlas",
    ];
    for invalid in invalids {
        assert!(
            p79_parse_level1_router_contract_file(invalid).is_err(),
            "{} should be refused",
            invalid
        );
    }
}

#[test]
fn p79_level1_router_bench_produces_living_metrics_and_oracle() {
    let report = p79_report(&temp_dir("bench"));

    assert_eq!(report.astra_step, "P79");
    assert_eq!(report.actual_source_bytes, 1_048_576);
    assert!(report.comparison.router_result.ratio_living > 4.0);
    assert!(
        report.comparison.hybrid_only_result.ratio_living
            > report.comparison.router_result.ratio_living
    );
    assert!(report.comparison.router_result.address_lookup_p95_steps <= 8.0);
    assert!(
        report.index_cost_report.level1_router_index_bytes
            < report.index_cost_report.level1_hybrid_index_bytes
    );
    assert!(report.oracle_report.level1_wrong_route_count > 0);
    assert!(report.oracle_report.level1_wrong_route_cost > 0);
    assert_eq!(report.comparison.router_result.crud_success_rate, 1.0);
    assert_eq!(report.comparison.router_result.retrieval_success_rate, 1.0);
    assert!(report.comparison.router_result.reopen_equivalence);
    assert_eq!(report.comparison.router_result.drift_status, "NO_DRIFT");
    assert_eq!(
        report.comparison.router_result.guard_decision,
        "NO_GO_GUARD_INCOMPRESSIBLE_REFUSED"
    );
    assert!(report.virtual_space_metrics.bytes_are_equivalent_not_stored);
    assert_eq!(report.decision, P79Decision::RecalibrateLevel1RouterPolicy);
}

#[test]
fn p79_exports_expected_artifacts_and_phase_map() {
    let export_dir = temp_dir("exports");
    let report = p79_report(&export_dir);

    assert!(export_dir.join("p79_level1_router_report.json").exists());
    assert!(export_dir.join("p79_level1_routes.jsonl").exists());
    assert!(export_dir.join("p79_level1_oracle.jsonl").exists());
    assert!(export_dir
        .join("p79_level1_topology_comparison.csv")
        .exists());
    assert!(export_dir.join("p79_level1_index_cost.csv").exists());
    assert!(export_dir.join("p79_addressing_metrics.csv").exists());
    assert!(export_dir.join("p79_virtual_space_metrics.json").exists());
    assert!(export_dir.join("p79_summary.md").exists());
    assert!(export_dir
        .join("stores")
        .join("router")
        .join("cold")
        .exists());
    assert!(export_dir
        .join("stores")
        .join("oracle")
        .join("cold")
        .exists());
    assert!(export_dir
        .join("stores")
        .join("hybrid")
        .join("cold")
        .exists());
    assert!(report.phase_map.green_count > 0);
    assert!(report.phase_map.yellow_count > 0);
    assert!(report.phase_map.red_count > 0);
}

#[test]
fn p79_level1_router_estimate_preserves_virtual_space_metrics() {
    let estimate = p79_level1_router_estimate("p79-router", 1_048_576);

    assert_eq!(estimate.virtual_space_metrics.virtual_cell_count, 10_000);
    assert_eq!(estimate.virtual_space_metrics.virtual_fiber_count, 40_000);
    assert!(
        estimate
            .virtual_space_metrics
            .virtual_effective_bytes_equivalent
            > 1_048_576
    );
    assert!(estimate.bytes_are_equivalent_not_stored);
    assert!(estimate.level1_router_index_bytes > 0);
}

#[test]
fn p79_level1_router_cli_succeeds() {
    let export_dir = temp_dir("cli");
    let output = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "level1-router-bench",
            "--corpus",
            "all",
            "--level1-router",
            "p79-router",
            "--target-source-bytes",
            "1048576",
            "--cycles",
            "2",
            "--queries",
            "500",
            "--updates",
            "50",
            "--deletes",
            "5",
            "--compact",
            "adaptive",
            "--adaptive",
            "on",
            "--compare",
            "router,oracle,hybrid,path-trie,product-typed,content-dag",
            "--export-dir",
            export_dir.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("level1-router-bench");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"astra_step\": \"P79\""));
    assert!(stdout.contains("\"ratio_living_router\""));
    assert!(stdout.contains("\"level1_wrong_route_count\""));
    assert!(export_dir.join("p79_level1_router_report.json").exists());
}

#[test]
fn p79_estimate_cli_and_prior_milestones_remain_available() {
    let estimate = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args([
            "level1-router-estimate",
            "--level1-router",
            "p79-router",
            "--target-source-bytes",
            "1048576",
            "--format",
            "json",
        ])
        .output()
        .expect("level1-router-estimate");
    assert!(
        estimate.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&estimate.stderr)
    );
    assert!(
        String::from_utf8_lossy(&estimate.stdout).contains("virtual_effective_bytes_equivalent")
    );

    let p79 = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["check", VALID])
        .output()
        .expect("p79 check");
    assert!(
        p79.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&p79.stderr)
    );
    assert!(String::from_utf8_lossy(&p79.stdout).contains("p79_level1_router"));

    let p78 = Command::new(env!("CARGO_BIN_EXE_atlas-cli"))
        .args(["check", "examples/valid/p78_level1_virtual_space.atlas"])
        .output()
        .expect("p78 check");
    assert!(
        p78.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&p78.stderr)
    );
}

fn p79_report(export_dir: &std::path::Path) -> astra_atlas_lang::P79Level1RouterReport {
    p79_level1_router_bench(
        P79Level1RouterOptions {
            corpora: p71_all_corpora(),
            level1_router: "p79-router".to_string(),
            target_source_bytes: 1_048_576,
            cycles: 2,
            queries: 500,
            updates: 50,
            deletes: 5,
            compact: P74CompactionPolicy::Adaptive,
            adaptive: true,
            compare: vec![
                P79CompareTarget::Router,
                P79CompareTarget::Oracle,
                P79CompareTarget::HybridOnly,
                P79CompareTarget::PathTrieOnly,
                P79CompareTarget::ProductTypedOnly,
                P79CompareTarget::ContentDagOnly,
            ],
        },
        export_dir,
    )
    .expect("P79 report")
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    path.push(format!(
        "astra_p79_{}_{}_{}",
        label,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}
