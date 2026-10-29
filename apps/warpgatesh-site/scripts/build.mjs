import { readFile, writeFile, mkdir, cp, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { marked } from "marked";

const root = fileURLToPath(new URL("../", import.meta.url));
const repository = path.resolve(root, "../..");
const output = path.join(root, "dist");
const guides = [
  ["installation", "Install on macOS"],
  ["installation-linux", "Install on Linux"],
  ["getting-started", "Connect your first target"],
  ["troubleshooting", "Troubleshooting"],
  ["uninstall", "Uninstall"],
];
const escape = (value) =>
  value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll('"', "&quot;");

// Never remove paths outside this site's generated output.
await rm(output, { recursive: true, force: true });
await cp(path.join(root, "public"), output, { recursive: true });
await mkdir(path.join(output, "docs"), { recursive: true });
const home = await readFile(path.join(output, "index.html"), "utf8");
const header = home
  .match(/<header[\s\S]*?<\/header>/)[0]
  .replace('href="./"', 'href="../"')
  .replaceAll('src="assets/', 'src="../assets/')
  .replaceAll('href="#companion"', 'href="../#companion"')
  .replaceAll('href="#cli"', 'href="../#cli"')
  .replace('href="docs/"', 'href="./"');

// Resolve Markdown paths relative to each source guide. Local supported guides
// stay on the site; architecture and maintainer references point to GitHub.
function rewriteLink(href, isImage = false) {
  if (/^(?:[a-z]+:|\/\/|#)/i.test(href)) return href;
  const [file, fragment] = href.split("#");
  const resolved = path.posix.normalize(path.posix.join("docs", file));
  const slug = path.posix.basename(resolved, ".md");
  if (
    resolved === `docs/${slug}.md` &&
    guides.some(([name]) => name === slug)
  ) {
    return `${slug}.html${fragment ? `#${fragment}` : ""}`;
  }
  return `https://${isImage ? "raw.githubusercontent.com" : "github.com"}/M0okz/warpgatesh/${isImage ? "" : "blob/"}main/${resolved}${fragment ? `#${fragment}` : ""}`;
}
const renderer = new marked.Renderer();
renderer.link = function (token) {
  return `<a href="${escape(rewriteLink(token.href))}">${this.parser.parseInline(token.tokens)}</a>`;
};
renderer.image = (token) =>
  `<img src="${escape(rewriteLink(token.href, true))}" alt="${escape(token.text)}" loading="lazy">`;
const headingIds = new Map();
renderer.heading = function (token) {
  const base = token.text
    .toLowerCase()
    .replace(/<[^>]*>/g, "")
    .replace(/[^\p{L}\p{N}\s-]/gu, "")
    .replace(/\s+/g, "-");
  const count = headingIds.get(base) ?? 0;
  headingIds.set(base, count + 1);
  const id = count ? `${base}-${count}` : base;
  return `<h${token.depth} id="${id}">${this.parser.parseInline(token.tokens)}</h${token.depth}>`;
};
marked.use({ renderer });

async function page(slug, title, content, source) {
  const sidebar = `<aside class="docs-sidebar" aria-label="Documentation"><strong>Documentation</strong><nav aria-label="Documentation guides"><a href="./"${slug === "index" ? ' aria-current="page"' : ""}>Overview</a>${guides.map(([name, label]) => `<a href="${name}.html"${slug === name ? ' aria-current="page"' : ""}>${label}</a>`).join("")}</nav></aside>`;
  const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="theme-color" content="#111217"><title>${escape(title)} — WarpgateSH Docs</title><meta name="description" content="${escape(title)}. Official WarpgateSH project guide."><link rel="icon" href="../assets/app-icon.svg" type="image/svg+xml"><link rel="stylesheet" href="../styles.css"><script src="../app.js" defer></script></head><body><a class="skip" href="#main">Skip to content</a>${header}<main id="main" class="docs-layout wrap">${sidebar}<article class="docs-content">${content}<p class="docs-meta">Built from the project's versioned documentation. <a href="https://github.com/M0okz/warpgatesh/blob/main/${source}">View source on GitHub</a>.</p></article></main><footer class="footer wrap"><p>WarpgateSH is an independent, unofficial community project.<br>Not affiliated with or endorsed by the Warpgate project.</p><nav aria-label="Footer"><a href="../">Home</a><a href="https://github.com/M0okz/warpgatesh">GitHub</a><a href="https://github.com/M0okz/warpgatesh/releases/latest">Download</a></nav></footer></body></html>`;
  await writeFile(path.join(output, "docs", `${slug}.html`), html);
}
await page(
  "index",
  "Get started",
  `<p class="eyebrow">WarpgateSH / Guides</p><h1>From your instance<br>to your first connection.</h1><p>Install WarpgateSH, add a profile for your Warpgate instance, and use familiar SSH aliases from your workstation.</p><h2>Choose your platform</h2><ul><li><a href="installation.html">macOS 13+ — signed universal app, CLI, and background agent</a></li><li><a href="installation-linux.html">Linux beta — CLI and per-user systemd agent</a></li></ul><h2>Connect and keep going</h2><ol><li><a href="getting-started.html">Create a personal API token, verify host keys, and connect.</a></li><li><a href="troubleshooting.html">Diagnose installation, synchronization, or SSH problems.</a></li><li><a href="uninstall.html">Remove WarpgateSH and understand what data remains.</a></li></ol><h2>Understand the boundaries</h2><p>WarpgateSH discovers your authorized SSH targets. It does not administer Warpgate users or roles. Your API token synchronizes targets; OpenSSH and Warpgate handle SSH authentication.</p><p><a href="https://warpgate.null.page/">Read the upstream Warpgate documentation</a> for instance installation and administration.</p>`,
  "docs/README.md",
);
for (const [slug, title] of guides) {
  headingIds.clear();
  const markdown = await readFile(
    path.join(repository, "docs", `${slug}.md`),
    "utf8",
  );
  await page(slug, title, marked.parse(markdown), `docs/${slug}.md`);
}
const origin = "https://warpgatesh.homeblack.fr";
await writeFile(
  path.join(output, "robots.txt"),
  `User-agent: *\nAllow: /\nSitemap: ${origin}/sitemap.xml\n`,
);
const urls = ["/", "/docs/", ...guides.map(([slug]) => `/docs/${slug}.html`)];
await writeFile(
  path.join(output, "sitemap.xml"),
  `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">${urls.map((url) => `<url><loc>${origin}${url}</loc></url>`).join("")}</urlset>\n`,
);
console.log(
  `Built landing page and ${guides.length + 1} documentation pages in ${output}`,
);
