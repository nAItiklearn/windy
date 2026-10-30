// Generates the 1024x1024 source icon (blue rounded square, three white bars) with no dependencies.
// Usage: node scripts/make-icon.mjs <output.png>
import { deflateSync } from "node:zlib";
import { writeFileSync } from "node:fs";

const N = 1024;
const out = process.argv[2] ?? "icon-source.png";

function crc32(buf) {
  let c, crc = 0xffffffff;
  for (let n = 0; n < buf.length; n++) {
    c = (crc ^ buf[n]) & 0xff;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    crc = (crc >>> 8) ^ c;
  }
  return (crc ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const td = Buffer.concat([Buffer.from(type), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(td));
  return Buffer.concat([len, td, crc]);
}

// Signed distance to a rounded rectangle centred at (cx, cy).
function rr(x, y, cx, cy, hw, hh, r) {
  const dx = Math.abs(x - cx) - (hw - r);
  const dy = Math.abs(y - cy) - (hh - r);
  return Math.hypot(Math.max(dx, 0), Math.max(dy, 0)) + Math.min(Math.max(dx, dy), 0) - r;
}
const cover = (d) => Math.max(0, Math.min(1, 0.5 - d));

const bars = [
  { cx: 470, cy: 340, hw: 270, hh: 46 },
  { cx: 560, cy: 512, hw: 330, hh: 46 },
  { cx: 410, cy: 684, hw: 210, hh: 46 },
];

const raw = Buffer.alloc(N * (N * 4 + 1));
for (let y = 0; y < N; y++) {
  raw[y * (N * 4 + 1)] = 0;
  for (let x = 0; x < N; x++) {
    const o = y * (N * 4 + 1) + 1 + x * 4;
    const bg = cover(rr(x + 0.5, y + 0.5, N / 2, N / 2, N / 2, N / 2, 230));
    let w = 0;
    for (const b of bars) w = Math.max(w, cover(rr(x + 0.5, y + 0.5, b.cx, b.cy, b.hw, b.hh, 46)));
    const r = 10 + (255 - 10) * w, g = 100 + (255 - 100) * w, bl = 214 + (255 - 214) * w;
    raw[o] = r; raw[o + 1] = g; raw[o + 2] = bl; raw[o + 3] = Math.round(255 * bg);
  }
}
const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(N, 0); ihdr.writeUInt32BE(N, 4); ihdr[8] = 8; ihdr[9] = 6;
writeFileSync(out, Buffer.concat([
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw, { level: 9 })), chunk("IEND", Buffer.alloc(0)),
]));
console.log("wrote", out);
