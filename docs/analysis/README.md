# ASTRA Analysis Reports

Ce dossier contient les rapports d'analyse committables du projet ASTRA. Un
rapport d'analyse est une synthese durable, lisible dans le repo, qui relie un
sprint ou une etape importante aux commandes executees, aux resultats resumes,
aux decisions, aux limites et a la recommandation suivante.

## Role

Les rapports d'analyse servent a garder une memoire structurée des validations
repo-first sans coller de gros journaux d'execution. Ils doivent rester utiles
pour une relecture future dans GitHub, dans un editeur local, ou dans une
conversation ChatGPT ulterieure.

Chaque step important doit conserver un rapport Markdown committable apres les
tests locaux pertinents. Le rapport doit etre complete avec les resultats
resumes avant le commit de consolidation. Quand une etape est suffisamment
figee, Codex peut generer un livrable Results en LaTeX/PDF directement depuis le
repo.

## Difference avec les autres traces

- Rapport d'analyse repo: Markdown durable, committable, centre sur
  interpretation, decisions, limites et suite.
- Trace locale: resume operationnel court d'une execution locale ou Codespaces,
  utile pour verifier ce qui a ete lance sur une machine donnee.
- Results LaTeX/PDF: livrable final fige, plus formel, derive des rapports et
  des resultats stabilises. Les sources vivent sous `reports/` et sont compilees
  localement, Tectonic en priorite.
- Colonne vertebrale: synthese transversale du projet ASTRA, reliant les
  sprints, les invariants et les claims autorises.

## Regles de contenu

- Ne pas coller de logs volumineux.
- Ne pas inventer de resultats.
- Marquer les resultats manquants avec des statuts explicites comme
  `TODO_AFTER_LOCAL_VALIDATION`, `NOT_MEASURED_YET` ou
  `PENDING_P63_IMPLEMENTATION`.
- Inclure les commandes pertinentes, les resultats resumes, les decisions, les
  limites et la recommandation suivante.
- Garder les fichiers compacts et versionnables.

## Test stack hygiene

A partir de P70, chaque jalon repo-first doit inclure un audit de la stack de
tests Rust locale. Les tests obsoletes, redondants ou trompeurs doivent etre
supprimes, fusionnes ou recalibres avec justification. Les tests historiques qui
protegent encore un invariant reel restent des non-regressions versionnees.

## Results LaTeX/PDF

Le rapport Markdown d'analyse reste la trace vivante. Le `.tex` et le `.pdf`
Results sont le livrable final fige d'une etape. Ils doivent etre generes
localement par Codex apres validation locale, sans dependre de ChatGPT pour la
compilation PDF.

Le compilateur prioritaire est Tectonic:

```bash
bash scripts/build_report.sh reports/P63/RPA_ASTRA-P63-Results_measured-ratio_v1.0_2026-05-01.tex
```

Le script essaie ensuite `latexmk`, puis `pdflatex` si Tectonic n'est pas
disponible. La CI GitHub reste minimale: elle ne compile pas les rapports
lourds, ne lance pas les campagnes longues et ne porte pas de tests dependants
du timing.
