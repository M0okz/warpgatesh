# Construire le compagnon graphique avec Tauri

Le compagnon graphique utilise Tauri 2 afin de partager une seule interface entre macOS et Linux, tout en réutilisant le cœur Rust du projet et les primitives de barre système et de distribution de chaque plateforme. L’agent reste un processus indépendant : fermer l’interface Tauri ne suspend ni la synchronisation ni l’accès SSH.
