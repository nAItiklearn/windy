// The only module that talks to the backend. Outside Tauri (browser, Playwright) it falls back
// to an in-memory demo so the same UI runs with realistic fake data: ?n=10000 for a big history.

const tauri = window.__TAURI__;

const SAMPLE = [
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

const demo = (() => {
  const n = Number(new URLSearchParams(location.search).get("n")) || SAMPLE.length;
  let nextId = 1;
  const items = Array.from({ length: n }, (_, i) => ({
    id: nextId++,
    title: i < SAMPLE.length ? SAMPLE[i] : `${SAMPLE[i % SAMPLE.length]} (${i})`,
    pin: null,
    application: i % 3 ? "code.exe" : "chrome.exe",
    copies: 1 + (i % 4),
    kind: i === 3 ? "text" : "text",
  }));
  let settings = {
    historySize: 200, sortBy: "lastCopiedAt", pinTo: "top", searchMode: "exact",
    pasteByDefault: false, removeFormattingByDefault: false, saveFiles: true, saveImages: true,
    saveText: true, ignoredApps: [], ignoreRegexp: [], ignoreEvents: false, clearOnQuit: false,
    clearSystemClipboard: false, popupPosition: "cursor", popupHotkey: "CommandOrControl+Alt+V",
    showFooter: true, showSpecialSymbols: true,
  };
  const listeners = new Set();
  const changed = () => listeners.forEach((f) => f());
  const pins = "bcdefghijklmnorstuxy";
  const api = {
    list_items: ({ query }) => {
      const q = query.toLowerCase();
      const rows = items.filter((i) => i.title.toLowerCase().includes(q));
      return [...rows.filter((i) => i.pin), ...rows.filter((i) => !i.pin)];
    },
    get_image: () => null,
    select_item: ({ id, modifier }) => {
      const i = items.findIndex((x) => x.id === id);
      if (i < 0) return;
      const [it] = items.splice(i, 1);
      it.copies += 1;
      items.unshift(it);
      window.__lastSelected = { id, modifier, title: it.title };
      changed();
    },
    toggle_pin: ({ id }) => {
      const it = items.find((x) => x.id === id);
      if (it) it.pin = it.pin ? null : pins[id % pins.length];
      changed();
    },
    delete_item: ({ id }) => {
      const i = items.findIndex((x) => x.id === id);
      if (i >= 0) items.splice(i, 1);
      changed();
    },
    clear_history: ({ all }) => {
      for (let i = items.length - 1; i >= 0; i--) if (all || !items[i].pin) items.splice(i, 1);
      changed();
    },
    get_settings: () => structuredClone(settings),
    set_settings: ({ settings: s }) => { settings = structuredClone(s); window.__settings = settings; changed(); },
    get_info: () => ({ version: "demo", os: "demo" }),
    hide_popup: () => { window.__hidden = true; },
    open_settings_window: () => { window.__openedSettings = true; },
    quit_app: () => {},
  };
  return { api, listeners };
})();

async function call(cmd, args = {}) {
  if (tauri) return tauri.core.invoke(cmd, args);
  return demo.api[cmd](args);
}

export const listItems = (query) => call("list_items", { query });
export const getImage = (id) => call("get_image", { id });
export const selectItem = (id, modifier) => call("select_item", { id, modifier });
export const togglePin = (id) => call("toggle_pin", { id });
export const deleteItem = (id) => call("delete_item", { id });
export const clearHistory = (all) => call("clear_history", { all });
export const getSettings = () => call("get_settings");
export const setSettings = (settings) => call("set_settings", { settings });
export const getInfo = () => call("get_info");
export const hidePopup = () => call("hide_popup");
export const openSettings = () => call("open_settings_window");
export const quitApp = () => call("quit_app");
export const isDemo = !tauri;

function on(event, cb) {
  if (tauri) tauri.event.listen(event, cb);
  else if (event === "history-changed") demo.listeners.add(cb);
}
export const onChanged = (cb) => on("history-changed", cb);
export const onShown = (cb) => on("popup-shown", cb);
