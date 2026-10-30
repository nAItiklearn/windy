// Tests run against the UI in demo mode (no Tauri) and the static website.
// Locally set PW_CHANNEL=msedge to reuse an installed browser; CI uses the bundled Chromium.
const { defineConfig } = require("@playwright/test");

module.exports = defineConfig({
  testDir: "tests/e2e",
  fullyParallel: true,
  reporter: [["list"]],
  use: { channel: process.env.PW_CHANNEL || undefined, trace: "retain-on-failure" },
  webServer: [
    { command: "node scripts/serve.mjs ui 4173", url: "http://127.0.0.1:4173/popup.html", reuseExistingServer: true },
    { command: "node scripts/serve.mjs site 4174", url: "http://127.0.0.1:4174/", reuseExistingServer: true },
  ],
});
