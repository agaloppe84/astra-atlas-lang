# ASTRA-P61 Virtual Ratio Lab

## Objet

ASTRA-P61 est une iteration ASTRA classique outillee par le repo Rust. Elle
introduit un premier laboratoire deterministe pour mesurer un ratio virtuel
effectif entre espace adressable et cout systeme reel proxy.

P61 ne change pas la grammaire `.atlas`, ne modifie pas `strict_p53`, ne retire
aucun invalide et ne remplace pas les goldens P57/P58/P60.

## Ratio Principal

Le ratio principal est:

```text
ratio_effective = virtual_effective / real_total_cost_units
```

`ratio_declared` peut etre affiche pour contexte, mais il ne pilote jamais la
decision. L'espace virtuel effectif est borne par l'adressabilite reelle:

```text
virtual_effective
<= virtual_safe
<= virtual_updatable
<= virtual_readable
<= virtual_reachable
<= virtual_declared
```

`virtual_declared` represente l'espace annonce par le proxy de workload.
`virtual_effective` represente uniquement la partie atteignable, lisible,
modifiable, sure et auditable. Un workload peut donc declarer un espace tres
grand et contribuer zero a `virtual_effective` s'il est refuse ou dangereux.

## Modele De Cout

Le modele courant est `deterministic_proxy_v1`. Il additionne:

- payload;
- index;
- journal;
- manifest;
- checksum;
- ROM/dictionnaire;
- redondance;
- metadata;
- cout runtime proxy.

Ce modele est utile pour tester la surface P61, mais il ne constitue pas une
validation scientifique. Les champs `read_p50_us`, `read_p95_us`, `read_p99_us`,
`update_p50_us`, `update_p95_us`, et `update_p99_us` restent `null` tant qu'une
mesure temporelle reelle n'est pas implementee.

Un workload refuse peut encore payer un cout reel: parsing, inspection,
manifest, checksum, journal de refus ou audit minimal. Ce cout est conserve dans
le denominateur, mais l'espace virtuel effectif reste nul.

## Commande

```bash
cargo run -p atlas-cli -- ratio examples/p53_strict.atlas --mode smoke --format json
cargo run -p atlas-cli -- ratio examples/p53_strict.atlas --mode standard --format json
```

Le format JSON est requis pour cette premiere version.

## Workloads

Chaque entree `workloads` expose un `id`, un `kind`, un `mechanism`, les couts
reels detailles, les compteurs CRUD/systeme et une `refusal_reason`.

- `W0_guard_random_space`, mecanisme `guard_refusal`: espace virtuel declare
  eleve, refuse, effet nul.
- `W1_lattice_symmetry_space`, mecanisme `lattice_symmetry`: espace regulier
  genere, lisible et partiellement updatable.
- `W2_sparse_event_space`, mecanisme `sparse_indexed_events`: espace
  evenementiel sparse, beneficie d'index et journal mais les paie dans le cout
  reel.
- `W3_hybrid_field_space`, mecanisme `hybrid_global_local_field`: proxy simplifie
  pour regle globale et singularites locales, sans pretention de validation
  mathematique.
- `W4_topological_atlas_space`, mecanisme `topological_atlas_gluing`: charts
  locaux et gluing proxy.
- `W5_adversarial_virtual_space`, mecanisme `adversarial_refusal`: espace
  apparemment compressible mais dangereux, refuse ou rendu ineffectif.

## Semantique CRUD Proxy

Pour un workload accepte:

- `create_count > 0`;
- `read_count > 0`;
- `update_count` indique les mutations proxy;
- `delete_count` indique les tombstones ou invalidations quand le workload en
  utilise;
- `snapshot_count >= 1`;
- `rebuild_count >= 1`;
- `audit_count >= 1`;
- `refusal_reason = "none"`.

Pour un workload refuse:

- `accepted = false`;
- `refused = true`;
- `virtual_effective = 0`;
- `refusal_reason` est explicite, par exemple `guard_random_space` ou
  `adversarial_or_dangerous_space`;
- le cout reel peut rester positif, car evaluer/refuser un espace consomme des
  ressources.

## Decisions

P61 expose:

- `VALIDATE_P61_VIRTUAL_RATIO_CORE`;
- `RECALIBRATE_P61_RATIO_COST_MODEL`;
- `RECALIBRATE_P61_ADDRESSABILITY`;
- `NO_GO_P61_VIRTUAL_RATIO`.

La premiere implementation reste conservatrice et devrait normalement produire
`RECALIBRATE_P61_RATIO_COST_MODEL` tant que le modele de cout n'est pas calibre.

## Validation Locale

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
bash scripts/validate_p58_local.sh
cargo run -p atlas-cli -- ratio examples/p53_strict.atlas --mode smoke --format json
cargo run -p atlas-cli -- ratio examples/p53_strict.atlas --mode standard --format json
```

## Limites Et Suite

- Aucun timing reel n'est revendique.
- Les couts sont des proxies deterministes.
- `standard` reste assez leger, mais CI peut se limiter a `smoke`.
- Le golden `tests/golden/p61_ratio_smoke.json` fige seulement la surface smoke
  deterministe; il ne prouve pas une calibration scientifique.
- Les prochaines etapes sont la calibration du cout, des seuils calibres, des
  timings reels, le renforcement des semantiques update/delete, et la
  non-regression explicite contre P51/P53.
