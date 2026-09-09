import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { safeAssetName, writeSharePack } from "./write-share-pack.mjs";

test("strips spaces from NSIS setup filenames so links and chats do not break", () => {
  assert.equal(safeAssetName("Quota Tray_0.1.0_x64-setup.exe"), "QuotaTray_0.1.0_x64-setup.exe");
});

test("writes SHA256 sums and a Teams-safe note that tells people to share the GitHub link", () => {
  const dir = mkdtempSync(join(tmpdir(), "quota-tray-share-"));
  try {
    const srcDir = join(dir, "src");
    const outDir = join(dir, "out");
    mkdirSync(srcDir);
    const payload = Buffer.from("fake-nsis-installer");
    const src = join(srcDir, "Quota Tray_0.1.0_x64-setup.exe");
    writeFileSync(src, payload);

    const result = writeSharePack({
      files: [src],
      outDir,
      version: "0.1.0",
      releaseUrl: "https://github.com/lovinmaxwell/claude-tracker/releases/latest",
    });

    const destName = "QuotaTray_0.1.0_x64-setup.exe";
    const dest = join(outDir, destName);
    const hash = createHash("sha256").update(payload).digest("hex");

    assert.equal(result.assets[0].name, destName);
    assert.equal(readFileSync(dest).equals(payload), true);
    assert.match(readFileSync(join(outDir, "SHA256SUMS.txt"), "utf8"), new RegExp(`${hash}  ${destName}`));

    const note = readFileSync(join(outDir, "TEAMS-SHARE.txt"), "utf8");
    assert.match(note, /Do not attach/i);
    assert.match(note, /Teams/i);
    assert.match(note, /https:\/\/github\.com\/lovinmaxwell\/claude-tracker\/releases\/latest/);
    assert.match(note, /https:\/\/github\.com\/lovinmaxwell\/claude-tracker\b/);
    assert.match(note, new RegExp(hash));
    assert.match(note, /QuotaTray_0\.1\.0_x64-setup\.exe/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
