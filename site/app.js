"use strict";

// ---- Download buttons ----------------------------------------------------
const FALLBACK = {
  repo: "nAItiklearn/windy",
  assets: {
    windows: { label: "Windows", file: "Windy-windows-x64-setup.exe" },
    macos: { label: "macOS", file: "Windy-macos-universal.dmg" },
    linux: { label: "Linux", file: "Windy-linux-x64.AppImage" },
  },
};

function detectOS() {
  const p = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
  const ua = navigator.userAgent || "";
  const s = (p + " " + ua).toLowerCase();
  if (/android|iphone|ipad/.test(s)) return null;
  if (s.includes("win")) return "windows";
  if (s.includes("mac")) return "macos";
  if (s.includes("linux") || s.includes("x11") || s.includes("cros")) return "linux";
  return null;
}

async function loadConfig() {
  try {
    const r = await fetch("downloads.json", { cache: "no-cache" });
    if (r.ok) return await r.json();
  } catch (_) { /* use fallback */ }
  return FALLBACK;
}

async function initDownloads() {
  const cfg = await loadConfig();
  const base = "https://github.com/" + cfg.repo + "/releases/latest/download/";
  const os = detectOS();
  document.querySelectorAll("#downloads .card").forEach((card) => {
    const a = cfg.assets[card.dataset.os];
    if (a) card.href = base + a.file;
    if (card.dataset.os === os) card.classList.add("match");
  });
  const btn = document.getElementById("primary-download");
  const note = document.getElementById("primary-note");
  const detected = document.getElementById("dl-detected");
  if (os && cfg.assets[os]) {
    btn.href = base + cfg.assets[os].file;
    btn.textContent = "Download for " + cfg.assets[os].label;
    note.textContent = "Free and open source";
    detected.textContent = "We think you are on " + cfg.assets[os].label + ". Other systems below.";
  } else {
    detected.textContent = "Pick your system.";
  }
}

// ---- Popup demo ----------------------------------------------------------
const ITEMS = [
  "https://github.com/nAItiklearn/windy",
  "meeting notes: ship the installer on Friday",
  "SELECT id, title FROM history ORDER BY last_copied_at DESC;",
  "#0a64d6",
  "Dear team, thanks for the quick turnaround on this.",
  "C:\\Users\\you\\Documents\\report-final-v2.pdf",
  "npm run build && npm test",
  "+1 (555) 010-0199",
  "Remember to water the plants",
  "ssh deploy@example.com -p 2222",
];

const list = document.getElementById("demo-list");
const input = document.getElementById("demo-search");
const status = document.getElementById("demo-status");
const state = { items: ITEMS.map((t, i) => ({ id: i, text: t, pin: null })), sel: 0, query: "" };

function visible() {
  const q = state.query.toLowerCase();
  const rows = state.items.filter((i) => i.text.toLowerCase().includes(q));
  return [...rows.filter((i) => i.pin), ...rows.filter((i) => !i.pin)];
}

function highlight(li, text, q) {
  const span = document.createElement("span");
  span.className = "t";
  const at = q ? text.toLowerCase().indexOf(q.toLowerCase()) : -1;
  if (at < 0) {
    span.textContent = text;
  } else {
    span.append(text.slice(0, at));
    const m = document.createElement("mark");
    m.textContent = text.slice(at, at + q.length);
    span.append(m, text.slice(at + q.length));
  }
  li.append(span);
}

function render() {
  const rows = visible();
  state.sel = Math.max(0, Math.min(state.sel, rows.length - 1));
  list.replaceChildren();
  if (!rows.length) {
    const li = document.createElement("li");
    li.className = "empty";
    li.textContent = "Nothing found";
    list.append(li);
    return;
  }
  let n = 0;
  rows.forEach((item, idx) => {
    const li = document.createElement("li");
    li.id = "demo-" + item.id;
    li.setAttribute("role", "option");
    li.setAttribute("aria-selected", String(idx === state.sel));
    highlight(li, item.text, state.query);
    const hint = document.createElement("span");
    hint.className = "h";
    hint.textContent = item.pin ? "📌" : ++n <= 9 ? String(n) : "";
    li.append(hint);
    li.addEventListener("click", () => choose(idx));
    list.append(li);
  });
  input.setAttribute("aria-activedescendant", "demo-" + rows[state.sel].id);
  const el = list.children[state.sel];
  if (el && state.sel > 0) {
    // Scroll only the list itself, never the page.
    if (el.offsetTop < list.scrollTop) list.scrollTop = el.offsetTop;
    else if (el.offsetTop + el.offsetHeight > list.scrollTop + list.clientHeight)
      list.scrollTop = el.offsetTop + el.offsetHeight - list.clientHeight;
  } else {
    list.scrollTop = 0;
  }
}

function choose(idx) {
  const rows = visible();
  const item = rows[idx];
  if (!item) return;
  status.textContent = "Copied: " + (item.text.length > 28 ? item.text.slice(0, 28) + "…" : item.text);
  if (navigator.clipboard && navigator.clipboard.writeText) navigator.clipboard.writeText(item.text).catch(() => {});
  state.items = [item, ...state.items.filter((i) => i !== item)];
  state.query = "";
  input.value = "";
  state.sel = 0;
  render();
}

input.addEventListener("input", () => { state.query = input.value; state.sel = 0; render(); });
input.addEventListener("keydown", (e) => {
  const rows = visible();
  if (e.key === "ArrowDown") { state.sel = Math.min(state.sel + 1, rows.length - 1); e.preventDefault(); render(); }
  else if (e.key === "ArrowUp") { state.sel = Math.max(state.sel - 1, 0); e.preventDefault(); render(); }
  else if (e.key === "Enter") { e.preventDefault(); choose(state.sel); }
  else if (e.altKey && e.key.toLowerCase() === "p") {
    e.preventDefault();
    const item = rows[state.sel];
    if (item) { item.pin = item.pin ? null : "b"; status.textContent = item.pin ? "Pinned" : "Unpinned"; render(); }
  } else if (e.key === "Escape") { state.query = ""; input.value = ""; render(); }
});

render();
initDownloads();
