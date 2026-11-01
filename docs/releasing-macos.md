# Publier WarpgateSH sur macOS

Depuis 0.1.25, la distribution utilise Swift, Cargo et les outils Apple, sans
Tauri. Le bundle universel contient `warpgatesh-ui`, le backend autonome
`warpgatesh-companion`, la CLI et l’agent. L’interface est l’exécutable principal
et porte l’identifiant `dev.warpgatesh.companion`.

## Construction et contrôles

Nix reste la référence. Xcode fournit Swift 6.2+, les SDK et les outils Apple.

```sh
nix run .#check
nix flake check
nix run .#build-macos
nix run .#build-site
```

Les sorties sont dans `apps/warpgatesh-macos/.build/distribution/`.
`package.py` compile les deux architectures, assemble le bundle et le DMG,
puis exécute `verify-bundle.py` et `hdiutil verify`. Le build local utilise une
signature ad hoc ; il ne constitue pas une distribution notarisée.

## Identifiants de publication

Le workflow conserve les secrets Apple existants : `APPLE_CERTIFICATE`,
`APPLE_CERTIFICATE_PASSWORD`, `KEYCHAIN_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD`
et `APPLE_TEAM_ID`. Le certificat Developer ID est importé dans le trousseau
éphémère du runner. `APPLE_SIGNING_IDENTITY` désigne cette identité.

`TAURI_SIGNING_PRIVATE_KEY` et `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` sont les noms
historiques des secrets de signature d’update. Ils conservent leur contenu et
ne nécessitent pas Tauri : `warpgatesh-update-sign` charge la clé Minisign
existante et écrit la signature encodée attendue par les anciens clients.
La clé publique reste dans `apps/warpgatesh-macos/Bundle/updater.pub`.
Ne jamais copier les secrets dans Git, Nix ou les logs.

## Préparer une release

1. Aligner la version du workspace, les notes et la documentation. Vérifier
   qu’aucun tag du même nom n’existe.
2. Exécuter les contrôles, les scénarios d’update/reprise et la validation du
   bundle empaqueté sur les environnements supportés.
3. Créer et vérifier le tag signé `vX.Y.Z`, puis le pousser. Le workflow
   **Release macOS** construit une distribution en brouillon.
4. Télécharger les six artefacts, vérifier leur intégrité et leurs signatures,
   tester l’installation et la relance, puis publier le brouillon.
5. Vérifier le flux public depuis la 0.1.24 officielle et le nouveau moteur ;
   aligner Homebrew, le site, les captures et les preuves de validation.

Une reconstruction peut utiliser `workflow_dispatch` avec le tag existant.
Un tag signé n’est pas déplacé pour réparer un workflow.

## Pipeline de distribution

Avec les accès de release injectés et `RELEASE_NOTES_FILE` pointant sur le texte
des notes, `nix run .#release-macos` exécute `scripts/release.py` :

1. Construire et signer les composants imbriqués puis l’application, avec
   Hardened Runtime et horodatage.
2. Soumettre une copie ZIP à Apple, agrafer et valider le ticket de l’app finale.
3. Fabriquer le DMG et `WarpgateSH.app.tar.gz` depuis cette même app ; signer
   l’archive avec l’outil autonome et générer `latest.json`.
4. Signer, notariser, agrafer et vérifier le DMG. Vérifier aussi l’app extraite
   de l’archive d’update, puis produire le fichier SHA-256.

Les sorties publiques restent le DMG universel, son `.sha256`,
`WarpgateSH.app.tar.gz`, sa `.sig`, `latest.json` et `appcast.xml`. Les deux clés de plateforme
`darwin-aarch64` et `darwin-x86_64` pointent vers l’archive universelle. Les
clients installés conservent le même point d’accès HTTPS et la même clé publique.
Le manifeste et la release doivent avoir les mêmes notes et version.

## Vérifications fonctionnelles

Lancer depuis Applications/Launch Services, puis tester fermeture par la croix
rouge, icône visible, réouverture et sortie complète. Répéter avec une identité
neuve, puis avec un profil existant et l’agent actif. Une présence de processus
ou une entrée dans les permissions de barre des menus ne prouve pas à elle seule
la visibilité de l’icône.

Le mode `--demo` et `--demo-first-launch` sert aux captures et aux essais visuels
isolés. Il ne lance pas le backend réel. Les captures publiées restent fictives ;
les comptes-rendus doivent distinguer build universel, tests Intel/macOS minimum
et tests sur le matériel réellement utilisé.

## Native Sparkle transition (0.1.25)

The native companion uses pinned Sparkle 2.10.1. `SUPublicEDKey` comes from
`apps/warpgatesh-macos/Bundle/sparkle.pub`. `SPARKLE_ED_PRIVATE_KEY` is injected
as a release secret, separately from the legacy Minisign secrets. The pinned
official Sparkle signing tools are checksum-verified before use; private keys
are supplied over standard input, never command arguments.

Publish `appcast.xml` alongside the existing five release assets. Both its
embedded signature and its enclosure signature are required. `latest.json`
and `WarpgateSH.app.tar.gz.sig` remain for 0.1.24; the identical notarized archive
is also signed for Sparkle. All bundle copies and tar archives preserve framework
symlinks. Nested Sparkle services are signed inside-out before the parent app.
Sparkle owns authorized installation and relaunch; no custom Rust installer runs.
