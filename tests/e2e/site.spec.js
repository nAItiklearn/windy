const { test, expect } = require("@playwright/test");

const SITE = "http://127.0.0.1:4174/";
const FILES = {
  windows: "Windy-windows-x64-setup.exe",
  macos: "Windy-macos-universal.dmg",
  linux: "Windy-linux-x64.AppImage",
};

test("download cards point at version-free release assets", async ({ page }) => {
  await page.goto(SITE);
  for (const [os, file] of Object.entries(FILES)) {
    await expect(page.locator(`#downloads [data-os="${os}"]`)).toHaveAttribute(
      "href",
      `https://github.com/nAItiklearn/windy/releases/latest/download/${file}`,
    );
  }
});

test("the primary button matches the visitor's OS", async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, "platform", { configurable: true, get: () => "Win32" });
    Object.defineProperty(navigator, "userAgentData", { configurable: true, get: () => ({ platform: "Windows" }) });
  });
  await page.goto(SITE);
  await expect(page.locator("#primary-download")).toHaveText("Download for Windows");
  await expect(page.locator("#primary-download")).toHaveAttribute("href", /Windy-windows-x64-setup\.exe$/);
});

test("the live demo searches and selects", async ({ page }) => {
  await page.goto(SITE);
  await page.locator("#demo-search").fill("meeting");
  await expect(page.locator("#demo-list li")).toHaveCount(1);
  await page.keyboard.press("Enter");
  await expect(page.locator("#demo-status")).toContainText("Copied");
});

test("page has the basics: title, FAQ, contact, no external scripts", async ({ page }) => {
  await page.goto(SITE);
  await expect(page).toHaveTitle(/Windy/);
  await expect(page.locator("details")).toHaveCount(6);
  await expect(page.locator("#contact a[href*='issues']")).toBeVisible();
  const external = await page.$$eval("script[src]", (s) => s.filter((x) => /^https?:/.test(x.getAttribute("src"))).length);
  expect(external).toBe(0);
});

test("the shortcuts graphic loads, is described, and reserves its space", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto(SITE);
  const img = page.locator(".callouts img");
  // The graphic is lazy-loaded below the fold, so bring it into view first.
  await img.scrollIntoViewIfNeeded();
  await expect(img).toBeVisible();
  await expect(img).toHaveAttribute("alt", /Alt\+P pins a row/);
  await expect(img).toHaveAttribute("width", "1200");
  await expect(img).toHaveAttribute("height", "630");
  await expect.poll(() => img.evaluate((el) => (el.complete ? el.naturalWidth : 0))).toBe(1200);
});

test("the shortcuts graphic is hidden on phones and the page does not scroll sideways", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(SITE);
  await expect(page.locator(".callouts")).toBeHidden();
  expect(await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth)).toBeLessThanOrEqual(0);
  // overflow:hidden on a section can mask clipping, so also check that key blocks fit on screen.
  const clipped = await page.$$eval(".hero-copy, #primary-download, .popup, .card, .cta-band", (els) =>
    els.filter((el) => el.getBoundingClientRect().right > window.innerWidth + 1).map((el) => el.className || el.id),
  );
  expect(clipped).toEqual([]);
});

test("no console errors on load", async ({ page }) => {
  const errors = [];
  page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
  page.on("pageerror", (e) => errors.push(String(e)));
  await page.goto(SITE);
  await page.waitForLoadState("networkidle");
  expect(errors).toEqual([]);
});
