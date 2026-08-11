# Utiliser un profil par défaut et des alias SSH qualifiés

Chaque cible reçoit un alias qualifié stable `<cible>.<profil>`, tandis que seules les cibles du profil par défaut reçoivent également leur nom non qualifié comme alias court. Cette règle conserve l’expérience `ssh <cible>` tout en évitant les collisions multi-instance ; `warpgatesh use <profil>` change le profil par défaut et réconcilie atomiquement les alias courts.
