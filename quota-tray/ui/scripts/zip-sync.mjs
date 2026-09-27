import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, posix, relative, sep } from "node:path";
import { deflateRawSync } from "node:zlib";

/** Minimal zip writer so we don't add a packing dependency. */
export function zipSync(sourceDir, outFile) {
  const files = [];
  walk(sourceDir, sourceDir, files);
  const chunks = [];
  const central = [];
  let offset = 0;
  for (const file of files) {
    const data = readFileSync(file.abs);
    const name = file.rel.split(sep).join(posix.sep);
    const nameBuf = Buffer.from(name, "utf8");
    const compressed = deflateRawSync(data);
    const crc = crc32(data);
    const local = Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50, 0);
    local.writeUInt16LE(20, 4);
    local.writeUInt16LE(0, 6);
    local.writeUInt16LE(8, 8);
    local.writeUInt16LE(0, 10);
    local.writeUInt16LE(0, 12);
    local.writeUInt32LE(crc, 14);
    local.writeUInt32LE(compressed.length, 18);
    local.writeUInt32LE(data.length, 22);
    local.writeUInt16LE(nameBuf.length, 26);
    local.writeUInt16LE(0, 28);
    chunks.push(local, nameBuf, compressed);
    const hdr = Buffer.alloc(46);
    hdr.writeUInt32LE(0x02014b50, 0);
    hdr.writeUInt16LE(20, 4);
    hdr.writeUInt16LE(20, 6);
    hdr.writeUInt16LE(0, 8);
    hdr.writeUInt16LE(8, 10);
    hdr.writeUInt16LE(0, 12);
    hdr.writeUInt16LE(0, 14);
    hdr.writeUInt32LE(crc, 16);
    hdr.writeUInt32LE(compressed.length, 20);
    hdr.writeUInt32LE(data.length, 24);
    hdr.writeUInt16LE(nameBuf.length, 28);
    hdr.writeUInt16LE(0, 30);
    hdr.writeUInt16LE(0, 32);
    hdr.writeUInt16LE(0, 34);
    hdr.writeUInt16LE(0, 36);
    hdr.writeUInt32LE(0, 38);
    hdr.writeUInt32LE(offset, 42);
    central.push(hdr, nameBuf);
    offset += local.length + nameBuf.length + compressed.length;
  }
  const centralBuf = Buffer.concat(central);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(0, 4);
  end.writeUInt16LE(0, 6);
  end.writeUInt16LE(files.length, 8);
  end.writeUInt16LE(files.length, 10);
  end.writeUInt32LE(centralBuf.length, 12);
  end.writeUInt32LE(offset, 16);
  end.writeUInt16LE(0, 20);
  writeFileSync(outFile, Buffer.concat([...chunks, centralBuf, end]));
}

function walk(dir, root, out) {
  for (const name of readdirSync(dir)) {
    const abs = join(dir, name);
    const st = statSync(abs);
    if (st.isDirectory()) {
      walk(abs, root, out);
    } else {
      out.push({ abs, rel: relative(root, abs) });
    }
  }
}

function crc32(buf) {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) {
      c = (c >>> 1) ^ (c & 1 ? 0xedb88320 : 0);
    }
  }
  return ~c >>> 0;
}
