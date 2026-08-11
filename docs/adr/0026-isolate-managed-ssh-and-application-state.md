# Isoler la configuration SSH gérée et l’état de l’application

WarpgateSH ajoute près du début de `~/.ssh/config` une unique Directive d’inclusion vers `~/.ssh/warpgatesh/config`. Les alias et clés d’hôte gérés restent sous `~/.ssh/warpgatesh/`, les profils et l’état non secret sous `~/Library/Application Support/WarpgateSH/`, et les Jetons d’accès exclusivement dans le Trousseau macOS ; la désinstallation peut ainsi retirer uniquement les éléments appartenant au client.
