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
sur matériel Intel/macOS 13 restent à réaliser ; cette validation locale ne les couvre pas.


## Livraison v0.1.18 vérifiée

La [PR #8](https://github.com/M0okz/warpgatesh/pull/8) a été fusionnée et le tag
v0.1.18 signé sur `3ade37523eed24b320c9f088bd9897371b33fd41`. Les cinq contrôles
[CI](https://github.com/M0okz/warpgatesh/actions/runs/41035902936) ont réussi,
ainsi que le [workflow de distribution](https://github.com/M0okz/warpgatesh/actions/runs/41038220887).
Le compagnon compte désormais 36 tests Rust, dont la stabilité de l'approbation
quand les empreintes SSH reviennent dans un ordre différent.

Les fichiers téléchargés depuis la release ont été contrôlés : SHA-256 du DMG,
signature Minisign de l'archive de mise à jour, `codesign --deep --strict`,
acceptation Gatekeeper comme Developer ID notarié, tickets Apple du bundle et du
DMG, et intégrité `hdiutil`. Les quatre exécutables contiennent `arm64` et
`x86_64`, sans dépendance au Nix store. Le CLI annonce 0.1.18 ; l'interface
signée téléchargée a été ouverte en mode démo et sa page À propos annonce 0.1.18.
L'application installée et les données personnelles n'ont pas été remplacées.

La release publique et le manifeste de mise à jour annoncent 0.1.18. Le tap
Homebrew distribue le code source du même tag avec son SHA-256 vérifié. Le site
public sert les six captures communes et son guide natif ; les contrôles sur
ordinateur et mobile confirment leur chargement sans débordement horizontal ni
erreur JavaScript. Les captures sont stockées en PNG 860 × 620 ; leur manifeste
consigne leurs dimensions et empreintes réelles.


## Development regression check: closing the companion

After 0.1.20, closing the main window explicitly changes the companion to the
macOS accessory activation policy. The process and SwiftUI menu-bar extra stay
alive; reopening restores the regular application policy. Explicit Quit still
terminates the companion. The cause was a regular policy set only at launch,
with no matching window-close transition.

This AppKit lifecycle needs an actual running application rather than a view
unit test. The read-only regression check examines Launch Services metadata:

```sh
# Close the preview window with the red button, then check without reopening it.
python3 apps/warpgatesh-macos/scripts/check-window-mode.py --expect menu-bar
# Reopen the companion, then check the reverse transition.
python3 apps/warpgatesh-macos/scripts/check-window-mode.py --expect window
```

Verified locally: the original close returned Foreground (failure); after the
fix it returned UIElement (pass), and reopening returned Foreground (pass).
