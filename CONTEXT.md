# Accès local Warpgate

Ce contexte décrit les concepts employés par `warpgatesh` pour rendre localement accessibles les ressources SSH publiées par Warpgate.

## Language

**Instance Warpgate**:
Un déploiement Warpgate auquel un utilisateur peut connecter son Client WarpgateSH.
_Avoid_: Serveur Warpgate, bastion, environnement

**Profil Warpgate**:
La relation locale nommée entre un utilisateur et une Instance Warpgate, avec son identité, ses préférences et son Jeton d’accès propres.
_Avoid_: Compte, connexion, configuration d’instance

Le nom local d’un Profil Warpgate est composé uniquement de lettres minuscules, de chiffres et de tirets.

**Profil par défaut**:
Le Profil Warpgate dont les Cibles SSH peuvent être utilisées par leur Alias court.
_Avoid_: Profil actif, instance active

**Client WarpgateSH**:
Le logiciel local générique et communautaire non officiel qui donne accès aux ressources SSH d’une Instance Warpgate et maintient leur représentation sur le poste de l’utilisateur. Il est développé indépendamment du projet Warpgate et ne prétend ni à son affiliation ni à son approbation.
_Avoid_: Application warpctl, outil warpctl, Client warpctl

**Utilisateur humain**:
Une personne qui utilise le Client WarpgateSH dans sa propre session de poste de travail et peut répondre interactivement aux demandes d’authentification.
_Avoid_: Compte de service, agent, opérateur automatique

**Compagnon graphique**:
L’interface facultative du Client WarpgateSH dédiée aux préférences personnelles, à l’état de connexion et à la présentation. Elle n’est jamais requise pour accéder aux ressources SSH.
_Avoid_: Application principale, client graphique

**Cible SSH**:
Une destination SSH publiée par une Instance Warpgate et à laquelle l’utilisateur peut être autorisé à accéder.
_Avoid_: Host, serveur, machine

**Alias court**:
Le nom non qualifié d’une Cible SSH du Profil par défaut, utilisable directement par les clients SSH locaux.
_Avoid_: Alias global, hostname

L’Alias court conserve le nom de la Cible SSH lorsqu’il est compatible avec OpenSSH. Sinon, il est normalisé en minuscules avec des tirets ; une collision reçoit un court suffixe stable dérivé de l’identifiant Warpgate.

**Alias qualifié**:
Le nom stable `<cible>.<profil>` qui identifie sans ambiguïté une Cible SSH parmi tous les Profils Warpgate.
_Avoid_: FQDN, nom complet

**Conflit d’alias**:
La situation où un alias que le Client WarpgateSH souhaite gérer est déjà défini dans la configuration SSH personnelle de l’utilisateur.
_Avoid_: Doublon, collision de host

**Configuration SSH gérée**:
L’ensemble isolé des alias et clés d’hôte que le Client WarpgateSH produit sous `~/.ssh/warpgatesh/` et rend visible à OpenSSH par une unique directive d’inclusion.
_Avoid_: Configuration SSH personnelle, fichier SSH principal

**Directive d’inclusion**:
L’unique ligne `Include ~/.ssh/warpgatesh/config` ajoutée et identifiable dans `~/.ssh/config` pour charger la Configuration SSH gérée.
_Avoid_: Bloc SSH généré, configuration Warpgate inline

**Instantané local**:
La dernière représentation locale réussie des Cibles SSH accessibles à l’utilisateur.
_Avoid_: Cache, liste des hosts

**Instantané périmé**:
Un Instantané local conservé après l’échec d’une synchronisation plus récente et dont l’ancienneté est rendue visible à l’utilisateur.
_Avoid_: Cache invalide, configuration cassée

**Agent de synchronisation**:
Le composant propre à la session de chaque utilisateur qui maintient automatiquement l’Instantané local, indépendamment du Compagnon graphique.
_Avoid_: Compagnon graphique, daemon

**Synchronisation forcée**:
Une demande de mise à jour immédiate de l’Instantané local, sans attendre la prochaine synchronisation automatique.
_Avoid_: Rafraîchissement, resynchronisation manuelle

**Jeton d’accès**:
Un jeton API Warpgate personnel utilisé par le Client WarpgateSH pour synchroniser les données de son Utilisateur humain ; il ne constitue pas son authentification SSH.
_Avoid_: Jeton administrateur, mot de passe Warpgate, token

**Authentification SSH**:
La preuve interactive ou cryptographique présentée à l’Instance Warpgate lorsqu’un Utilisateur humain ouvre une connexion vers une Cible SSH.
_Avoid_: Jeton d’accès, authentification de la cible
