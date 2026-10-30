import * as api from "./api.js";

const ROW = 32;
const $ = (id) => document.getElementById(id);
const q = $("q");
const viewport = $("viewport");
const spacer = $("spacer");
const status = $("status");

let rows = [];
let sel = 0;
let settings = null;
const images = new Map();

function modifierFor(e) {
  const primary = e.ctrlKey || e.metaKey;
  if (e.altKey && e.shiftKey) return "altShift";
  if (e.altKey) return "alt";
  if (primary && e.shiftKey) return "primaryShift";
  if (primary) return "primary";
  return "none";
}

function titleNode(row, query) {
  const span = document.createElement("span");
  span.className = "t";
  const text = row.title || (row.kind === "image" ? "Image" : "");
  const at = query && settings && settings.searchMode === "exact" ? text.toLowerCase().indexOf(query.toLowerCase()) : -1;
  if (at < 0) {
    span.textContent = text;
  } else {
    const m = document.createElement("mark");
    m.textContent = text.slice(at, at + query.length);
    span.append(text.slice(0, at), m, text.slice(at + query.length));
  }
  return span;
}

async function fillImage(img, id) {
  if (!images.has(id)) images.set(id, api.getImage(id));
  const url = await images.get(id);
  if (url) img.src = url;
}

function render() {
  const top = viewport.scrollTop;
  const first = Math.max(0, Math.floor(top / ROW) - 3);
  const last = Math.min(rows.length, Math.ceil((top + viewport.clientHeight) / ROW) + 3);
  spacer.style.height = `${Math.max(rows.length * ROW, 0)}px`;
  spacer.replaceChildren();
  if (!rows.length) {
    const d = document.createElement("div");
    d.className = "empty";
    d.textContent = q.value ? "Nothing found" : "Copy something and it will show up here";
    spacer.append(d);
    return;
  }
  // Number hints are for the first nine unpinned rows, counted from the top.
  const hints = new Map();
  let n = 0;
  rows.forEach((r, i) => { if (!r.pin && n < 9) hints.set(i, String(++n)); });
  for (let i = first; i < last; i++) {
    const r = rows[i];
    const el = document.createElement("div");
    el.className = "row";
    el.id = `row-${r.id}`;
    el.setAttribute("role", "option");
    el.setAttribute("aria-selected", String(i === sel));
    el.setAttribute("aria-label", `${r.title || r.kind}${r.pin ? ", pinned" : ""}${r.application ? `, from ${r.application}` : ""}`);
    el.style.top = `${i * ROW}px`;
    if (r.kind === "image") {
      const img = document.createElement("img");
      img.alt = "";
      el.append(img);
      fillImage(img, r.id);
    }
    el.append(titleNode(r, q.value));
    const h = document.createElement("span");
    h.className = "h";
    h.textContent = r.pin ? r.pin : hints.get(i) || "";
    el.append(h);
    el.addEventListener("click", (e) => choose(i, modifierFor(e)));
    spacer.append(el);
  }
  if (rows[sel]) q.setAttribute("aria-activedescendant", `row-${rows[sel].id}`);
}

function ensureVisible() {
  const top = sel * ROW;
  if (top < viewport.scrollTop) viewport.scrollTop = top;
  else if (top + ROW > viewport.scrollTop + viewport.clientHeight) viewport.scrollTop = top + ROW - viewport.clientHeight;
}

let seq = 0;
async function refresh(keepSel = false) {
  const prev = rows[sel] && rows[sel].id;
  const mine = ++seq;
  const result = await api.listItems(q.value);
  if (mine !== seq) return; // a newer request superseded this one
  rows = result;
  if (keepSel && prev !== undefined) {
    const at = rows.findIndex((r) => r.id === prev);
    sel = at >= 0 ? at : Math.min(sel, Math.max(rows.length - 1, 0));
  } else {
    // Empty query selects the first unpinned item; a query highlights the first result.
    const firstFree = rows.findIndex((r) => !r.pin);
    sel = q.value || firstFree < 0 ? 0 : firstFree;
  }
  render();
  ensureVisible();
  render();
}

function say(text) {
  status.textContent = text;
  clearTimeout(say.t);
  say.t = setTimeout(() => (status.textContent = ""), 2500);
}

async function choose(i, modifier) {
  const r = rows[i];
  if (!r) return;
  await api.selectItem(r.id, modifier);
  q.value = "";
  if (api.isDemo) await refresh();
}

function move(to) {
  if (!rows.length) return;
  sel = Math.max(0, Math.min(to, rows.length - 1));
  render();
  ensureVisible();
  render();
}

async function removeSelected() {
  const r = rows[sel];
  if (!r) return;
  await api.deleteItem(r.id);
  await refresh(true);
}

q.addEventListener("input", () => refresh());
viewport.addEventListener("scroll", render, { passive: true });

document.addEventListener("keydown", async (e) => {
  const primary = e.ctrlKey || e.metaKey;
  const k = e.key;
  if (k === "Escape") { e.preventDefault(); if (q.value) { q.value = ""; refresh(); } else api.hidePopup(); }
  else if (k === "ArrowDown" || (e.ctrlKey && (k === "n" || k === "j"))) { e.preventDefault(); move(sel + 1); }
  else if (k === "ArrowUp" || (e.ctrlKey && (k === "p" || k === "k"))) { e.preventDefault(); move(sel - 1); }
  else if (k === "PageDown") { e.preventDefault(); move(sel + 8); }
  else if (k === "PageUp") { e.preventDefault(); move(sel - 8); }
  else if (primary && k === "ArrowDown") { e.preventDefault(); move(rows.length - 1); }
  else if (primary && k === "ArrowUp") { e.preventDefault(); move(0); }
  else if (k === "Enter") { e.preventDefault(); if (rows[sel]) choose(sel, modifierFor(e)); }
  else if (e.altKey && k.toLowerCase() === "p") { e.preventDefault(); if (rows[sel]) { await api.togglePin(rows[sel].id); await refresh(true); } }
  else if (e.altKey && k === "Backspace") { e.preventDefault(); removeSelected(); }
  else if (e.altKey && primary && k === "Backspace") { e.preventDefault(); onClear(e.shiftKey); }
  else if (primary && k === ",") { e.preventDefault(); api.openSettings(); }
  else if (primary && /^[1-9]$/.test(k)) {
    e.preventDefault();
    const target = rows.filter((r) => !r.pin)[Number(k) - 1];
    if (target) choose(rows.indexOf(target), "primary");
  } else if (e.ctrlKey && k === "u") { e.preventDefault(); q.value = ""; refresh(); }
  else if (e.ctrlKey && k === "w") {
    e.preventDefault();
    q.value = q.value.replace(/\s*\S+\s*$/, "");
    refresh();
  }
});

let armed = null;
function onClear(all) {
  const which = all ? "all" : "unpinned";
  if (armed !== which) {
    armed = which;
    say(all ? "Click again to clear everything, including pins" : "Click again to clear unpinned items");
    clearTimeout(onClear.t);
    onClear.t = setTimeout(() => (armed = null), 3000);
    return;
  }
  armed = null;
  api.clearHistory(all).then(() => refresh());
}
$("clear").addEventListener("click", () => onClear(false));
$("clear-all").addEventListener("click", () => onClear(true));
$("open-settings").addEventListener("click", () => api.openSettings());
$("quit").addEventListener("click", () => api.quitApp());

api.onChanged(() => refresh(true));
api.onShown(() => { q.value = ""; refresh(); q.focus(); });

api.getSettings().then((s) => {
  settings = s;
  $("foot").hidden = s.showFooter === false;
  return refresh();
});
q.focus();
