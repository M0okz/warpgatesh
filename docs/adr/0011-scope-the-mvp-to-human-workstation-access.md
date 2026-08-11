# Limiter le MVP à l’accès humain depuis un poste

Le MVP de `warpgatesh` sert exclusivement les utilisateurs humains dans leur session de poste de travail. Les outils compatibles OpenSSH peuvent réutiliser les alias générés, mais le client ne crée ni comptes de service, ni tickets d’accès, ni mécanismes d’authentification sans interaction ; ces usages nécessitent un modèle de sécurité distinct.
