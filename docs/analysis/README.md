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
resumes avant le commit de consolidation.

## Difference avec les autres traces

- Rapport d'analyse repo: Markdown durable, committable, centre sur
  interpretation, decisions, limites et suite.
- Trace locale: resume operationnel court d'une execution locale ou Codespaces,
  utile pour verifier ce qui a ete lance sur une machine donnee.
- Results LaTeX/PDF: artefact futur de presentation scientifique, plus formel,
  derive des rapports et des resultats stabilises.
- Colonne vertebrale: synthese transversale du projet ASTRA, reliant les
  sprints, les invariants et les claims autorises.

## Regles de contenu

- Ne pas coller de logs volumineux.
- Ne pas inventer de resultats.
- Marquer les resultats manquants avec des statuts explicites comme
  `TODO_AFTER_LOCAL_VALIDATION`, `TODO_AFTER_CI`, `NOT_MEASURED_YET` ou
  `PENDING_P63_IMPLEMENTATION`.
- Inclure les commandes pertinentes, les resultats resumes, les decisions, les
  limites et la recommandation suivante.
- Garder les fichiers compacts et versionnables.

