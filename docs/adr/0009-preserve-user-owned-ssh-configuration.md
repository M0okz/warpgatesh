# Préserver la configuration SSH appartenant à l’utilisateur

`warpgatesh` ne modifie que son fichier SSH isolé et ne remplace jamais une entrée écrite par l’utilisateur. Lorsqu’un alias court entre en conflit avec la configuration personnelle, le client n’émet pas cet alias, signale le conflit dans son diagnostic et conserve l’alias qualifié de la cible ; la configuration manuelle reste ainsi prioritaire sans rendre la cible Warpgate inaccessible.
