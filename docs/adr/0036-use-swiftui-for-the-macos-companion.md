# Utiliser SwiftUI pour le compagnon macOS

Le compagnon macOS adopte l’interface SwiftUI validée : navigation latérale,
écran Accès, profils, préférences, synchronisation, diagnostics, mises à jour,
barre de menus, logo existant et matériau natif translucide. Le minimum reste
macOS 13, avec une distribution universelle Apple Silicon et Intel.

`apps/warpgatesh-macos` contient l’interface native et ses tests. La CLI et l’agent
Rust restent inchangés. L’agent reste l’unique écrivain des profils, jetons et
fichiers SSH. Le compagnon ne duplique pas les appels API ni le stockage secret.

Le lanceur Rust existant exécute le binaire embarqué `warpgatesh-ui`. Celui-ci
ouvre une passerelle privée vers le même lanceur en mode `--native-backend`,
par des pipes hérités, sans port réseau ni socket supplémentaire. Le protocole
JSON par lignes corrèle les réponses à leur identifiant, limite les requêtes à
64 Kio et la concurrence à seize opérations. Les jetons passent dans les pipes,
jamais dans les arguments de processus. Les requêtes ne sont pas journalisées.
Les appels Swift ont un délai maximal et une annulation ; une réponse perdue ne
provoque pas de nouvelle tentative automatique d’une mutation.

Le processus backend conserve temporairement l’hôte Tauri sans fenêtre ni
WebView. Cela réutilise les opérations Rust existantes, l’ouverture de session,
la vérification des signatures et l’installation atomique du produit complet.
L’interface utilisateur, les fenêtres et la barre de menus sont natives.
Le redémarrage après mise à jour attend la fin du processus SwiftUI parent avant
de rouvrir le bundle. Le backend s’arrête à la fermeture des pipes ; quitter le
compagnon conserve l’agent indépendant.

L’ajout d’un profil impose une inspection et une approbation des empreintes SSH.
Le backend les rescane au moment de l’enregistrement et refuse une modification
survenue entre l’inspection et l’approbation. Le renouvellement de jeton, la
suppression de profil, la désinstallation et la mise à jour restent des actions
explicites dans l’interface.

Le bundle conserve son identifiant, ses chemins de données et les identifiants
Trousseau. Aucune conversion ni suppression de données n’est nécessaire. Le
packaging Tauri existant assemble et signe les quatre exécutables universels
(lanceur/backend, interface SwiftUI, CLI et agent) et les ressources Swift avant
notarisation. Les archives de mise à jour existantes restent compatibles.

Le frontend Tauri est conservé comme repli de développement et pour le futur
Linux. La démo historique n’est pas un composant de production. Le mode `--demo`
de l’interface native utilise uniquement des données fictives et ne lance aucun
backend. Le script de prévisualisation le choisit par défaut ; `--live` est
réservé à la validation explicite de l’intégration locale.
