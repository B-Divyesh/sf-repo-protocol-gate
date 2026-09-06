import { AxeBuilder } from "@axe-core/playwright";
import { chromium } from "playwright";
import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";

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
  const requestOrigins = new Set();
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  page.on("request", (request) => requestOrigins.add(new URL(request.url()).origin));

  await page.goto(baseURL, { waitUntil: "networkidle" });
  if ((await page.title()) !== "Repo Protocol Gate — enforce repository rules") {
    throw new Error("Unexpected document title");
  }
  if ((await page.locator("h1").count()) !== 1 || (await page.locator("main").count()) !== 1) {
    throw new Error("Expected exactly one h1 and one main landmark");
  }
  if (!(await page.locator(".hero-scene img").getAttribute("alt"))) {
    throw new Error("Hero image is missing meaningful alt text");
  }
  if (!(await page.locator('meta[name="twitter:card"]').count()) || !(await page.locator('link[rel="apple-touch-icon"]').count())) {
    throw new Error("Twitter and Apple touch metadata are required");
  }

  await page.setViewportSize({ width: 1440, height: 1000 });
  for (const name of ["Demo", "GitHub"]) {
    const box = await page.getByRole("link", { name, exact: true }).first().boundingBox();
    if (!box || box.width < 44 || box.height < 44) {
      throw new Error(`${name} must have a 44 by 44 CSS pixel touch target`);
    }
  }
  await page.getByRole("link", { name: "Demo", exact: true }).click();
  await page.waitForFunction(() => document.title === "Demo — Repo Protocol Gate");
  if (!(await page.locator("#demo-mode-banner").isVisible())) {
    throw new Error("Demo banner is missing");
  }
  if ((await page.locator("#verdict-title").textContent()) !== "Change allowed") {
    throw new Error("Direct sample demo should load a realistic allowed migration");
  }
  await page.getByRole("button", { name: "Blocked README" }).click();
  if ((await page.locator("#verdict-title").textContent()) !== "Change denied") {
    throw new Error("Desktop demo should deny a protected README change");
  }
  const desktopAccessibility = await new AxeBuilder({ page }).analyze();
  const desktopSerious = desktopAccessibility.violations.filter((item) => ["serious", "critical"].includes(item.impact));
  if (desktopSerious.length) {
    throw new Error(`Desktop axe serious/critical violations: ${JSON.stringify(desktopSerious, null, 2)}`);
  }

  await page.setViewportSize({ width: 390, height: 844 });
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
  await page.goto(baseURL, { waitUntil: "networkidle" });
  await page.keyboard.press("Tab");
  if (!(await page.locator(".skip-link").evaluate((element) => document.activeElement === element))) {
    throw new Error("Skip link must be the first keyboard destination");
  }
  const reducedMotion = await page.evaluate(() => ({
    matches: matchMedia("(prefers-reduced-motion: reduce)").matches,
    duration: getComputedStyle(document.querySelector(".hero-copy")).animationDuration,
  }));
  if (!reducedMotion.matches || Number.parseFloat(reducedMotion.duration) > 0.02) {
    throw new Error("Reduced motion must disable entrance movement");
  }
  await page.evaluate(() => {
    document.documentElement.style.fontSize = "200%";
  });
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
  const primary = await page.getByRole("link", { name: /Try it with sample data/ }).boundingBox();
  if (overflow || !primary || primary.width < 44 || primary.height < 44) {
    throw new Error("200% text must preserve the primary action without horizontal overflow");
  }
  const accessibility = await new AxeBuilder({ page }).analyze();
  const serious = accessibility.violations.filter((item) => ["serious", "critical"].includes(item.impact));
  if (serious.length) {
    throw new Error(`Phone axe serious/critical violations: ${JSON.stringify(serious, null, 2)}`);
  }
  if (consoleErrors.length) {
    throw new Error(`Console errors: ${consoleErrors.join(" | ")}`);
  }
  if ([...requestOrigins].some((origin) => origin !== baseURL)) {
    throw new Error(`Privacy regression: cross-origin requests observed: ${[...requestOrigins].join(", ")}`);
  }
  await context.close();

  const offlineContext = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const offlinePage = await offlineContext.newPage();
  await offlinePage.goto(baseURL, { waitUntil: "networkidle" });
  await offlinePage.evaluate(() => navigator.serviceWorker.ready);
  await offlinePage.reload({ waitUntil: "networkidle" });
  await offlinePage.goto(`${baseURL}/demo`, { waitUntil: "networkidle" });
  await offlineContext.setOffline(true);
  await offlinePage.reload({ waitUntil: "domcontentloaded" });
  if (!(await offlinePage.locator("#network-status").isVisible())) {
    throw new Error("Offline state should be visible");
  }
  if ((await offlinePage.locator("#verdict-title").textContent()) !== "Change allowed") {
    throw new Error("Local sample should keep working offline");
  }
  await offlineContext.close();

  const deployment = JSON.parse(
    await readFile(new URL("../dist/site/staticwebapp.config.json", import.meta.url), "utf8"),
  );
  const cacheRules = new Map(
    deployment.routes.map((route) => [route.route, route.headers?.["Cache-Control"]]),
  );
  if (cacheRules.get("/assets/*") !== "public, max-age=31536000, immutable") {
    throw new Error("Hashed assets must have immutable one-year cache headers");
  }
  if (cacheRules.get("/sw.js") !== "no-cache") {
    throw new Error("The service worker must not be cached");
  }
  if (!deployment.globalHeaders["Content-Security-Policy"]?.includes("frame-ancestors 'none'")) {
    throw new Error("Deployment CSP must prevent framing");
  }
  if (deployment.responseOverrides?.["404"]?.rewrite !== "/404.html") {
    throw new Error("Deployment must return the designed 404 page for missing static routes");
  }
  const static404 = await readFile(new URL("../dist/site/404.html", import.meta.url), "utf8");
  if (!static404.includes("<h1>Page not found</h1>")) {
    throw new Error("Designed static 404 page is missing");
  }

  console.log("site smoke: routes, demo, reset paths, desktop/mobile axe, offline, metadata, privacy, and deployment headers passed");
} finally {
  if (browser) await browser.close();
  server.kill("SIGTERM");
}
