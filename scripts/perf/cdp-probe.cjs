// Inspects the real app's WebView2 pages over the local debugging port (9333). Run by windy-perf.ps1 -Mode debug-probe.
// Prints each page's URL, readiness, whether window.__TAURI__ exists, and any console/page errors seen on a reload of the
// settings page. Never calls browser.close(): that could shut down the app's WebView2 browser process.
const { chromium } = require("playwright-core");

(async () => {
  const browser = await chromium.connectOverCDP("http://127.0.0.1:9333");
  const report = [];
  for (const ctx of browser.contexts()) {
    for (const page of ctx.pages()) {
      const entry = { url: page.url() };
      try { entry.title = await page.title(); } catch (e) { entry.titleError = String(e.message).slice(0, 160); }
      try {
        entry.state = await page.evaluate(() => ({
          readyState: document.readyState,
          tauri: typeof window.__TAURI__,
          bodyText: document.body ? document.body.innerText.slice(0, 120) : null,
          scripts: [...document.scripts].map((s) => s.src),
          sheets: [...document.styleSheets].map((s) => s.href),
        }));
      } catch (e) { entry.evalError = String(e.message).slice(0, 200); }

      if (/(settings|popup)\.html/.test(entry.url)) {
        // Chromium's Log and Network domains carry the URL that a plain console message lacks, so a
        // "Failed to load resource" line can be traced to the request that caused it.
        const messages = [];
        const urls = new Map();
        const client = await ctx.newCDPSession(page);
        await client.send("Network.enable");
        await client.send("Log.enable");
        client.on("Log.entryAdded", ({ entry: e }) => messages.push(`log.${e.source}.${e.level}: ${e.text}${e.url ? ` [${e.url}]` : ""}`.slice(0, 300)));
        client.on("Network.requestWillBeSent", (p) => urls.set(p.requestId, `${p.request.method} ${p.request.url}`));
        client.on("Network.responseReceived", (p) => { if (p.response.status >= 400) messages.push(`network ${p.response.status} (${p.type}): ${p.response.url}`); });
        client.on("Network.loadingFailed", (p) => messages.push(`network failed (${p.type}): ${p.errorText} ${urls.get(p.requestId) || ""}`));
        page.on("console", (m) => { const l = m.location(); messages.push(`console.${m.type()}: ${m.text()}${l && l.url ? ` [${l.url}:${l.lineNumber}]` : ""}`.slice(0, 300)); });
        page.on("pageerror", (e) => messages.push(`pageerror: ${String(e.message).slice(0, 240)}`));
        try {
          entry.invokeGetSettings = await page.evaluate(async () => {
            try { const s = await window.__TAURI__.core.invoke("get_settings"); return "ok, keys=" + Object.keys(s).length; }
            catch (e) { return "FAILED: " + String(e); }
          });
        } catch (e) { entry.invokeError = String(e.message).slice(0, 200); }
        try { await page.reload({ waitUntil: "load", timeout: 8000 }); } catch (e) { messages.push("reload: " + String(e.message).slice(0, 160)); }
        await new Promise((r) => setTimeout(r, 2500));
        try { entry.afterReloadBodyText = await page.evaluate(() => document.body.innerText.slice(0, 200)); } catch (e) { entry.afterReloadError = String(e.message).slice(0, 160); }
        entry.requestsSeen = [...urls.values()].slice(0, 40);
        entry.events = messages;
      }
      report.push(entry);
    }
  }
  console.log(JSON.stringify(report, null, 2));
  process.exit(0);
})().catch((e) => { console.error("probe failed:", e.message); process.exit(1); });
