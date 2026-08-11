# Agent de synchronisation toujours actif dès le MVP

Le MVP installe un agent de synchronisation toujours actif, indépendant du compagnon graphique, afin que `ssh <cible>` repose sur un instantané local régulièrement actualisé même lorsque l’utilisateur n’ouvre jamais l’interface graphique. `warpgatesh sync` demande à cet agent une synchronisation immédiate plutôt que de constituer l’unique mécanisme de mise à jour.
