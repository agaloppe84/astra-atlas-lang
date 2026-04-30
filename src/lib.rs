use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Debug, Clone)]
pub struct FamilySpec {
    pub name: String,
    pub action: String,
    pub safety: String,
    pub layout: String,
    pub index: String,
    pub threshold: f64,
}

#[derive(Debug, Clone)]
pub struct AtlasProgram {
    pub version: String,
    pub runtime: HashMap<String, String>,
    pub families: Vec<FamilySpec>,
}

fn parse_kv(tokens: &[&str]) -> Result<HashMap<String, String>, String> {
    let mut kv = HashMap::new();
    for tok in tokens {
        if let Some((k, v)) = tok.split_once('=') {
            kv.insert(k.to_string(), v.trim_matches('"').to_string());
        } else {
            return Err(format!("token sans '=': {}", tok));
        }
    }
    Ok(kv)
}

pub fn parse_atlas_str(text: &str) -> Result<AtlasProgram, String> {
    let mut version: Option<String> = None;
    let mut runtime: HashMap<String, String> = HashMap::new();
    let mut families: Vec<FamilySpec> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !line.ends_with(';') {
            return Err(format!("ligne {}: ';' manquant", i + 1));
        }
        let line = &line[..line.len() - 1];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "atlas" => {
                let kv = parse_kv(&parts[1..])?;
                let v = kv.get("version").ok_or("version absente")?;
                if v != "0.1" {
                    return Err("version non supportee".to_string());
                }
                version = Some(v.clone());
            }
            "runtime" => {
                runtime = parse_kv(&parts[1..])?;
            }
            "family" => {
                if parts.len() < 2 {
                    return Err(format!("ligne {}: nom famille absent", i + 1));
                }
                let name = parts[1].to_string();
                let kv = parse_kv(&parts[2..])?;
                let threshold: f64 = kv
                    .get("threshold")
                    .ok_or("threshold absent")?
                    .parse()
                    .map_err(|_| "threshold invalide")?;
                families.push(FamilySpec {
                    name,
                    action: kv.get("action").ok_or("action absente")?.clone(),
                    safety: kv.get("safety").ok_or("safety absente")?.clone(),
                    layout: kv.get("layout").ok_or("layout absent")?.clone(),
                    index: kv.get("index").ok_or("index absent")?.clone(),
                    threshold,
                });
            }
            other => return Err(format!("bloc inconnu: {}", other)),
        }
    }
    Ok(AtlasProgram {
        version: version.ok_or("version absente")?,
        runtime,
        families,
    })
}

pub fn parse_atlas_file(path: &str) -> Result<AtlasProgram, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_atlas_str(&text)
}

pub fn typecheck(program: &AtlasProgram) -> Result<(), String> {
    let expected: HashSet<&str> = [
        "guard",
        "stream_processing",
        "sparse_index",
        "image_field_surrogate",
        "log_request_index",
        "columnar_table",
        "graph_lowrank_surrogate",
        "critical_sparse_archive",
        "compressible_but_wrong",
        "field_surrogate",
        "topological_field",
        "local_global_conflict",
    ]
    .iter()
    .copied()
    .collect();
    let actions: HashSet<&str> = [
        "refuse",
        "stream_delta",
        "sparse_csr",
        "wavelet_tile",
        "log_trie",
        "columnar_delta",
        "low_rank",
        "dict_rom",
        "atlas_sheaf",
    ]
    .iter()
    .copied()
    .collect();
    let safeties: HashSet<&str> = ["refuse", "erasure_light", "mirror_critical"]
        .iter()
        .copied()
        .collect();
    if program.runtime.get("strict_p53").map(|s| s.as_str()) == Some("true") {
        if program.runtime.get("snapshot").map(|s| s.as_str()) != Some("incremental_manifest") {
            return Err("strict_p53 exige snapshot=incremental_manifest".to_string());
        }
    }
    if program.runtime.get("snapshot").map(|s| s.as_str()) == Some("full") {
        return Err("snapshot=full interdit".to_string());
    }
    let mut seen: HashSet<&str> = HashSet::new();
    for f in &program.families {
        if !expected.contains(f.name.as_str()) {
            return Err(format!("famille inconnue: {}", f.name));
        }
        if !seen.insert(f.name.as_str()) {
            return Err(format!("famille dupliquee: {}", f.name));
        }
        if !actions.contains(f.action.as_str()) {
            return Err(format!("action invalide: {}", f.action));
        }
        if !safeties.contains(f.safety.as_str()) {
            return Err(format!("safety invalide: {}", f.safety));
        }
        if f.name == "guard"
            && (f.action != "refuse" || f.safety != "refuse" || f.layout != "refuse")
        {
            return Err("guard_active".to_string());
        }
        if f.action != "refuse" && f.safety == "refuse" {
            return Err("active_without_safety".to_string());
        }
        if f.threshold < 0.05 || f.threshold > 0.50 {
            return Err("threshold_bad".to_string());
        }
    }
    for e in expected {
        if !seen.contains(e) {
            return Err(format!("famille manquante: {}", e));
        }
    }
    Ok(())
}

pub fn validate(text: &str) -> Result<AtlasProgram, String> {
    let p = parse_atlas_str(text)?;
    typecheck(&p)?;
    Ok(p)
}
