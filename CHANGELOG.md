# Changelog

Toutes les évolutions notables de WarpgateSH sont documentées ici.

## 0.1.8 — 2026-08-13

### Amélioration

- Le menu de la barre des menus affiche désormais un voyant vert lorsque l’agent répond et un voyant rouge lorsqu’il est arrêté ou indisponible.
- La ligne d’état de l’agent ouvre WarpgateSH, ce qui la rend utile tout en conservant les informations de synchronisation en lecture seule.

### Mise à jour intégrée

- Cette version est la première destinée à valider le parcours complet de mise à jour signée depuis WarpgateSH 0.1.7, avec consentement explicite, relance de l’application et reprise de l’agent.

## 0.1.7 — 2026-08-13

### Nouveautés

- WarpgateSH vérifie désormais chaque jour si une nouvelle version est disponible, avec une vérification manuelle accessible depuis les préférences et la barre des menus.
- Les installations directes peuvent télécharger et appliquer une mise à jour après confirmation explicite. La signature est vérifiée avant toute installation.
- L’interface affiche la version installée, les notes de version, la progression du téléchargement et l’état du redémarrage.

### Distribution

- Les releases macOS fournissent maintenant un manifeste `latest.json` et une archive universelle signée pour les Mac Apple silicon et Intel.
- L’application et l’agent d’arrière-plan redémarrent automatiquement après une mise à jour réussie, sans interrompre les sessions SSH déjà ouvertes.
- Les installations gérées par Homebrew restent orientées vers `brew upgrade --cask warpgatesh` afin de conserver une seule source de mise à jour.

### Première installation

La version 0.1.7 est la première à intégrer ce mécanisme. Elle doit donc être installée manuellement depuis le DMG GitHub ; les versions suivantes pourront être proposées directement par l’application.
