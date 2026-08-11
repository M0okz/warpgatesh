# Installer une CLI embarquée sans modifier le shell

Le bundle macOS contient l’unique exécutable `warpgatesh` partagé par l’usage en terminal et le produit installé. Le Homebrew Cask le lie automatiquement dans le préfixe Homebrew ; après une installation par DMG, le compagnon propose de le lier dans `/usr/local/bin` avec l’autorisation administrateur de macOS. WarpgateSH ne modifie ni le `PATH` ni les fichiers du shell.
