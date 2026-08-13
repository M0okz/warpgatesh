export function createUpdaterManifest({ version, notes, pubDate, url, signature }) {
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`Invalid updater version: ${JSON.stringify(version)}`);
  }
  if (!url.startsWith("https://")) {
    throw new Error("Updater artifacts must use HTTPS");
  }
  if (!signature.trim()) {
    throw new Error("The updater signature is empty");
  }

  const artifact = { url, signature: signature.trim() };
  return {
    version,
    notes: notes.trim(),
    pub_date: pubDate,
    platforms: {
      "darwin-aarch64": artifact,
      "darwin-x86_64": artifact,
    },
  };
}
