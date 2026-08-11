# Gérer séparément les clés d’hôte SSH de Warpgate

Lors de la création d’un profil, `warpgatesh` présente l’empreinte de la clé d’hôte du point d’entrée SSH Warpgate, exige une confirmation initiale puis la conserve dans un registre `known_hosts` isolé. Toutes les cibles de ce profil réutilisent cette confiance et un changement de clé bloque les connexions jusqu’à validation, sans modifier le registre personnel de l’utilisateur ni répéter la question pour chaque alias.
