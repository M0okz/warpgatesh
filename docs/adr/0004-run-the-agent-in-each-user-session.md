# Exécuter l’agent dans chaque session utilisateur

L’agent de synchronisation s’exécute sans privilèges administrateur dans la session de chaque utilisateur, via un LaunchAgent sur macOS puis un service `systemd --user` sur Linux. Les identifiants Warpgate, les autorisations et la configuration SSH étant personnels, cette isolation évite qu’un service système partage des secrets ou mélange les instantanés de plusieurs utilisateurs.
