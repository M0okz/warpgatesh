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

Le workflow importe le certificat dans un Trousseau temporaire du runner,
active le Hardened Runtime, signe tous les exécutables, soumet le DMG au service
notarial Apple, agrafe le ticket et exécute `codesign`, `spctl`, `stapler` et
`hdiutil` avant toute publication.

## Créer une version

1. Aligner les versions dans `Cargo.toml`, `package.json` et
   `src-tauri/tauri.conf.json`.
2. Exécuter les tests et créer le tag signé `vX.Y.Z`.
3. Pousser le tag. Le workflow crée une GitHub Release en brouillon avec le DMG
   et sa somme SHA-256.
4. Installer le DMG sur un Mac de test, puis publier manuellement le brouillon.

Pour tester uniquement l’assemblage universel en local, installer les cibles
Rust `aarch64-apple-darwin` et `x86_64-apple-darwin`, puis exécuter :

```sh
cd apps/warpgatesh-companion
npm run bundle:macos:universal
```
