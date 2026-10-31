import { readFile, readdir, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";

const root = fileURLToPath(new URL("../dist/", import.meta.url));
const pages = [
  "index.html",
  ...(await readdir(path.join(root, "docs"))).map((name) => `docs/${name}`),
];
const errors = [];
const screenshotRoot = path.join(root, "assets/swiftui");
const manifest = JSON.parse(
  await readFile(path.join(screenshotRoot, "manifest.json"), "utf8"),
);
for (const image of manifest.images) {
  const data = await readFile(path.join(screenshotRoot, image.file));
  if (
    data.length < 24 ||
    !data.subarray(0, 8).equals(Buffer.from("89504e470d0a1a0a", "hex"))
  ) {
    errors.push(`${image.file}: expected PNG screenshot`);
    continue;
  }
  if (
    data.readUInt32BE(16) !== image.width ||
    data.readUInt32BE(20) !== image.height ||
    createHash("sha256").update(data).digest("hex") !== image.sha256
  ) errors.push(`${image.file}: screenshot manifest does not match file`);
}
let links = 0;
for (const page of pages) {
  const html = await readFile(path.join(root, page), "utf8");
  if ((html.match(/<h1(?:\s|>)/g) ?? []).length !== 1)
    errors.push(`${page}: expected one h1`);
  for (const match of html.matchAll(/(?:href|src)="([^"]+)"/g)) {
    const href = match[1];
    if (/^(?:[a-z]+:|\/\/)/i.test(href)) continue;
    links++;
    const [target, anchor] = href.split("#");
    let file = path.resolve(
      path.dirname(path.join(root, page)),
      decodeURIComponent(target || path.basename(page)),
    );
    try {
      if ((await stat(file)).isDirectory())
        file = path.join(file, "index.html");
      if (anchor) {
        const content = await readFile(file, "utf8");
        if (!content.includes(`id="${decodeURIComponent(anchor)}"`))
          errors.push(`${page}: missing anchor ${href}`);
      }
    } catch {
      errors.push(`${page}: missing local resource ${href}`);
    }
  }
}
if (errors.length) {
  console.error(errors.join("\n"));
  process.exitCode = 1;
} else
  console.log(`Checked ${pages.length} pages and ${links} local links/assets.`);
