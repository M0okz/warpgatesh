# Publier WarpgateSH sur macOS

La publication macOS produit un unique DMG universel contenant le compagnon,
la CLI et l’agent pour `arm64` et `x86_64`. Le workflow ne se déclenche que sur
un tag `vX.Y.Z` correspondant exactement aux versions du workspace Rust, du
compagnon npm et de Tauri.

## Prérequis Apple

La distribution hors Mac App Store exige un abonnement Apple Developer et un
certificat **Developer ID Application** exporté au format PKCS#12 (`.p12`). La
clé privée ne doit jamais être ajoutée au dépôt. La notarisation utilise un mot
de passe spécifique à l’application Apple ID.

Configurer les secrets GitHub Actions suivants :

- `APPLE_CERTIFICATE` : contenu base64 du fichier `.p12` ;
- `APPLE_CERTIFICATE_PASSWORD` : mot de passe choisi lors de l’export ;
- `KEYCHAIN_PASSWORD` : mot de passe aléatoire du Trousseau éphémère de CI ;
- `APPLE_ID` : adresse du compte Apple utilisé pour notariser ;
- `APPLE_PASSWORD` : mot de passe spécifique à l’application ;
- `APPLE_TEAM_ID` : identifiant de l’équipe Apple Developer.
- `TAURI_SIGNING_PRIVATE_KEY` : clé privée de l’updater Tauri, également
  conservée dans Bitwarden et jamais ajoutée au dépôt ;
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` : mot de passe de cette clé.

La clé publique Tauri est intégrée à `tauri.conf.json`. Cette signature est
indépendante de la signature Developer ID : Apple valide l’application macOS,
tandis que l’updater vérifie que l’archive a bien été produite par WarpgateSH.

Le workflow importe le certificat dans un Trousseau temporaire du runner,
active le Hardened Runtime, signe tous les exécutables, soumet le DMG au service
notarial Apple, agrafe le ticket et exécute `codesign`, `spctl`, `stapler` et
`hdiutil` avant toute publication.

## Créer une version

1. Aligner les versions dans `Cargo.toml`, `package.json` et
   `src-tauri/tauri.conf.json`.
2. Exécuter les tests et créer le tag signé `vX.Y.Z`.
3. Pousser le tag. Le workflow crée une GitHub Release en brouillon avec le DMG,
   sa somme SHA-256, l’archive de mise à jour `WarpgateSH.app.tar.gz`, sa
   signature et `latest.json`.
4. Installer le DMG sur un Mac de test, puis publier manuellement le brouillon.

Le workflow agrafe explicitement le ticket de l’application après l’assemblage Tauri, puis reconstruit le DMG et l’archive de mise à jour à partir de ce bundle. L’archive est signée de nouveau et son contenu extrait doit encore passer `codesign` et `stapler validate`.

Pour reconstruire une version dont le workflow a échoué sans déplacer son tag signé, utiliser le déclenchement manuel du workflow **Release macOS** depuis `main`, avec le tag existant dans le champ `tag`.

Le brouillon n’est jamais proposé aux clients. Après publication, l’application
lit `releases/latest/download/latest.json` au maximum une fois toutes les six heures. Elle ne
télécharge et n’installe une archive qu’après une action explicite de
l’utilisateur et une vérification réussie de sa signature.

La première version intégrant l’updater doit encore être installée depuis son
DMG. Les versions suivantes peuvent remplacer ensemble le compagnon, la CLI et
l’agent, puis redémarrent l’agent `launchd` depuis le nouveau bundle.

## Environnement de build Nix

Le développement, les contrôles CI et la publication utilisent les entrées
verrouillées de `flake.lock`. Nix fournit Rust 1.85.0, les deux cibles Apple,
Node.js 22 et npm. Xcode et ses outils de signature restent installés sur le
Mac ou le runner. Voir [le guide Nix](development-nix.md).

Pour vérifier l’assemblage universel complet sans identifiants de publication,
depuis la racine du dépôt :

```sh
nix run .#build-macos
```

Cette commande produit l’application et le DMG, contrôle les trois exécutables
universels et vérifie l’image disque. Elle n’active pas les artefacts d’updater,
la signature Developer ID ou la notarisation.

Pour construire une publication avec les identifiants Apple et Tauri injectés
dans le processus :

```sh
nix run .#release-macos
```

Cette commande active `tauri.release.conf.json` et exige notamment
`TAURI_SIGNING_PRIVATE_KEY` et `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
Ne jamais ajouter ces secrets aux fichiers Nix. Le workflow conserve les étapes
Apple de finalisation, notarisation et validation avant la création du brouillon.

## Interface native SwiftUI

Le packaging prépare aussi `warpgatesh-ui` et le bundle de ressources Swift.
Xcode doit fournir Swift 6.2 ou ultérieur. Le lanceur/backend Rust, l’interface
SwiftUI, la CLI et l’agent sont tous universels et signés ensemble. Le protocole
de mise à jour signé, l’identifiant du bundle et les chemins Trousseau restent
identiques ; aucune conversion des données utilisateur n’est nécessaire.


## Smoke test du lancement macOS

Lancer le bundle final depuis Finder ou avec `open -n /Applications/WarpgateSH.app`,
puis vérifier que la fenêtre reste ouverte et que l’agent répond. Répéter avec
`--args --demo` et vérifier la fermeture/réouverture. Exécuter seulement le
binaire en ligne de commande ne suffit pas à tester l’enregistrement Launch
Services. Celui-ci peut présenter une entrée temporaire dont le PID vaut -1 ;
elle ne doit jamais être considérée comme une autre instance active.
