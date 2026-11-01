# Compagnon macOS SwiftUI

Interface native de WarpgateSH, minimum macOS 13, Swift 6.2 ou ultérieur et Xcode.
Le backend Rust autonome, la CLI et l’agent conservent les données et les accès existants.
Sparkle 2.10.1 gère les mises à jour natives, avec transition depuis la 0.1.24.
Voir [ADR 0038](../../docs/adr/0038-remove-tauri-from-the-macos-companion.md).

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
doit déjà être actif pour la synchronisation en arrière-plan. Le mode manuel
reste utilisable sans agent. Le bundle de production demande l’accord au premier
lancement d’une nouvelle installation, sans solliciter les agents déjà installés.
Pour prévisualiser cet accueil sans rien installer, lancer le mode `--demo` avec
l’argument supplémentaire `--demo-first-launch`.
Ne pas tester une suppression ou une mise à jour sur l’installation personnelle.

## Validation

```sh
python3 apps/warpgatesh-macos/scripts/swift.py test --package-path apps/warpgatesh-macos
python3 apps/warpgatesh-macos/scripts/swift.py format lint --strict --recursive apps/warpgatesh-macos/Sources apps/warpgatesh-macos/Tests
nix run .#check
nix run .#build-macos
```

Les contrôles macOS compilent aussi l’interface SwiftUI. Le packaging natif assemble directement l’interface, le backend autonome,
la CLI, l’agent et les ressources Swift. Le vérificateur
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

## Preferences in the development version

These settings are included from v0.1.20:

- **General**: automatic/French/English language; original Warpgate, Light, Black,
  or System appearance. System follows the macOS light/dark setting. Language and
  appearance update immediately, including the menu bar and dialogs.
- **Synchronization**: interval and default profile, saved through the existing
  backend. **System**: launch at login, CLI installation and documentation,
  application removal. These settings retain their explicit Save button.
- Local presentation settings persist in UserDefaults independently of backend
  configuration. Demo settings use `dev.warpgatesh.preview.settings` and never
  overwrite the installed application's presentation settings.

## Notifications

Notifications are opt-in. Enabling them requests the native macOS authorization;
denied permission is displayed with a link to System Settings. The test button
sends a real local notification after authorization. No push service is used.

The running companion evaluates the agent's existing persistent-error alerts
and client update state. Incidents are delivered once until resolution; updates
are delivered once per version. Clicking the action opens Profiles, Diagnostics,
About, or the official Warpgate release notes. The companion keeps monitoring
when its window is closed, while its menu-bar process is running. Quitting it
stops notifications; the independent Rust agent continues synchronizing.

From v0.1.22, app-managed tokens use their server-reported expiry and are renewed
by the Rust agent. Imported tokens without an expiry retain **Profiles → Token
expiry reminder**, an optional user-supplied date. Reminders occur within seven
days, within one day and after expiry. No date is guessed. Rejected authentication
and failed automatic renewal open the profile. Successful replacement clears an
old manual reminder. See [sign-in and token renewal](../../docs/getting-started.md).

Server release checks are optional per profile, every six hours, against the
public GitHub releases API. They compare stable numeric versions with the server
version observed during synchronization. No instance URL, username or token is
sent to GitHub. An optional release-check failure never blocks SSH or syncing.


La prévisualisation est signée ad hoc pour permettre à macOS d’autoriser les
notifications locales. Les nouveaux tests couvrent les traductions, les réglages
persistants, les seuils de rappel, les incidents et les comparaisons de versions.
