# Déléguer les connexions à OpenSSH

`warpgatesh` maintient un fichier de configuration SSH isolé, inclus par la configuration OpenSSH de l’utilisateur, et délègue les connexions au client OpenSSH du système. Ce choix permet à `ssh <cible>` de fonctionner sans processus `warpgatesh` actif et rend les mêmes cibles disponibles à `scp`, `sftp`, `rsync`, Ansible et aux IDE, plutôt que d’enfermer l’accès dans un client SSH propriétaire.
