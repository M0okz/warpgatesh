# Create a token and connect

Before starting, you need:

- the root URL of a Warpgate instance, for example
  `https://bastion.example.org`;
- a Warpgate user allowed to access at least one SSH target;
- network access to both the Warpgate HTTP endpoint and its SSH endpoint.

WarpgateSH supports user access. It does not create Warpgate users, roles, or
targets.

## 1. Create a personal API token

Sign in to Warpgate with the same user whose SSH targets you want to use. Open
**Profile → API tokens** and create a personal token. WarpgateSH can open the
corresponding page for you from either the graphical profile form or the CLI.

Treat the token like a password. Paste it only into WarpgateSH. The client
validates it against your instance, stores it in the macOS Keychain, and never
writes it to the generated SSH configuration.

## 2. Add the instance

### With the graphical companion

1. Open **Profils** and select **+ Ajouter**.
2. Enter a short profile name using lowercase letters, digits, and hyphens,
   such as `production`.
3. Enter the Warpgate root URL and your personal API token.
4. Select **Vérifier la connexion**.
5. Compare the displayed SSH host-key fingerprints with a trusted source.
6. Select **Faire confiance et ajouter** only after the fingerprints match.

If the SSH service uses a different public hostname or port from the HTTP
service, open **Adresse SSH différente ?** and provide the reachable endpoint.

### With the CLI

```sh
warpgatesh profile add production https://bastion.example.org
```

The command opens the token page, prompts for the token without echoing it,
discovers the SSH endpoint, and displays its host-key fingerprints. If the
advertised SSH endpoint is unreachable, the CLI asks for a different hostname
or port.

The first profile automatically becomes the default profile.

## 3. Synchronize targets

The background agent requests an initial synchronization when the profile is
saved. To request one immediately:

```sh
warpgatesh sync
warpgatesh status
warpgatesh ls
```

Closing the graphical window does not stop the agent. A failed synchronization
keeps the last complete local snapshot instead of replacing it with partial or
empty data.

## 4. Connect

Use the main CLI:

```sh
warpgatesh app-01
```

Or use any tool that reads your OpenSSH configuration:

```sh
ssh app-01
scp ./backup.tar app-01:/tmp/
```

WarpgateSH delegates the connection to `/usr/bin/ssh`; SSH authentication and
any Warpgate authorization prompt remain owned by OpenSSH and Warpgate.

With multiple profiles, every target also receives a qualified alias:

```sh
ssh app-01.production
ssh app-01.staging
```

Only the default profile receives short aliases. Change it with:

```sh
warpgatesh profile default production
```

## Renew an expired token

Create a new personal token in Warpgate, then use the profile action in the
graphical companion or run:

```sh
warpgatesh login production
```

The previous Keychain value is replaced after the new token is validated.

If anything fails, continue with [Troubleshooting](troubleshooting.md).
