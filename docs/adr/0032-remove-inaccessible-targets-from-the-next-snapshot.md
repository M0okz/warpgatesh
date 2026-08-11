# Retirer les cibles inaccessibles du prochain instantané

Lorsqu’une synchronisation complète réussie ne contient plus une Cible SSH auparavant accessible, WarpgateSH retire immédiatement ses alias du nouvel Instantané local et expose ce retrait dans son résumé d’état. Le client ne tente pas de fermer les sessions OpenSSH déjà établies, puisqu’il ne possède ni ne relaie ces connexions.
