# WarpgateSH

Client CLI pour rester synchronisé avec Warpgate.

## But

`warpgatesh` doit fournir une expérience proche de `tsh`, mais adaptée à
l’infrastructure Homeblack : rester dans le terminal, voir les cibles SSH
accessibles et se connecter à travers Warpgate sans maintenir manuellement sa
configuration locale.

## Architecture cible

```text
Terraform
  └─ crée les VM, les IP et les tags Proxmox
       ↓
Ansible
  └─ configure Warpgate et réconcilie ses cibles via l’API
       ↓
Warpgate
  └─ source de vérité des accès SSH
       ↓
WarpgateSH
  └─ client local macOS puis Linux
```

Terraform reste responsable du cycle de vie des VM. Ansible reste responsable
de la configuration serveur et de la création, mise à jour ou suppression des
cibles Warpgate. `warpgatesh` ne modifie pas l’infrastructure : il consomme l’API
Warpgate et maintient l’accès local à jour.

## Expérience souhaitée

```sh
warpgatesh login homeblack
warpgatesh ls
warpgatesh sync
warpgatesh dmz-nextcloud-01
warpgatesh status
```

`warpgatesh <alias>` résout une Cible SSH connue puis remplace son propre
processus par l’OpenSSH du système. Les arguments placés après `--` sont transmis
à `ssh`. Les noms des sous-commandes de gestion sont réservés ; un Alias qualifié
permet d’éviter toute ambiguïté avec une cible portant le même nom.

La connexion utilise immédiatement l’Instantané local et ne déclenche jamais de
synchronisation préalable. Si cet instantané est périmé, `warpgatesh` affiche un
avertissement puis lance OpenSSH sans attendre ; `warpgatesh sync` reste le moyen
explicite de demander une actualisation immédiate.

La CLI reste l’interface principale. Une petite interface optionnelle dans la
barre des menus sert aux préférences, à la recherche de cibles et au suivi de
la synchronisation automatique.

## MVP

1. Authentification auprès de Warpgate via l’API.
2. Liste des cibles accessibles à l’utilisateur.
3. Synchronisation des alias SSH locaux.
4. Connexion SSH depuis le terminal.
5. Stockage des préférences et des secrets dans les mécanismes natifs du poste.
6. Distribution macOS complète et usage quotidien validé.

Linux vient dans une phase ultérieure, après validation quotidienne du produit
sur macOS. Le cœur Rust et les tests automatisés restent néanmoins portables
dès le MVP afin d’éviter une réécriture lors de cette seconde phase.

Le MVP prend en charge macOS 13 et les versions ultérieures. Les distributions
macOS sont universelles et exécutent nativement le compagnon graphique, la CLI
et l’agent sur Apple Silicon comme sur Intel.

## Principes

- CLI-first : aucune interface graphique obligatoire.
- Une seule source de vérité côté serveur : Warpgate.
- Terraform et Ansible restent les outils d’infrastructure.
- Ne jamais stocker de secret Warpgate dans le state Terraform ou dans Git.
- Les entrées SSH gérées par `warpgatesh` sont isolées des entrées manuelles.
- Rust est le langage envisagé pour le cœur, la CLI et la portabilité macOS/Linux.

## Synchronisation locale

Un agent propre à la session de chaque utilisateur synchronise tous les profils
Warpgate configurés. Il lance une tentative au démarrage, au réveil du poste,
au retour du réseau, puis toutes les cinq minutes par défaut avec un léger
décalage aléatoire. L’intervalle est personnalisable et les échecs utilisent
une attente progressive. `warpgatesh sync` demande une tentative immédiate.

La création du premier profil enregistre l’agent pour qu’il démarre à chaque
ouverture de session. Le compagnon graphique ne démarre automatiquement que si
l’utilisateur active cette préférence ; le quitter n’arrête jamais l’agent.
L’arrêt de la synchronisation de fond reste une action explicite accompagnée
d’un avertissement.

