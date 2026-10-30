// Builds the annotated feature graphic for the website hero and the social preview.
//   site/img/feature-callouts.svg   self-contained: the real popup screenshot is embedded as a data URI
//   site/img/social-preview.png     1200x630 raster of the same graphic (--png); social networks do not accept SVG
// Usage: node scripts/make-callouts.mjs [--png]      (--png needs a browser: set PW_CHANNEL=msedge to reuse Edge)
// Every shortcut below comes from ui/popup.js and the README; change those first if a shortcut changes.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outDir = resolve(root, "site/img");
const W = 1200;
const H = 630;

// Windy's dark palette and system font stack (site/style.css).
const C = { bg: "#0a0a0c", fg: "#f2f2f0", muted: "#a0a3a8", line: "#2c2c2e", card: "#1f1f21", accent: "#5aa2ff", violet: "#8b7cf6" };
const FONT = `system-ui, -apple-system, "Segoe UI", Roboto, sans-serif`;
const MONO = `ui-monospace, SFMono-Regular, Consolas, monospace`;

// The real popup capture, trimmed by `inset` px per side to drop the window-edge artefacts, then drawn at scale `s`.
const SHOT = { file: "docs/screenshots/popup-pinned.png", w: 581, h: 710, inset: 8, x: 380, y: 45, s: 0.78 };
const cw = SHOT.w - 2 * SHOT.inset;
const ch = SHOT.h - 2 * SHOT.inset;
// Raw screenshot coordinates -> graphic coordinates.
const at = (rx, ry) => [SHOT.x + SHOT.s * (rx - SHOT.inset), SHOT.y + SHOT.s * (ry - SHOT.inset)];
const r1 = (n) => Math.round(n * 10) / 10;

// Annotations. `from` is where the dotted line leaves the text, `to` the point it marks on the screenshot.
// Lines end on the edge of what they mark (an input edge, the selected row, or a dotted box), never on a glyph.
const callouts = [
  { x: 48, y: 26, label: "Type to search", lines: ["Focus starts in the search box.", "Exact, fuzzy, regex or mixed."], from: [330, 46], to: at(87, 18) },
  { x: 48, y: 126, label: "Open source", lines: ["MIT licensed.", "The code is on GitHub."], from: [300, 146], to: at(40, 48) },
  { x: 48, y: 226, keys: ["Enter"], label: "copies", lines: ["Alt+Enter pastes instead.", "Add Shift to drop formatting."], from: [300, 246], to: at(15, 95) },
  { x: 48, y: 486, label: "Clear all", lines: ["Click twice to confirm.", "Pinned rows go too."], from: [300, 500], to: at(376, 660) },
  { x: 862, y: 26, keys: ["Esc"], label: "to close", lines: ["Clears the search first,", "then closes the popup."], from: [852, 46], to: at(559, 30) },
  { x: 862, y: 126, keys: ["Alt", "P"], label: "pins a row", lines: ["Pinned rows stay at the top.", "Press it again to unpin."], from: [852, 146], to: at(562, 95) },
  { x: 862, y: 226, keys: ["Ctrl", "1…9"], label: "picks a row", lines: ["Rows 1 to 9 are numbered.", "Hold Ctrl, press the number."], from: [852, 246], to: at(564, 215) },
];

// Dotted highlight boxes around the things being pointed at, in raw screenshot coordinates.
const boxes = [
  { rx: 18, ry: 24, rw: 66, rh: 24 }, // the "Windy" wordmark
  { rx: 536, ry: 82, rw: 26, rh: 26 }, // the pin letter
  { rx: 536, ry: 120, rw: 28, rh: 346 }, // the number column
  { rx: 340, ry: 660, rw: 72, rh: 28 }, // "Clear all"
];

const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;");

function keycap(x, y, text) {
  const w = r1(text.length * 9.8 + 22);
  return {
    w,
    svg: `<rect x="${x}" y="${y + 3}" width="${w}" height="28" rx="7" class="kc"/><text x="${r1(x + w / 2)}" y="${y + 22}" text-anchor="middle" class="k">${esc(text)}</text>`,
  };
}

function block(c) {
  let cx = c.x;
  let out = "";
  (c.keys ?? []).forEach((k, i) => {
    if (i > 0) {
      out += `<text x="${cx + 2}" y="${c.y + 24}" class="plus">+</text>`;
      cx += 18;
    }
    const kc = keycap(cx, c.y, k);
    out += kc.svg;
    cx += kc.w + 4;
  });
  if (c.keys) cx += 8;
  out += `<text x="${cx}" y="${c.y + 25}" class="t">${esc(c.label)}</text>`;
  c.lines.forEach((line, i) => {
    out += `<text x="${c.x}" y="${c.y + 59 + i * 27}" class="d">${esc(line)}</text>`;
  });
  return out;
}

function connector(c) {
  const [x1, y1] = c.from;
  const [x2, y2] = c.to.map(r1);
  return `<path d="M${x1} ${y1} L${x2} ${y2}" class="dots"/><circle cx="${x2}" cy="${y2}" r="8" class="halo"/><circle cx="${x2}" cy="${y2}" r="4" fill="${C.accent}"/>`;
}

