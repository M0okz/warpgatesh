# Troubleshooting

Start with the local status commands:

```sh
warpgatesh status
warpgatesh doctor
warpgatesh agent status
```

If the CLI is missing, open **Préférences → Intégration terminal** in the app
and install it first.

## The background agent is not running

Re-register the per-user LaunchAgent:

```sh
warpgatesh agent install
warpgatesh agent status
```

The agent belongs to your macOS login session and does not require a root
daemon. Closing the WarpgateSH window does not stop it.

## macOS repeatedly asks for Keychain access

WarpgateSH stores personal API tokens under the Keychain service
`dev.warpgatesh.api-token`. When macOS asks whether the signed WarpgateSH agent
may read that item, choose **Always Allow** if you trust the installed app.
Choosing **Allow** grants access only for that attempt and can make the prompt
return at the next synchronization.

Never approve an unexpected unsigned binary. Reinstall the official notarized
DMG if the requesting application is not WarpgateSH.

## Warpgate rejects the API token

The token may be expired, revoked, copied incompletely, or owned by another
account. Create a new personal token while signed in as the intended Warpgate
user, then replace it:

```sh
warpgatesh login <profile>
warpgatesh sync
```

## The HTTPS endpoint is not trusted

WarpgateSH currently uses the macOS system trust store. For an internal
certificate authority, install and trust that CA in macOS before adding the
profile. The client does not silently disable TLS certificate verification.

Use the instance root URL, not a copied `@warpgate` application page; copied
Warpgate pages are normalized automatically, but a stable root URL is easier to
diagnose.

## The advertised SSH endpoint is unreachable

The HTTP and SSH services may use different public addresses or ports. Confirm
the reachable SSH endpoint with the Warpgate administrator and test it:

```sh
nc -vz bastion.example.org 2222
ssh-keyscan -T 5 -p 2222 bastion.example.org
```

In the graphical profile form, select **Adresse SSH différente ?**. The CLI asks
for an alternate endpoint automatically after the advertised endpoint times
out.

## Warpgate reports a changed SSH host key

WarpgateSH fails closed when the pinned Warpgate SSH key changes. Do not work
around this warning until the administrator confirms the rotation and provides
the new fingerprint through a trusted channel.

After verification, remove and add the affected profile again. This deletes the
old pin, presents the new fingerprints, and asks for explicit approval.

## No targets appear after synchronization

- Confirm that the Warpgate user can see SSH targets in the Warpgate UI.
- Confirm that the targets are of type SSH; other protocol targets are ignored.
- Run `warpgatesh sync`, then `warpgatesh ls`.
- Check the error shown by the companion or `warpgatesh status`.

Removing a user's access or deleting a target removes its alias on the next
successful complete synchronization.

## A short alias is missing

Short aliases belong only to the default profile. Qualified aliases such as
`target.production` are always generated.

WarpgateSH also preserves aliases already defined in your personal SSH files.
When a manual `Host` entry conflicts with a generated short name, the manual
entry wins and WarpgateSH keeps only the qualified alias.

Inspect OpenSSH's resolved configuration with:

```sh
ssh -G target.production | head
```

Your `~/.ssh/config` must contain this managed include:

```sshconfig
Include ~/.ssh/warpgatesh/config
```

## A synchronization fails while existing aliases still work

This is expected fail-safe behavior. Connections use the last successful local
snapshot and never wait for a live API request. Fix the reported token, network,
host-key, or compatibility error, then request another synchronization.

## Collect diagnostics for an issue

Preview the local files before exporting:

```sh
warpgatesh diagnostics preview
warpgatesh diagnostics export
```

The graphical companion provides the same actions under **Préférences →
Diagnostics locaux**. Logs are retained for seven days, sanitized, and never uploaded
automatically. Review the ZIP before attaching it to a GitHub issue.

When reporting a problem, include the WarpgateSH version, macOS version,
installation method, `warpgatesh doctor` output, and exact reproduction steps.
Do not paste API tokens, private keys, or unredacted SSH configuration.
