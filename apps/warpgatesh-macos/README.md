# Compagnon macOS SwiftUI

Interface native de WarpgateSH, minimum macOS 13, Swift 6.2 ou ultérieur et Xcode.
Le moteur Rust, la CLI, l’agent et le mécanisme de mise à jour signé sont réutilisés.
Voir [ADR 0036](../../docs/adr/0036-use-swiftui-for-the-macos-companion.md).

## Prévisualisation

Depuis la racine, dans l’environnement Nix :

```sh
nix develop --command sh apps/warpgatesh-macos/scripts/run.sh
```

Le script assemble `.build/WarpgateSH Preview.app`, avec un identifiant de
prévisualisation distinct de l’application installée. Il ouvre le mode `--demo` :
aucun profil réel, aucun jeton, aucune connexion, aucune modification SSH.
Les actions sensibles sont refusées dans ce mode. Les recherches, préférences et
synchronisations de démonstration fonctionnent localement.

Pour utiliser le moteur réel en développement, ajouter `--live`. Cette option
lit les profils et l’instantané existants et permet les actions demandées dans
l’interface. Le backend debug n’installe pas automatiquement l’agent ; celui-ci
doit déjà être actif. Le bundle de production enregistre l’agent comme auparavant.
Ne pas tester une suppression ou une mise à jour sur l’installation personnelle.

## Validation

```sh
swift test --package-path apps/warpgatesh-macos
swift format lint --strict --recursive apps/warpgatesh-macos/Sources apps/warpgatesh-macos/Tests
nix run .#check
nix run .#build-macos
```

Les contrôles macOS compilent aussi l’interface SwiftUI. Le packaging prépare le
sidecar natif et ses ressources avant le bundle Tauri existant. Le vérificateur
contrôle les quatre exécutables, leurs deux architectures et l’absence de liens
vers le Nix store. Le workflow de publication existant signe et notarise le
bundle complet ; aucune publication n’est déclenchée par ces commandes locales.

## Fonctionnalités

- Accès : agent, dernière synchronisation, compteurs, recherche `⌘K`, connexions
  via le terminal et les alias qualifiés ; états première utilisation et erreur.
- Profils : inspection, approbation des empreintes, ajout, renouvellement du jeton,
  choix du profil par défaut et suppression confirmée.
- Préférences : intervalle, ouverture de session, installation de la CLI et
  désinstallation avec confirmation saisie et suppression des données facultative.
- Synchronisation : état de l’agent, instantané et alertes, action immédiate.
- Journaux : aperçu des fichiers et volumes avant export local nettoyé.
- À propos : identité communautaire non officielle, version, notes et mise à jour
  signée avec confirmation ; instructions adaptées à Homebrew.
- Barre de menus : état, compteurs, accès aux profils, synchronisation et fenêtre.

Les contrôles natifs et libellés assistent VoiceOver. Les animations de survol
respectent Réduire les animations ; le matériau latéral devient opaque avec
Réduire la transparence. Une vérification manuelle VoiceOver et des réglages
macOS reste requise avant une publication publique.
