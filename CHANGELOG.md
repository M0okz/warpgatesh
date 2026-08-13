# Changelog

Toutes les évolutions notables de WarpgateSH sont documentées ici.

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
