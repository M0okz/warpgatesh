# Distribuer macOS par DMG et Homebrew Cask

WarpgateSH est distribué hors Mac App Store sous forme d’un DMG signé et notarié publié dans les releases GitHub, ainsi que d’un Homebrew Cask. Le même produit contient le compagnon, la CLI et l’agent ; le premier lancement enregistre l’agent dans la session utilisateur, tandis que Homebrew lie automatiquement `warpgatesh` dans le `PATH`. Cette distribution évite les contraintes de sandbox incompatibles avec l’intégration à OpenSSH et à un agent utilisateur.
