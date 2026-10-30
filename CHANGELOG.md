# Changelog

Toutes les évolutions notables de WarpgateSH sont documentées ici.

## Non publié

- Fermer la fenêtre conserve le compagnon en barre de menus et masque son icône du Dock ; rouvrir la fenêtre restaure le mode application.

- L’icône du Dock reçoit des marges transparentes pour une taille cohérente avec les autres applications.

- La connexion intégrée à la page officielle de Warpgate permet de créer un
  jeton depuis l’app après authentification et approbation des empreintes SSH.
- L’agent renouvelle les jetons créés par l’app avant leur expiration, avec une
  reprise après interruption et un interrupteur par profil. Les jetons importés
  restent disponibles et ne sont jamais révoqués automatiquement.
- La date d’expiration fournie par l’instance alimente les rappels des profils
  gérés. Un jeton déjà expiré ou révoqué nécessite une nouvelle authentification.

## 0.1.20 — 2026-10-30

- Les liens vers les sites et la documentation sont colorés et se soulignent au survol.
- La page À propos centre le logo, le nom, la version, les informations et le bloc de mise à jour.

- Les préférences sont réparties en Général, Synchronisation et Système. La langue
  peut suivre macOS ou être réglée en français ou en anglais ; les thèmes Warpgate,
  Clair, Noir et Système s’appliquent immédiatement et sont conservés sur ce Mac.
- Les notifications macOS facultatives signalent les incidents persistants et les
  mises à jour du client. La surveillance des versions du serveur est optionnelle
  par profil. Les rappels d’expiration nécessitent une date connue, saisie dans le
  profil ; un renouvellement réussi efface cette ancienne date.
- Un lien dans les préférences ouvre directement le guide de la commande du terminal.
- La bordure verticale de la barre latérale est retirée dans tous les thèmes.

## 0.1.18 — 2026-10-30

- Le compagnon macOS adopte SwiftUI avec six rubriques, une navigation latérale,
  un panneau translucide, le logo de l’application et des animations de survol.
- Les profils, la synchronisation, les préférences, les diagnostics et les mises
  à jour réutilisent le moteur Rust existant via une passerelle locale privée.
  La CLI, l’agent indépendant, les profils et le Trousseau sont conservés.
- L’enregistrement d’un profil refuse les empreintes SSH modifiées depuis leur
  vérification. L’interface protège aussi l’état récent contre une réponse de
  lecture plus ancienne.
- Le README, les guides et le site présentent les captures réelles du mode démo
  SwiftUI, avec des données fictives et les parcours de cette version native.

## 0.1.17 — 2026-10-30

- Une intégration zsh facultative permet de taper directement le nom d’une cible, avec la complétion Tab habituelle. `warpgatesh shell install zsh` active les alias courts et qualifiés dans les nouveaux terminaux ; `warpgatesh shell uninstall zsh` retire cette intégration.
- Les commandes de cibles sont actualisées depuis l’instantané local avant chaque invite, sans accès réseau. Les commandes, alias et fonctions personnels restent prioritaires et les arguments SSH sont transmis à la CLI.
- Le site public présente le produit et fournit les guides d’installation, de connexion, de dépannage et de désinstallation.

- Les changements de bannière SSH dans les commentaires de `ssh-keyscan` ne sont plus signalés comme une rotation des clés hôtes. Les modifications du matériel des clés restent refusées.
- Des tests de contrat couvrent les API utilisateur Warpgate 0.27.5, 0.28.6 et 0.29.1, ainsi que la conservation de l’état local en cas de réponse invalide ou d’erreur HTTP.

## 0.1.16 — 2026-09-10

- Une préférence d’authentification SSH par profil permet de conserver la validation navigateur après chaque régénération des alias, y compris pour les nouvelles cibles. Elle se règle avec `warpgatesh profile ssh-auth <profil> in-browser` et survit au renouvellement du jeton ainsi qu’au réenrôlement du profil. Les profils existants conservent le mode automatique.

## 0.1.15 — 2026-09-10

- Le menu de la barre système conserve la dernière synchronisation réussie et retire le compte à rebours de la suivante.
- Un indicateur tourne dans la barre des menus pendant les synchronisations manuelles et automatiques, puis l’icône habituelle réapparaît.
- L’état et la dernière réussite sont actualisés dès la fin de la synchronisation. L’action manuelle est désactivée pendant le travail et signale une demande impossible.

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
