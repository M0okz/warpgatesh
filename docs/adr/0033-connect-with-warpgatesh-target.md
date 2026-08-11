# Se connecter avec `warpgatesh <cible>`

La syntaxe principale de connexion est `warpgatesh <alias>`, qui résout une Cible SSH connue puis exécute l’OpenSSH du système. Les arguments placés après `--` lui sont transmis, les noms des sous-commandes de gestion restent réservés, et un Alias qualifié lève une éventuelle ambiguïté ; `ssh <alias>` continue de fonctionner directement.
