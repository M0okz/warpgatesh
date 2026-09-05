# Changelog

Toutes les évolutions notables de WarpgateSH sont documentées ici.

## 0.1.14 — 2026-09-06

- Après une mise à jour, macOS rouvre le paquet installé une fois l’ancienne instance terminée. Un échec du redémarrage de l’agent ne bloque plus la réouverture du compagnon.
- Les opérations réseau, disque et système du compagnon s’exécutent sur des threads adaptés aux appels bloquants ; l’inspection d’un profil ne provoque plus de panique du moteur asynchrone.
- Un enregistrement effectué pendant une lecture de l’état attend désormais un nouvel instantané. Les réponses anciennes ne remplacent plus une progression de mise à jour reçue entre-temps.
- Le menu suit les mises à jour par événements ; le téléchargement ne déclenche plus de lecture complète de l’état chaque seconde. Les erreurs de lecture temporaires disparaissent au retour à la normale.
- Les étapes et erreurs de mise à jour sont conservées dans les diagnostics locaux, sans journaliser les notes de version ni chaque pourcentage de téléchargement.

## 0.1.13 — 2026-09-02

### Linux en bêta

- La CLI `warpgatesh` et l’agent de synchronisation fonctionnent désormais sous Linux avec un service `systemd --user` qui ne demande aucun accès administrateur.
- Les jetons Warpgate sont conservés dans le service de secrets natif du bureau, via Secret Service, et ne sont jamais écrits dans les fichiers de configuration.
- Les alias OpenSSH, la synchronisation automatique, `warpgatesh sync` et la connexion directe à une cible conservent le même comportement que sous macOS.

### Distribution Homebrew

- Le Tap public `M0okz/warpgatesh` permet d’installer la CLI et l’agent sur macOS ou Linux avec Homebrew/Linuxbrew.
- L’installation de l’agent attend désormais que son canal de communication soit réellement disponible, ce qui évite un faux échec lors du premier démarrage.
- Une mise à niveau Homebrew actualise le chemin versionné de l’agent et redémarre correctement son service utilisateur.

### Administration et documentation

- La nouvelle commande `warpgatesh agent uninstall` arrête et retire proprement le service d’arrière-plan sans supprimer les profils ni les jetons.
- Le guide Linux couvre les prérequis, l’installation, les mises à niveau et la désinstallation.

## 0.1.12 — 2026-08-20

### Mise à jour intégrée

- WarpgateSH recherche désormais automatiquement une nouvelle version toutes les six heures.
- L’action « Rechercher les mises à jour… » du menu de la barre des menus est de nouveau cliquable et lance immédiatement une vérification forcée.

### Fiabilité de la synchronisation

- Un premier échec transitoire reste silencieux pendant que l’agent réessaie automatiquement.
- Une alerte est affichée uniquement après deux échecs consécutifs ; toute synchronisation réussie remet le compteur à zéro.
- Les anciens fichiers d’état de l’agent restent compatibles et sont migrés sans intervention.

## 0.1.11 — 2026-08-19

### Synchronisation

- L’agent suit désormais les changements d’adresse ou de port SSH annoncés par Warpgate, sans demander de recréer le profil lorsque la clé d’hôte épinglée reste identique.
- Les profils et les alias SSH enregistrent le nouvel endpoint après une synchronisation réussie.

### Interface

- La page des profils permet de régénérer immédiatement les alias SSH.
- La suppression d’un profil utilise maintenant une confirmation intégrée, plus claire que la boîte de dialogue système.

### Correctifs CLI

- `warpgatesh <cible> -- <commande>` conserve correctement les options OpenSSH avant la destination et place la commande distante après celle-ci.
- `warpgatesh agent install` retrouve correctement l’agent inclus dans l’application lorsque la CLI est appelée depuis son lien `/usr/local/bin/warpgatesh`.

### Documentation et diagnostics

- Les guides publics couvrent désormais l’installation, la première connexion, le dépannage, la désinstallation et la contribution.
- Cette publication inclut également les diagnostics locaux et l’export expurgé préparés pour v0.1.10, qui était restée en brouillon et n’avait pas été proposée aux utilisateurs.

## 0.1.10 — 2026-08-13

### Nouveauté

- L’agent et le compagnon enregistrent désormais des journaux structurés quotidiens dans `~/Library/Logs/WarpgateSH/`, avec une conservation automatique limitée à sept jours.
- Les préférences permettent de prévisualiser les fichiers, le nombre d’événements et leur taille, puis de créer une archive ZIP destinée à une issue GitHub.
- La CLI propose les mêmes fonctions avec `warpgatesh diagnostics preview` et `warpgatesh diagnostics export`.

### Confidentialité

- Les champs sensibles sont expurgés à l’écriture et une seconde fois lors de l’export ; les jetons, mots de passe, clés privées et contenus de configuration SSH ne sont pas journalisés intentionnellement.
- L’archive n’expose pas le chemin absolu du compte macOS et aucun diagnostic n’est envoyé automatiquement.
- Les anciennes sorties brutes de `launchd` sont désactivées afin d’éviter des fichiers sans rotation qui grossissent indéfiniment.

## 0.1.9 — 2026-08-13

### Correctif

- Une installation réalisée depuis le DMG n’est plus confondue avec Homebrew lorsque la CLI intégrée est liée dans `/usr/local/bin`.
- Le bouton de mise à jour signée reste ainsi disponible pour les installations directes, tandis qu’un Cask Homebrew réellement installé conserve sa commande `brew upgrade --cask warpgatesh`.

## 0.1.8 — 2026-08-13

### Amélioration

- Le menu de la barre des menus affiche désormais un voyant vert lorsque l’agent répond et un voyant rouge lorsqu’il est arrêté ou indisponible.
- La ligne d’état de l’agent ouvre WarpgateSH, ce qui la rend utile tout en conservant les informations de synchronisation en lecture seule.

### Mise à jour intégrée

- Cette version est la première destinée à valider le parcours complet de mise à jour signée depuis WarpgateSH 0.1.7, avec consentement explicite, relance de l’application et reprise de l’agent.

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
