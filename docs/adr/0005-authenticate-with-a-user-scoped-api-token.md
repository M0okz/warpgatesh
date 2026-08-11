# Authentifier le client avec un jeton API personnel

`warpgatesh` utilise un jeton API Warpgate propre à l’utilisateur pour lire l’API utilisateur et conserve ce secret dans le gestionnaire de secrets natif du poste. Le client n’accepte ni mot de passe administrateur ni jeton administrateur : la découverte des cibles doit toujours rester bornée par les autorisations de l’utilisateur connecté.
