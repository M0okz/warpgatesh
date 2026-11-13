# Exposer les Cibles SSH comme commandes zsh sur activation explicite

`warpgatesh shell install zsh` ajoute un bloc balisé dans `${ZDOTDIR:-$HOME}/.zshrc`, sans modifier le PATH. Le bloc charge `warpgatesh shell init zsh`. La désinstallation retire uniquement ce bloc ; les liens symboliques et les blocs ambigus nécessitent une configuration manuelle. L’installation standard reste inchangée.

Une fonction zsh est créée pour chaque Alias court ou qualifié compatible avec le shell et la CLI. Les noms réservés aux commandes de gestion sont exclus ; l’Alias qualifié reste disponible. Les fonctions appellent `warpgatesh <alias> -- <arguments>` et deviennent complétables par le système de complétion zsh existant. L’intégration ne reconfigure pas ce système.

Un hook `precmd` actualise les fonctions à partir de l’Instantané local avant chaque invite, sans synchronisation réseau. Les commandes, alias, fonctions et mots réservés personnels sont prioritaires. Seules les fonctions dont le corps correspond encore à celui installé sont retirées lors du renouvellement ; une fonction remplacée par l’utilisateur est préservée. Une erreur de lecture conserve les fonctions précédentes. Les sessions SSH restent gérées par OpenSSH.

Le compagnon macOS propose désormais cette activation après l’installation réussie de la CLI embarquée. « Plus tard » conserve la configuration du shell. Préférences → Système affiche l’état réel du bloc géré et permet de l’activer ou le retirer. CLI et compagnon partagent le même éditeur réversible et les mêmes refus pour les fichiers ambigus ou symboliques. Une activation rappelle explicitement d’ouvrir un nouveau terminal ; elle ne modifie pas les sessions déjà ouvertes.