function build() {
  const png = readFileSync(resolve(root, SHOT.file)).toString("base64");
  const popW = r1(cw * SHOT.s);
  const popH = r1(ch * SHOT.s);
  const boxSvg = boxes
    .map((b) => {
      const [x, y] = at(b.rx, b.ry);
      return `<rect x="${r1(x)}" y="${r1(y)}" width="${r1(b.rw * SHOT.s)}" height="${r1(b.rh * SHOT.s)}" rx="6" class="dotbox"/>`;
    })
    .join("");

  return `<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" role="img" aria-labelledby="t d">
<title id="t">Windy's popup with its main shortcuts</title>
<desc id="d">Type to search. Enter copies the selected row and Alt+Enter pastes it. Esc clears the search and then closes the popup. Alt+P pins a row. Ctrl plus 1 to 9 picks one of the first nine rows. Clear all needs two clicks. Windy is open source and MIT licensed.</desc>
<style>
.t{font:600 26px ${FONT};fill:${C.fg}}
.d{font:400 19px ${FONT};fill:${C.muted}}
.k{font:600 16px ${MONO};fill:${C.fg}}
.kc{fill:${C.card};stroke:#3a3a3d;stroke-width:1.5}
.plus{font:500 20px ${FONT};fill:${C.muted}}
.dots{stroke:${C.accent};stroke-width:3;stroke-linecap:round;stroke-dasharray:0.1 8;fill:none}
.halo{fill:${C.accent};opacity:.25}
.dotbox{fill:none;stroke:${C.accent};stroke-width:2;stroke-linecap:round;stroke-dasharray:0.1 6;opacity:.9}
.brand{font:700 30px ${FONT};fill:${C.fg};letter-spacing:-.01em}
.tag{font:400 16px ${FONT};fill:${C.muted}}
</style>
<defs>
<radialGradient id="g1" cx="50%" cy="50%" r="50%"><stop offset="0" stop-color="${C.accent}" stop-opacity=".30"/><stop offset="1" stop-color="${C.accent}" stop-opacity="0"/></radialGradient>
<radialGradient id="g2" cx="50%" cy="50%" r="50%"><stop offset="0" stop-color="${C.violet}" stop-opacity=".22"/><stop offset="1" stop-color="${C.violet}" stop-opacity="0"/></radialGradient>
<linearGradient id="rib" x1="0" x2="1" y1="0" y2="0"><stop offset="0" stop-color="${C.accent}" stop-opacity="0"/><stop offset=".35" stop-color="${C.accent}"/><stop offset=".65" stop-color="${C.violet}"/><stop offset="1" stop-color="${C.violet}" stop-opacity="0"/></linearGradient>
<filter id="soft" x="-20%" y="-60%" width="140%" height="220%"><feGaussianBlur stdDeviation="26"/></filter>
<filter id="shadow" x="-20%" y="-10%" width="140%" height="130%"><feDropShadow dx="0" dy="14" stdDeviation="18" flood-color="#000" flood-opacity=".65"/></filter>
<clipPath id="shot"><rect width="${cw}" height="${ch}" rx="14"/></clipPath>
</defs>
<rect width="${W}" height="${H}" fill="${C.bg}"/>
<ellipse cx="560" cy="300" rx="520" ry="330" fill="url(#g1)"/>
<ellipse cx="760" cy="420" rx="420" ry="260" fill="url(#g2)"/>
<path d="M-40 470 C 200 380, 340 560, 620 460 S 1000 320, 1240 410" stroke="url(#rib)" stroke-width="70" fill="none" opacity=".30" filter="url(#soft)"/>
<rect x="${SHOT.x}" y="${SHOT.y}" width="${popW}" height="${popH}" rx="${r1(14 * SHOT.s)}" fill="#000" filter="url(#shadow)"/>
<g transform="translate(${SHOT.x} ${SHOT.y}) scale(${SHOT.s})"><g clip-path="url(#shot)"><image x="${-SHOT.inset}" y="${-SHOT.inset}" width="${SHOT.w}" height="${SHOT.h}" href="data:image/png;base64,${png}" xlink:href="data:image/png;base64,${png}"/></g></g>
<rect x="${SHOT.x}" y="${SHOT.y}" width="${popW}" height="${popH}" rx="${r1(14 * SHOT.s)}" fill="none" stroke="#ffffff" stroke-opacity=".14" stroke-width="1"/>
${boxSvg}
${callouts.map(connector).join("\n")}
${callouts.map(block).join("\n")}
<g transform="translate(862 540)"><rect width="46" height="46" rx="11" fill="#0a64d6"/><rect x="9" y="11" width="22" height="5" rx="2.5" fill="#fff"/><rect x="9" y="20.5" width="28" height="5" rx="2.5" fill="#fff"/><rect x="9" y="30" width="15" height="5" rx="2.5" fill="#fff"/><text x="60" y="33" class="brand">Windy</text><text x="0" y="76" class="tag">A calm clipboard history.</text></g>
</svg>
`;
}

mkdirSync(outDir, { recursive: true });
const svgPath = resolve(outDir, "feature-callouts.svg");
writeFileSync(svgPath, build());
console.log(`wrote ${svgPath}`);

if (process.argv.includes("--png")) {
  const { default: pw } = await import("@playwright/test");
  const browser = await pw.chromium.launch({ channel: process.env.PW_CHANNEL || undefined });
  const page = await browser.newPage({ viewport: { width: W, height: H }, deviceScaleFactor: 1 });
  await page.goto(`file:///${svgPath.replace(/\\/g, "/")}`);
  const pngPath = resolve(outDir, "social-preview.png");
  await page.screenshot({ path: pngPath, clip: { x: 0, y: 0, width: W, height: H } });
  await browser.close();
  console.log(`wrote ${pngPath}`);
}