L’agent est l’unique écrivain de la configuration locale. La CLI et le
compagnon lui transmettent des mutations typées par le socket Unix privé ; ils
ne modifient directement ni le catalogue, ni les préférences, ni les clés
épinglées, ni le Trousseau. Une synchronisation réseau déjà en cours peut finir
sur son instantané initial, puis l’agent enchaîne automatiquement une nouvelle
synchronisation avec la configuration modifiée.

Le fichier SSH est remplacé atomiquement seulement après une réponse Warpgate
valide et complète. Une erreur conserve le dernier instantané réussi et le
signale comme périmé ; elle ne produit jamais un fichier vide ou partiel.

Après une synchronisation réussie, une cible devenue inaccessible est retirée
immédiatement du nouvel instantané et ne peut plus être ouverte par son alias.
WarpgateSH ne ferme pas les sessions OpenSSH déjà établies, qu’il ne possède pas,
mais affiche le résumé des cibles ajoutées et retirées.

Les noms de profils utilisent des lettres minuscules, des chiffres et des
tirets. Un nom de cible déjà compatible avec OpenSSH est conservé ; sinon il
est normalisé en minuscules avec des tirets. Si deux noms normalisés entrent en
collision, un court suffixe stable dérivé de l’identifiant Warpgate les
distingue. `warpgatesh ls` affiche toujours les alias réellement générés.

OpenSSH charge cette configuration par une unique directive
`Include ~/.ssh/warpgatesh/config` placée près du début de `~/.ssh/config`. Les
alias et clés d’hôte gérés restent sous `~/.ssh/warpgatesh/`, tandis que les
profils et l’état non secret résident dans
`~/Library/Application Support/WarpgateSH/`. Une désinstallation peut ainsi
retirer uniquement ces éléments sans modifier la configuration SSH personnelle.

L’ajout du premier profil installe automatiquement cette directive si elle est
absente, sans question ni sauvegarde visible. Si le fichier ne peut pas être
modifié, WarpgateSH ne le change pas et affiche la ligne à ajouter manuellement.

## Ajout d’un profil

`warpgatesh profile add <nom> <url>` ouvre l’Instance Warpgate dans le navigateur
afin que l’Utilisateur humain s’y authentifie normalement et crée son propre
jeton API. Le jeton est ensuite collé une seule fois dans WarpgateSH, validé
auprès de l’instance puis stocké dans le Trousseau macOS.

Le même parcours est proposé par le compagnon graphique. WarpgateSH ne lit pas
les cookies du navigateur et ne recueille jamais le mot de passe Warpgate.

Lors de l’ajout puis des synchronisations, le client vérifie les capacités de
l’API dont il dépend plutôt que d’accepter ou refuser une instance d’après son
seul numéro de version. Une incompatibilité conserve l’Instantané local, ne
modifie pas SSH et indique la capacité manquante ainsi que la version Warpgate
détectée. Une version minimale documentée sera fixée après validation du premier
prototype contre de vraies instances.

Si l’endpoint SSH annoncé par l’API n’est pas joignable — par exemple lorsque le
nom HTTP pointe vers un reverse proxy qui ne publie pas le port SSH — la CLI
demande l’hôte et le port SSH réellement accessibles. Cet endpoint est épinglé
au Profil avec ses clés d’hôte et n’est pas remplacé silencieusement lors des
synchronisations suivantes.

## Compagnon graphique

Le compagnon graphique du MVP vit dans la barre des menus ou la zone de
notification. Il affiche l’état et la dernière synchronisation, permet de
rechercher une cible et de l’ouvrir dans le terminal, de changer le profil par
défaut, de forcer une synchronisation et de gérer les profils et préférences.
Il signale notamment les jetons expirés, les changements de certificat ou de
clé SSH et les instantanés périmés.

Le compagnon n’administre jamais les cibles, les rôles ou les utilisateurs de
l’instance Warpgate.

## Mises à jour

Une installation réalisée par DMG vérifie quotidiennement la présence d’une
nouvelle version et demande toujours une confirmation humaine avant de
l’installer. Une installation gérée par Homebrew se contente de la signaler et
laisse `brew upgrade --cask warpgatesh` effectuer la mise à jour.

