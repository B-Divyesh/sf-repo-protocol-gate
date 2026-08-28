import { AxeBuilder } from "@axe-core/playwright";
import { chromium } from "playwright";
import { spawn } from "node:child_process";

const port = 4179;
const baseURL = `http://127.0.0.1:${port}`;
const server = spawn(
  process.platform === "win32" ? "node_modules/.bin/vite.cmd" : "node_modules/.bin/vite",
  ["preview", "--config", "site/vite.config.ts", "--host", "127.0.0.1", "--port", String(port)],
  { stdio: ["ignore", "pipe", "pipe"] },
);

async function waitForServer() {
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      const response = await fetch(baseURL);
      if (response.ok) return;
    } catch {
      // The preview server is still starting.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("Vite preview did not start");
}

let browser;
try {
  await waitForServer();
  browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: 1,
    reducedMotion: "reduce",
  });
  const page = await context.newPage();
  const consoleErrors = [];
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });

  await page.goto(baseURL, { waitUntil: "networkidle" });
  if ((await page.title()) !== "Repo Protocol Gate — make repository rules enforceable") {
    throw new Error("Unexpected document title");
  }
  if ((await page.locator("h1").count()) !== 1 || (await page.locator("main").count()) !== 1) {
    throw new Error("Expected exactly one h1 and one main landmark");
  }
  if (!(await page.locator(".hero-scene img").getAttribute("alt"))) {
    throw new Error("Hero image is missing meaningful alt text");
  }

  await page.getByRole("button", { name: "Blocked README" }).focus();
  await page.keyboard.press("Enter");
  if ((await page.locator("#verdict-title").textContent()) !== "Change denied") {
    throw new Error("README preset should be denied");
  }
  await page.getByRole("button", { name: "Valid migration" }).focus();
  await page.keyboard.press("Space");
  if ((await page.locator("#verdict-title").textContent()) !== "Change allowed") {
    throw new Error("Valid generated migration should be allowed");
  }
  await page.locator("#changes").fill("");
  await page.getByRole("button", { name: "Inspect changes" }).click();
  if ((await page.locator("#verdict-title").textContent()) !== "Nothing to inspect") {
    throw new Error("Empty diff state is missing");
  }
  await page.locator("#changes").fill("README.md");
  await page.getByRole("button", { name: "Inspect changes" }).click();
  if ((await page.locator("#verdict-title").textContent()) !== "Input needs attention") {
    throw new Error("Malformed diff should show a helpful error");
  }

  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload({ waitUntil: "networkidle" });
  await context.setOffline(true);
  await page.reload({ waitUntil: "domcontentloaded" });
  if (!(await page.locator("#network-status").isVisible())) {
    throw new Error("Offline state should be visible");
  }
  await page.getByRole("button", { name: "Blocked README" }).click();
  if ((await page.locator("#verdict-title").textContent()) !== "Change denied") {
    throw new Error("Local demo should keep working offline");
  }
  await context.setOffline(false);

  const accessibility = await new AxeBuilder({ page }).analyze();
  const serious = accessibility.violations.filter((item) => ["serious", "critical"].includes(item.impact));
  if (serious.length) {
    throw new Error(`Axe serious/critical violations: ${JSON.stringify(serious, null, 2)}`);
  }
  if (consoleErrors.length) {
    throw new Error(`Console errors: ${consoleErrors.join(" | ")}`);
  }

  console.log("site smoke: mobile interactions, empty/error states, console, and axe passed");
} finally {
  if (browser) await browser.close();
  server.kill("SIGTERM");
}
