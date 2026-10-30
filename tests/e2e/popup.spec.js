const { test, expect } = require("@playwright/test");

const POPUP = "http://127.0.0.1:4173/popup.html";
const rows = (page) => page.locator(".row");

async function open(page, query = "") {
  await page.goto(POPUP + query);
  await expect(rows(page).first()).toBeVisible();
}

test("lists items with number hints on the first nine", async ({ page }) => {
  await open(page);
  await expect(rows(page).first()).toBeVisible();
  await expect(rows(page).first().locator(".h")).toHaveText("1");
  await expect(rows(page).nth(8).locator(".h")).toHaveText("9");
});

test("search filters and highlights the match", async ({ page }) => {
  await open(page);
  await page.locator("#q").fill("ssh");
  await expect(rows(page)).toHaveCount(1);
  await expect(rows(page).first().locator("mark")).toHaveText("ssh");
  await page.locator("#q").fill("zzzz-no-match");
  await expect(page.locator(".empty")).toHaveText("Nothing found");
});

test("arrow keys move the selection and Enter selects", async ({ page }) => {
  await open(page);
  await page.locator("#q").focus();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowDown");
  await expect(rows(page).nth(2)).toHaveAttribute("aria-selected", "true");
  await page.keyboard.press("Enter");
  const sel = await page.evaluate(() => window.__lastSelected);
  expect(sel.title).toBe("SELECT id, title FROM history ORDER BY last_copied_at DESC;");
  expect(sel.modifier).toBe("none");
});

test("modifiers are passed through", async ({ page }) => {
  await open(page);
  await page.locator("#q").focus();
  await page.keyboard.press("Alt+Enter");
  expect((await page.evaluate(() => window.__lastSelected)).modifier).toBe("alt");
  await page.keyboard.press("Alt+Shift+Enter");
  expect((await page.evaluate(() => window.__lastSelected)).modifier).toBe("altShift");
});

test("Alt+P pins the selected item to the top", async ({ page }) => {
  await open(page);
  await page.locator("#q").focus();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Alt+p");
  await expect(rows(page).first().locator(".h")).not.toHaveText("1");
  await expect(rows(page).first()).toContainText("SELECT id");
});

test("Alt+Backspace deletes the selected item", async ({ page }) => {
  await open(page);
  await page.locator("#q").focus();
  await page.keyboard.press("Alt+Backspace");
  await expect(rows(page).first()).not.toContainText("github.com");
});

test("Clear needs a second click", async ({ page }) => {
  await open(page);
  await page.locator("#clear-all").click();
  await expect(page.locator("#status")).toContainText("Click again");
  await expect(rows(page).first()).toBeVisible();
  await page.locator("#clear-all").click();
  await expect(page.locator(".empty")).toBeVisible();
});

test("Escape clears the query, then closes", async ({ page }) => {
  await open(page);
  await page.locator("#q").fill("npm");
  await page.keyboard.press("Escape");
  await expect(page.locator("#q")).toHaveValue("");
  await page.keyboard.press("Escape");
  expect(await page.evaluate(() => window.__hidden)).toBe(true);
});

test("a 10,000 item history stays virtualised and fast", async ({ page }) => {
  await page.goto(`${POPUP}?n=10000`);
  await expect(rows(page).first()).toBeVisible();
  expect(await rows(page).count()).toBeLessThan(40);
  const ms = await page.evaluate(async () => {
    const q = document.getElementById("q");
    const t0 = performance.now();
    q.value = "ssh deploy";
    q.dispatchEvent(new Event("input"));
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    return performance.now() - t0;
  });
  expect(ms).toBeLessThan(250);
});

test("settings page saves changes", async ({ page }) => {
  await page.goto("http://127.0.0.1:4173/settings.html");
  await page.getByLabel("History size").fill("50");
  await page.getByLabel("History size").blur();
  await page.getByLabel("Pause recording").check();
  const s = await page.evaluate(() => window.__settings);
  expect(s.historySize).toBe(50);
  expect(s.ignoreEvents).toBe(true);
});
