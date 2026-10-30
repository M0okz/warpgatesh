# Migration SwiftUI — validation locale initiale du 30 octobre 2026

Ce compte rendu décrit les contrôles locaux avant préparation de la release
v0.1.18. La livraison signée est suivie dans la [release GitHub](https://github.com/M0okz/warpgatesh/releases/tag/v0.1.18).

Le compagnon macOS est implémenté dans `apps/warpgatesh-macos`. L’interface
Accès validée a été conservée : logo, matériau natif translucide, cartes,
recherche et animations de survol. Les six rubriques sont raccordées au moteur
Rust existant. Le mode démo reste isolé des profils et du réseau.

## Contrôles réalisés

- `nix develop --command bash nix/tasks.sh check` : contrôles Rust du workspace,
  frontend conservé, scripts de distribution, Swift et compagnon Rust réussis.
- 4 tests Swift : recherche multi-profils et alias qualifiés, âge avant première
  synchronisation, refus des actions réelles en démo, lecture obsolète après mutation.
- 35 tests du compagnon Rust, avec refus d’empreintes SSH modifiées après
  approbation ; formatage et Clippy strict réussis.
- Test du backend réel via pipes : schéma de réponse, erreurs corrélées,
  approbation SSH obligatoire avant ajout, absence du jeton fictif dans l’erreur,
  et arrêt propre à la fermeture des pipes. L’agent n’est pas initialisé par ce test.
- Navigation native, formulaire d’ajout, préférence de démo, aperçu des journaux,
  page À propos et écran Accès inspectés dans une fenêtre macOS.
- Prévisualisation connectée : lecture du profil existant et des 54 cibles réelles,
  sans ajout, suppression ni renouvellement de jeton pour les besoins du test.
- `nix develop --command bash nix/tasks.sh build-macos` : application et DMG
  universels construits, vérificateur du bundle et `hdiutil verify` réussis.
- Exécutable SwiftUI : architectures `arm64` et `x86_64`, minimum macOS `13.0`
  dans les deux commandes `LC_BUILD_VERSION`, absence de dépendance au Nix store.
- `npm run test:relaunch` dans l’environnement Nix : remplacement de deux bundles
  jetables signés ad hoc, nouveau binaire renommé, chemins avec espaces et
  apostrophes ; fenêtre rouverte après arrêt de l’ancien processus.

## Portée et limites

La prévisualisation connectée utilise un identifiant distinct et l’application
installée n’a pas été remplacée. Aucune publication, aucun tag ni push n’a été
réalisé. Le DMG local provient du build `--no-sign` : les étapes Developer ID,
notarisation Apple et vérification d’une véritable mise à jour publiée restent
à exécuter dans le workflow de release. La signature ad hoc du test de relance
ne remplace pas ces validations de distribution.

Les parcours d’ajout, renouvellement et suppression de profils, installation CLI
et désinstallation n’ont pas été exécutés sur les données personnelles pour
cette validation. Leurs opérations Rust existantes sont réutilisées et testées.
Le scénario de relance avec un parent SwiftUI distinct est implémenté ; le test
jetable vérifie le mécanisme de remplacement partagé, sans simuler un téléchargement
signé de bout en bout. Une vérification manuelle complète VoiceOver et un essai
sur matériel Intel/macOS 13 restent nécessaires avant publication.
