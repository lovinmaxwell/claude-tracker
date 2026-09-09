#!/usr/bin/env node
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { zipSync } from "./zip-sync.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const dist = join(root, "dist-extension");
const iconsSrc = join(root, "..", "src-tauri", "icons");
const iconsDest = join(dist, "icons");

mkdirSync(iconsDest, { recursive: true });
for (const name of ["32x32.png", "128x128.png", "icon.png"]) {
  copyFileSync(join(iconsSrc, name), join(iconsDest, name));
}

const manifest = {
  manifest_version: 3,
  name: "Quota Tray",
  version: "0.1.0",
  description:
    "Local AI coding quota for Claude Code, Cursor, and GitHub Copilot. No Quota Tray servers.",
  action: {
    default_title: "Quota Tray",
    default_popup: "index.html",
    default_icon: {
      32: "icons/32x32.png",
      128: "icons/128x128.png",
    },
  },
  options_page: "settings.html",
  background: {
    service_worker: "background.js",
    type: "module",
  },
  icons: {
    32: "icons/32x32.png",
    128: "icons/128x128.png",
  },
  permissions: ["storage", "alarms", "cookies"],
  host_permissions: [
    "https://api.anthropic.com/*",
    "https://api2.cursor.sh/*",
    "https://cursor.com/*",
    "https://www.cursor.com/*",
    "https://api.github.com/*",
  ],
};

writeFileSync(join(dist, "manifest.json"), JSON.stringify(manifest, null, 2));

for (const htmlName of ["index.html", "settings.html"]) {
  const htmlPath = join(dist, htmlName);
  const html = readFileSync(htmlPath, "utf8").replace(/ crossorigin(?:="[^"]*")?/g, "");
  writeFileSync(htmlPath, html);
}

const zipPath = join(root, "..", "extension", "QuotaTray-chrome.zip");
mkdirSync(dirname(zipPath), { recursive: true });
zipSync(dist, zipPath);
console.log(`Unpacked extension: ${dist}`);
console.log(`Zip: ${zipPath}`);
console.log("Load unpacked in chrome://extensions (Developer mode).");
