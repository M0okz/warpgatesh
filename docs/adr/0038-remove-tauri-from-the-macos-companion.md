# Retirer Tauri du compagnon macOS

Décision pour la migration 0.1.25. Cette décision remplace le choix du framework
de l’ADR 0015 et termine la transition temporaire de l’ADR 0036.

SwiftUI et AppKit possèdent l’interface, les fenêtres, la barre des menus et les
notifications. Le binaire `warpgatesh-companion` devient un backend Rust autonome,
sans boucle d’événements graphique. Il conserve le protocole JSON-lines privé,
les limites de requête et la délégation des mutations à l’agent indépendant.

Les chemins de données, identifiants macOS/Trousseau et labels de services sont
conservés. Le petit mode lanceur du backend assure les anciens éléments de
connexion. Le mode démo reste isolé. WebKit demeure nécessaire au parcours
natif de connexion à Warpgate ; l’ancien frontend React et l’hôte Tauri sont retirés.

Le packaging assemble explicitement les quatre exécutables universels, avec
`warpgatesh-ui` comme point d’entrée. La version vient du workspace Cargo.
Nix pilote les builds et les contrôles ; le site conserve sa chaîne Node séparée.

Les mises à jour passent à **Sparkle 2.10.1**, moteur natif macOS approuvé par
Gregory le 1er novembre 2026. Sparkle possède la vérification EdDSA, l’extraction,
l’installation autorisée et la relance ; aucun installeur privilégié maison
n’est livré. Le transport natif présente son état au modèle SwiftUI existant.
Les vérifications d’information ont lieu toutes les six heures ; téléchargement
et installation nécessitent une action de l’utilisateur. Le profilage système
et les téléchargements automatiques sont désactivés.

La même archive universelle notarifiée garde sa signature Minisign et son
`latest.json` pour les clients 0.1.24. Elle reçoit une signature Sparkle distincte,
avec un `appcast.xml` signé pour les mises à jour suivantes. Les deux clés ont
des cycles de vie séparés. La clé privée Sparkle est conservée dans le coffre
et injectée dans la CI ; seule la clé publique fait partie du dépôt.

Nix pilote également les contrôles de bundle et de DMG. La signature Developer
ID, les tickets de notarisation des artefacts finaux, les deux parcours de mise
à niveau et la validation des permissions restent des critères de livraison.
Une interface Linux/Windows relève d’une décision ultérieure ; la CLI et
l’agent Linux sont conservés.
