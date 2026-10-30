# Connect your Warpgate instance

Before starting, you need:

- the root URL of a Warpgate instance, for example
  `https://bastion.example.org`;
- a Warpgate user allowed to access at least one SSH target;
- network access to both the Warpgate HTTP endpoint and its SSH endpoint.

WarpgateSH supports user access. It does not create Warpgate users, roles, or
targets.

## 1. Sign in with the macOS companion

From **v0.1.22**, the companion can create a personal API token without
copying it from your browser. French and English labels are available. See the
[native profile walkthrough and screenshots](desktop-app.md#profiles).

1. Open **Profiles → Add a profile**.
2. Enter a local name using lowercase letters, digits and hyphens, such as `production`.
3. Enter your instance's HTTPS root URL and select **Sign in to Warpgate**.
4. Authenticate on the official instance page inside the app. Password, SSO and
   MFA remain controlled by Warpgate and your identity provider.
5. Select **Check profile** and compare the displayed SSH host-key fingerprints
   with a trusted source.
6. Enable **I verified these fingerprints through a trusted channel**, then select **Save**.

Only saving creates the token. The agent stores it in the macOS Keychain and
renews it before its server-reported expiry. It never becomes an SSH credential.
If your identity provider disallows embedded sign-in, use the existing-token
option below. Real SSO behavior depends on your instance and provider.

**SSH options** can override a detected hostname or port. Enrollment is refused
if the host keys change between inspection and saving. The first profile becomes
the default profile.

## 2. Alternative: import a personal token or use the CLI

Sign in to Warpgate with the user whose SSH targets you want to use. Open
**Profile → API tokens** and create a personal token. In the companion, expand
**Use an existing token**, paste it into the secure field, then check and save
the profile with the same SSH fingerprint approval.

Treat the token like a password. WarpgateSH validates it against your instance,
stores it in the native secret store and never writes it to SSH configuration.
Imported tokens are not automatically adopted or revoked. To adopt a token managed
by the app, select **Renew token → Renew through the API** on that profile.

The CLI retains its interactive manual-token flow:

```sh
warpgatesh profile add production https://bastion.example.org
```

The command opens the token page, prompts without echoing the secret, discovers
the SSH endpoint and displays its fingerprints. If that endpoint is unreachable,
it asks for a different hostname or port.

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

## Keep browser authentication consistent

For a profile whose Warpgate account uses in-browser SSH approval:

```sh
warpgatesh profile ssh-auth production in-browser
```

The background agent stores this preference in the profile and requests a new
synchronization. Every generated alias, including new targets, uses
`keyboard-interactive` without offering SSH keys or passwords. The preference
survives automatic synchronization, token renewal and re-enrollment from the
companion. `warpgatesh profile list` shows the selected mode.

Existing profiles keep `auto`, which preserves the usual OpenSSH methods. To
restore it:

```sh
warpgatesh profile ssh-auth production auto
```

Use `in-browser` only when the server account supports that method. This is a
client preference: it does not change server authentication requirements, the
approval duration, or the server's “all targets” button. A remembered approval
may still require confirmation after a change of source address or identity.

The CLI and background agent must both include this feature. Older agents do
not support the preference and can discard it when saving profiles; upgrade the
complete application before configuring it. Personal SSH rules loaded earlier
still take precedence over generated aliases.

## Automatic renewal and reconnecting

A profile enrolled through integrated sign-in or **Renew through the API** shows
**Automatic renewal** and the instance's actual token expiry. The independent
agent performs rotation before expiry, even with the companion closed. You can
disable it per profile. Creation requests honor the instance's maximum lifetime,
up to 30 days. Unusually short policies (below six minutes) are not supported.

Failures keep the last usable token and are retried with a delay. Only tokens
created by the app may be revoked after a durable replacement and a grace period;
imported tokens are never revoked automatically. A known manual reminder is used
only when the token's actual expiry is unavailable.

An already expired or revoked token cannot renew itself. Use **Renew token →
Sign in to Warpgate again**, authenticate with the same identity, then the app
creates its replacement. Alternatively, import a fresh personal token, or run:

```sh
warpgatesh login production
```

Manual import replaces the local secret after validation and leaves automatic
management off until you explicitly adopt a managed token again. Review your
personal tokens in Warpgate to revoke any no-longer-used credentials.

If anything fails, continue with [Troubleshooting](troubleshooting.md).
