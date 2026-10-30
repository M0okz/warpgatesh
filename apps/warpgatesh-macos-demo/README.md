# WarpgateSH — écran Accès en SwiftUI

Prototype historique de référence. L’interface fonctionnelle est maintenant
dans [`../warpgatesh-macos`](../warpgatesh-macos/README.md).

Prototype macOS natif, isolé du compagnon Tauri existant. Reprend la maquette
fournie : fenêtre sombre, barre latérale de 198 points, cartes cyan/violet/ambre,
synchronisation et liste de cibles.

## Lancer

Depuis la racine du dépôt, sur macOS avec Xcode et Swift 6.2 ou ultérieur :

```sh
sh apps/warpgatesh-macos-demo/scripts/run.sh
```

Le script compile une application locale dans `.build/WarpgateSH Demo.app` et
l’ouvre sans installation. On peut aussi ouvrir `Package.swift` dans Xcode et
lancer le schéma `WarpgateSHDemo`. Minimum de déploiement : macOS 13.

## Interactions

- 54 cibles fictives et un profil fictif `homelab` ; les compteurs reflètent les données.
- Recherche par nom, alias qualifié ou profil, insensible à la casse ; `⌘K` place le focus dans le champ.
- Liste défilante et état sans résultat avec effacement de la recherche.
- Synchronisation simulée pendant 900 ms, bouton temporairement désactivé,
  puis dernière synchronisation « À l’instant ».
- Clic sur une cible : fiche de démonstration, fermeture avec Échap.
- Les autres sections de la barre latérale sont des repères visuels non interactifs.
- Contrôles de fenêtre macOS natifs, redimensionnement, contrôles clavier,
  libellés VoiceOver et styles adaptés au contraste renforcé.

Aucun accès réseau, aucune lecture de secrets, aucun appel à la CLI ou à l’agent,
aucune modification de `~/.ssh`, des profils ou de l’application installée.
Le prototype n’est pas intégré aux scripts Nix ni aux publications de production.

## Validation

```sh
cd apps/warpgatesh-macos-demo
swift build
```

Vérifier visuellement à 804 × 524 points et dans une fenêtre agrandie ; essayer
`⌘K`, une recherche sans résultat, une fiche de cible et la synchronisation.
Vérifier le parcours VoiceOver, l’accès clavier complet et les réglages macOS
Augmenter le contraste / Réduire la transparence avant intégration au produit.
