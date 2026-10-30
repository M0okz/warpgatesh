# WarpgateSH website

Static product website and documentation for WarpgateSH. The landing page uses
the existing app identity; product demonstrations contain fictional targets.
No analytics, third-party fonts, runtime API calls, or visitor data collection.

## Build and preview

From the repository root, use the reference Nix environment:

```sh
nix run .#build-site
nix develop --command bash -c 'cd apps/warpgatesh-site && npm run preview'
```

Open <http://127.0.0.1:4173>. The complete static output is `dist/`.
Serve this directory with a static web server. `deployment/nginx-site.conf`
provides an example; adapt its hostname and document root to your environment.
All local assets and links are
relative, so the output also works under a subdirectory.

The build renders the existing user guides in the repository's `docs/`
directory. Edit those Markdown sources to update installation and troubleshooting
instructions. Architecture and maintainer links stay on GitHub.

macOS download links use the project's latest release page rather than a pinned
artifact filename. Linux installation commands follow the versioned Linux guide.
The companion illustration is HTML/CSS, explicitly identified as an illustration.

The intended public domain is `warpgatesh.homeblack.fr`.
The production reverse proxy, firewall, and DNS configuration is maintained
separately in the infrastructure repository. The website CI builds and checks
the static artifact; production delivery is currently performed manually.
