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

Le brouillon n’est jamais proposé aux clients. Après publication, l’application
lit `releases/latest/download/latest.json` au maximum une fois par jour. Elle ne
télécharge et n’installe une archive qu’après une action explicite de
l’utilisateur et une vérification réussie de sa signature.

La première version intégrant l’updater doit encore être installée depuis son
DMG. Les versions suivantes peuvent remplacer ensemble le compagnon, la CLI et
l’agent, puis redémarrent l’agent `launchd` depuis le nouveau bundle.

Pour tester uniquement l’assemblage universel en local, installer les cibles
Rust `aarch64-apple-darwin` et `x86_64-apple-darwin`, puis exécuter :

```sh
cd apps/warpgatesh-companion
npm run bundle:macos:universal
```

Cette commande exige les variables `TAURI_SIGNING_PRIVATE_KEY` et
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Les builds ordinaires et la CI n’activent
pas `tauri.release.conf.json` et ne produisent donc pas d’artefacts d’updater.
