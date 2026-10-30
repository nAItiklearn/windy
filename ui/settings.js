import * as api from "./api.js";

const root = document.getElementById("root");

const SECTIONS = [
  ["General", [
    { key: "popupHotkey", type: "hotkey", label: "Open Windy with", help: "Press the keys you want. At least one modifier." },
    { key: "searchMode", type: "select", label: "Search", options: [["exact", "Exact"], ["fuzzy", "Fuzzy"], ["regexp", "Regular expression"], ["mixed", "Mixed"]] },
    { key: "pasteByDefault", type: "bool", label: "Paste automatically", help: "Enter pastes instead of only copying. Alt+Enter does the other one." },
    { key: "removeFormattingByDefault", type: "bool", label: "Paste without formatting" },
  ]],
  ["Storage", [
    { key: "saveText", type: "bool", label: "Save text" },
    { key: "saveImages", type: "bool", label: "Save images" },
    { key: "saveFiles", type: "bool", label: "Save files" },
    { key: "historySize", type: "number", label: "History size", min: 1, max: 999 },
    { key: "sortBy", type: "select", label: "Sort by", options: [["lastCopiedAt", "Last copied"], ["firstCopiedAt", "First copied"], ["numberOfCopies", "Copies"]] },
  ]],
  ["Appearance", [
    { key: "popupPosition", type: "select", label: "Open at", options: [["cursor", "Mouse cursor"], ["center", "Screen centre"]] },
    { key: "pinTo", type: "select", label: "Pin to", options: [["top", "Top"], ["bottom", "Bottom"]] },
    { key: "showSpecialSymbols", type: "bool", label: "Show special symbols", help: "Marks leading spaces, tabs and line breaks in titles." },
    { key: "showFooter", type: "bool", label: "Show footer" },
  ]],
  ["Ignore", [
    { key: "ignoredApps", type: "lines", label: "Ignored applications", help: "One per line, e.g. keepass.exe" },
    { key: "ignoreAllAppsExceptListed", type: "bool", label: "Ignore all except listed" },
    { key: "ignoreRegexp", type: "lines", label: "Ignore copies matching", help: "One regular expression per line" },
    { key: "ignoredPasteboardTypes", type: "lines", label: "Ignored clipboard types" },
  ]],
  ["Advanced", [
    { key: "ignoreEvents", type: "bool", label: "Pause recording" },
    { key: "clearOnQuit", type: "bool", label: "Clear history on quit" },
    { key: "clearSystemClipboard", type: "bool", label: "Also clear the system clipboard when clearing history" },
  ]],
];

function accelerator(e) {
  const mods = [];
  if (e.ctrlKey) mods.push("Control");
  if (e.metaKey) mods.push("Super");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  const k = e.key;
  if (["Control", "Alt", "Shift", "Meta"].includes(k)) return null;
  let key = k.length === 1 ? k.toUpperCase() : k;
  if (key === " ") key = "Space";
  key = key.replace(/^Arrow/, "");
  if (!mods.length) return null;
  return [...mods, key].join("+");
}

function build(settings) {
  const save = () => api.setSettings(settings);
  for (const [title, fields] of SECTIONS) {
    const h = document.createElement("h2");
    h.textContent = title;
    root.append(h);
    for (const f of fields) {
      const row = document.createElement("div");
      row.className = "field";
      const label = document.createElement("label");
      label.textContent = f.label;
      const id = `f-${f.key}`;
      label.htmlFor = id;
      if (f.help) {
        const s = document.createElement("small");
        s.textContent = f.help;
        label.append(s);
      }
      let input;
      if (f.type === "bool") {
        input = document.createElement("input");
        input.type = "checkbox";
        input.checked = !!settings[f.key];
        input.addEventListener("change", () => { settings[f.key] = input.checked; save(); });
      } else if (f.type === "number") {
        input = document.createElement("input");
        input.type = "number";
        input.min = f.min; input.max = f.max;
        input.value = settings[f.key];
        input.addEventListener("change", () => {
          settings[f.key] = Math.max(f.min, Math.min(f.max, Number(input.value) || f.min));
          input.value = settings[f.key];
          save();
        });
      } else if (f.type === "select") {
        input = document.createElement("select");
        for (const [v, text] of f.options) {
          const o = document.createElement("option");
          o.value = v; o.textContent = text;
          input.append(o);
        }
        input.value = settings[f.key];
        input.addEventListener("change", () => { settings[f.key] = input.value; save(); });
      } else if (f.type === "lines") {
        input = document.createElement("textarea");
        input.spellcheck = false;
        input.value = (settings[f.key] || []).join("\n");
        input.addEventListener("change", () => {
          settings[f.key] = input.value.split("\n").map((s) => s.trim()).filter(Boolean);
          save();
        });
      } else if (f.type === "hotkey") {
        input = document.createElement("input");
        input.type = "text";
        input.readOnly = true;
        input.value = settings[f.key];
        input.addEventListener("keydown", (e) => {
          if (e.key === "Tab") return;
          e.preventDefault();
          const a = accelerator(e);
          if (a) { settings[f.key] = a; input.value = a; save(); }
        });
      }
      input.id = id;
      row.append(label, input);
      root.append(row);
    }
  }
  const about = document.createElement("p");
  about.className = "about";
  api.getInfo().then((i) => (about.textContent = `Windy ${i.version}. MIT licensed. Inspired by Maccy.`));
  root.append(about);
}

api.getSettings().then(build);