Le compagnon, la CLI et l’agent partagent une version et sont toujours remplacés
ensemble par un artefact signé. Aucune mise à jour silencieuse n’est effectuée.

## Installation de la CLI

Le compagnon macOS embarque la même CLI `warpgatesh` que celle utilisée par
l’agent. Homebrew la lie automatiquement dans son répertoire `bin`. Après une
installation par DMG, le compagnon propose un bouton qui crée un lien vers la
CLI dans `/usr/local/bin`, avec l’autorisation administrateur de macOS.

WarpgateSH ne modifie ni le `PATH` ni les fichiers de configuration du shell.

## Confidentialité et diagnostics

WarpgateSH ne collecte aucune télémétrie et n’envoie automatiquement aucun
rapport de crash. Ses journaux tournants restent locaux pendant sept jours et
excluent les jetons, mots de passe, clés privées et contenus de configuration
SSH. L’utilisateur peut produire un diagnostic manuel et le relire avant de le
partager dans une issue GitHub.

## Hors périmètre initial

- Remplacer Terraform ou Ansible.
- Administrer les cibles Warpgate depuis le poste client.
- Imposer une application graphique pour utiliser le projet.
- Créer un réseau privé de type Tailscale : Warpgate reste la passerelle SSH.

## Développement

Le workspace Rust contient le cœur de domaine, le client de l’API utilisateur
Warpgate, la CLI `warpgatesh` et l’agent utilisateur. Le flux de développement
actuel permet d’ajouter un profil, d’enregistrer le jeton dans le Trousseau,
d’épingler les clés SSH présentées, d’installer automatiquement un
`LaunchAgent`, de synchroniser les cibles accessibles en arrière-plan et de
produire la Configuration SSH gérée. La CLI dialogue avec l’agent par un socket
Unix privé pour forcer une synchronisation sans lancer un second écrivain.

```sh
cargo build --workspace
./target/debug/warpgatesh profile add homeblack https://warpgate.example
./target/debug/warpgatesh agent status
./target/debug/warpgatesh ls
./target/debug/warpgatesh status
```

Pour vérifier le socle :

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run --bin warpgatesh -- --help
```

Le compagnon macOS se trouve dans `apps/warpgatesh-companion`. Il lit le même
état local que la CLI et dialogue directement avec l’agent déjà actif : quitter
l’application n’interrompt donc jamais la synchronisation de fond.

```sh
cd apps/warpgatesh-companion
npm install
npm run tauri dev
```

Pour construire un bundle macOS local :

```sh
npm run tauri build -- --bundles app
```

L’application permet de consulter l’état de l’agent et la dernière
synchronisation, de filtrer les cibles avec `⌘ K`, de forcer une synchronisation
et d’ouvrir une cible SSH dans le terminal. Elle gère aussi l’ajout, le
renouvellement et la suppression des profils, le profil par défaut, la cadence
de l’agent et l’ouverture du compagnon à la connexion. Elle se rafraîchit
automatiquement lorsqu’elle est visible et remonte les échecs API, les jetons
refusés, les clés SSH modifiées et les instantanés périmés.

Pour installer localement les deux binaires Release sur un Mac Apple Silicon
équipé de Homebrew, sans modifier le `PATH` :

```sh
cargo install --path crates/warpgatesh-cli --root /opt/homebrew --force --locked
cargo install --path crates/warpgatesh-agent --root /opt/homebrew --force --locked
warpgatesh agent install
```

Sur un Mac Intel, la racine Homebrew habituelle est `/usr/local`. La dernière
commande réenregistre le `LaunchAgent` avec le binaire installé ; les profils,
jetons, clés épinglées et instantanés existants sont conservés.

La prise en charge volontaire des certificats TLS auto-signés reste à
implémenter avant le MVP complet. Le comportement par défaut continue de
s’appuyer sur la validation TLS du système, sans ajouter de parcours de
certificat à l’usage courant.
