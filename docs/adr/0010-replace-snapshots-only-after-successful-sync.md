# Remplacer un instantané seulement après une synchronisation réussie

L’agent remplace atomiquement l’instantané local uniquement après une réponse Warpgate valide et complète : les cibles absentes de cette réponse sont alors retirées immédiatement. Une erreur réseau, serveur ou d’authentification conserve le dernier instantané réussi en le signalant comme périmé, afin qu’un incident transitoire ne produise jamais un fichier SSH vide ou partiel.
