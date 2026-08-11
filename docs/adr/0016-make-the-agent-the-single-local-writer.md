# Faire de l’agent l’unique écrivain local

L’agent est seul autorisé à modifier les profils, préférences, instantanés et fichiers SSH ; la CLI et le compagnon lui transmettent leurs demandes par un canal local limité à la session utilisateur. Cette sérialisation empêche les courses et écritures partielles sans ralentir `ssh <cible>`, qui lit toujours directement la configuration OpenSSH et ne contacte jamais l’agent.
