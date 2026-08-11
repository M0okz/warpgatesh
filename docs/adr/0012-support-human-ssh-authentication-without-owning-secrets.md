# Prendre en charge l’authentification SSH humaine sans posséder ses secrets

`warpgatesh` respecte les méthodes humaines annoncées par chaque instance Warpgate : approbation web/SSO, mot de passe interactif ou clé publique existante. Le client ne conserve aucun mot de passe SSH ni aucune clé privée ; il peut seulement configurer OpenSSH pour employer une clé ou un agent déjà géré par l’utilisateur, tandis que le jeton d’accès reste exclusivement destiné à l’API HTTP.
