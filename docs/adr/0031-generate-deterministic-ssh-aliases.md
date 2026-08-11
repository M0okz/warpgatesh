# Générer des alias SSH déterministes

Les noms de Profils Warpgate sont limités aux lettres minuscules, chiffres et tirets. Le nom d’une Cible SSH est conservé lorsqu’il est directement compatible avec OpenSSH ; sinon il est normalisé en minuscules avec des tirets, et une collision reçoit un court suffixe stable dérivé de l’identifiant Warpgate. La CLI affiche toujours l’alias effectivement produit.
