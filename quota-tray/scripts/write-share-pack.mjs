import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const DEFAULT_RELEASE_URL =
  "https://github.com/lovinmaxwell/claude-tracker/releases/latest";

export function safeAssetName(originalBasename) {
  return originalBasename.replace(/\s+/g, "");
}

export function sha256File(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

export function readAppVersion(root) {
  const conf = JSON.parse(readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf8"));
  if (typeof conf.version !== "string" || conf.version.length === 0) {
    throw new Error("tauri.conf.json is missing version");
  }
  return conf.version;
}

/**
 * @param {{ files: string[], outDir: string, version: string, releaseUrl?: string }} opts
 */
export function writeSharePack({ files, outDir, version, releaseUrl = DEFAULT_RELEASE_URL }) {
  if (!files.length) {
    throw new Error("writeSharePack requires at least one file");
  }

  mkdirSync(outDir, { recursive: true });

  const assets = files.map((src) => {
    const name = safeAssetName(basename(src));
    const dest = join(outDir, name);
    if (resolve(src) !== resolve(dest)) {
      copyFileSync(src, dest);
    }
    return { name, sha256: sha256File(dest) };
  });

  const sums = assets.map((asset) => `${asset.sha256}  ${asset.name}`).join("\n") + "\n";
  writeFileSync(join(outDir, "SHA256SUMS.txt"), sums);

  const note = formatTeamsShare({ version, assets, releaseUrl });
  writeFileSync(join(outDir, "TEAMS-SHARE.txt"), note);
  writeFileSync(join(outDir, "RELEASE-NOTES.md"), formatReleaseNotes({ version, assets, releaseUrl }));

  return { assets, outDir };
}

export function formatTeamsShare({ version, assets, releaseUrl }) {
  const rows = assets.map((asset) => `${asset.name}\nSHA-256: ${asset.sha256}`).join("\n\n");
  return `Quota Tray ${version}

Paste this message (or attach only this .txt file) in Microsoft Teams.
Do not attach the .exe or .zip. Teams uses Microsoft Defender, which often
flags unsigned NSIS setup wizards as a virus even when the file is clean.

Download from GitHub instead:
${releaseUrl}

Or clone (safest to paste in Teams — no attachment scan):
https://github.com/lovinmaxwell/claude-tracker

${rows}

Windows SmartScreen may say "Unknown publisher" because this build is not
Authenticode-signed. Choose More info → Run anyway.

Chrome: unzip QuotaTray-chrome.zip → chrome://extensions → Developer mode
→ Load unpacked → select the unzipped folder (must contain manifest.json).
`;
}

export function formatReleaseNotes({ version, assets, releaseUrl }) {
  const rows = assets.map((asset) => `- \`${asset.name}\` SHA-256 \`${asset.sha256}\``).join("\n");
  return `Quota Tray ${version}

Unsigned local builds. **Do not attach these binaries to Microsoft Teams or Outlook** — Defender commonly false-positives NSIS \`setup.exe\` files. Share this release URL instead: ${releaseUrl}

${rows}

SmartScreen: More info → Run anyway. Chrome extension: Load unpacked from the unzipped folder.
`;
}

function findFirst(dir, predicate) {
  try {
    return readdirSync(dir)
      .filter(predicate)
      .map((name) => join(dir, name))[0];
  } catch {
    return undefined;
  }
}

function resolveWindowsInstaller(root) {
  const dirs = [
    join(root, "target", "release", "bundle", "nsis"),
    join(root, "target", "x86_64-pc-windows-msvc", "release", "bundle", "nsis"),
  ];
  for (const dir of dirs) {
    const found = findFirst(dir, (name) => name.endsWith("-setup.exe"));
    if (found) {
      return found;
    }
  }
  throw new Error("No NSIS *-setup.exe found under target/**/bundle/nsis");
}

function parseArgs(argv) {
  return {
    windows: argv.includes("--windows"),
    chrome: argv.includes("--chrome"),
    notesOnly: argv.includes("--notes"),
  };
}

function main() {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const args = parseArgs(process.argv.slice(2));
  const outDir = join(root, "dist-share");
  const version = readAppVersion(root);
  const files = [];

  if (args.windows) {
    files.push(resolveWindowsInstaller(root));
  }
  if (args.chrome) {
    files.push(join(root, "extension", "QuotaTray-chrome.zip"));
  }
  if (args.notesOnly) {
    const existing = readdirSync(outDir)
      .filter((name) => name.endsWith(".exe") || name.endsWith(".zip"))
      .map((name) => join(outDir, name));
    if (!existing.length) {
      throw new Error("dist-share has no .exe/.zip to describe");
    }
    writeSharePack({ files: existing, outDir, version });
    console.log(`Updated notes in ${outDir}`);
    return;
  }
  if (!files.length) {
    throw new Error("Pass --windows and/or --chrome");
  }

  const result = writeSharePack({ files, outDir, version });
  for (const asset of result.assets) {
    console.log(`${asset.name}  ${asset.sha256}`);
  }
  console.log(`Share pack: ${outDir}`);
}

function invokedAsCli() {
  const entry = process.argv[1];
  if (!entry) {
    return false;
  }
  try {
    return resolve(fileURLToPath(import.meta.url)) === resolve(entry);
  } catch {
    return false;
  }
}

if (invokedAsCli()) {
  main();
}
