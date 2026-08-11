# Ne jamais bloquer une connexion sur une synchronisation

`warpgatesh <alias>` utilise immédiatement l’Instantané local et ne contacte jamais l’Instance Warpgate avant de lancer OpenSSH. Si l’instantané est périmé, la CLI affiche un avertissement sans attendre de réponse réseau ; seule la Synchronisation forcée demande explicitement une actualisation immédiate.
