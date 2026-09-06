import assert from "node:assert/strict";
import { execFile, spawn } from "node:child_process";
import { readFile } from "node:fs/promises";
import { after, before, test } from "node:test";
import { promisify } from "node:util";
import { chromium } from "playwright";

const execFileAsync = promisify(execFile);
const port = 4180;
const baseURL = `http://127.0.0.1:${port}`;
const preview = spawn(
  process.platform === "win32" ? "node_modules/.bin/vite.cmd" : "node_modules/.bin/vite",
  ["preview", "--config", "site/vite.config.ts", "--host", "127.0.0.1", "--port", String(port)],
  { stdio: "ignore" },
);
let browser;

async function waitForServer() {
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      if ((await fetch(baseURL)).ok) return;
    } catch {
      // The server is still starting.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("Vite preview did not start");
}

async function cargoTest(name) {
  await execFileAsync("cargo", ["test", "--workspace", "--locked", name], { maxBuffer: 4_000_000 });
}

async function withPage(options, callback) {
  const context = await browser.newContext(options);
  const page = await context.newPage();
  try {
    await callback(page, context);
  } finally {
    await context.close();
  }
}

before(async () => {
  await waitForServer();
  browser = await chromium.launch({ headless: true });
});

after(async () => {
  await browser?.close();
  preview.kill("SIGTERM");
});

test("@claim:deterministic-versioned-decisions", async () => {
  await cargoTest("documented_readme_change_is_denied_with_source_line");
});

test("@claim:protected-change-classes", async () => {
  await cargoTest("documented_readme_change_is_denied_with_source_line");
});

test("@claim:hash-bound-generated-artifacts", async () => {
  await cargoTest("edited_generated_file_invalidates_evidence");
  await cargoTest("generated_class_cannot_bypass_missing_hash_bound_entry");
});

test("@claim:generator-metadata-relationships", async () => {
  await cargoTest("generator_metadata_and_relationship_are_all_enforced");
});

test("@claim:line-level-denials", async () => {
  await cargoTest("documented_readme_change_is_denied_with_source_line");
});

test("@claim:json-and-exit-codes", async () => {
  await cargoTest("automatic_range_json_is_a_single_parseable_document");
  await cargoTest("json_mode_keeps_configuration_errors_machine_readable");
});

test("@claim:audited-overrides", async () => {
  await cargoTest("explicit_override_keeps_denials_and_writes_audit");
});

test("@claim:init-preserves-policy", async () => {
  await cargoTest("init_preserves_an_existing_policy_until_force_is_explicit");
});

test("@claim:bundled-cli-demo", async () => {
  await cargoTest("bundled_demo_runs_the_installed_binary_on_a_real_sample_repository");
});

test("@claim:one-binary-cli", async () => {
  const { stdout } = await execFileAsync("target/release/repo-protocol", ["--help"]);
  assert.match(stdout, /Run the bundled policy sample/);
});

test("@claim:local-no-network-cli", async () => {
  const { stdout } = await execFileAsync("target/release/repo-protocol", ["demo"], {
    env: { ...process.env, HTTP_PROXY: "http://127.0.0.1:1", HTTPS_PROXY: "http://127.0.0.1:1", ALL_PROXY: "http://127.0.0.1:1" },
  });
  assert.match(stdout, /Sample result: allowed/);
});

test("@claim:free-mit-license", async () => {
  const metadata = JSON.parse((await execFileAsync("cargo", ["metadata", "--no-deps", "--format-version", "1"])).stdout);
  assert.equal(metadata.packages.find((pkg) => pkg.name === "repo-protocol")?.license, "MIT");
  assert.match(await readFile("LICENSE", "utf8"), /Permission is hereby granted, free of charge/);
});

test("@claim:no-account-required", async () => {
  await withPage({}, async (page) => {
    await page.goto(`${baseURL}/demo`, { waitUntil: "networkidle" });
    await expectTitle(page, "Demo — Repo Protocol Gate");
    assert.equal(await page.locator("#verdict-title").textContent(), "Change allowed");
    assert.equal(await page.locator('input[type="password"]').count(), 0);
  });
});

test("@claim:browser-demo-local", async () => {
  await withPage({}, async (page) => {
    const origins = new Set();
    page.on("request", (request) => origins.add(new URL(request.url()).origin));
    await page.goto(`${baseURL}/demo`, { waitUntil: "networkidle" });
    assert.equal(await page.locator("#demo-mode-banner").isVisible(), true);
    assert.equal(await page.locator("#verdict-title").textContent(), "Change allowed");
    assert.deepEqual(await page.evaluate(() => Object.keys(localStorage)), []);
    assert.deepEqual(await page.evaluate(() => Object.keys(sessionStorage)), ["demo:repo-protocol-gate"]);
    assert.deepEqual([...origins], [baseURL]);
  });
});

test("@claim:demo-reset-and-discard", async () => {
  await withPage({}, async (page) => {
    await page.goto(`${baseURL}/demo`, { waitUntil: "networkidle" });
    await page.getByRole("button", { name: "Blocked README" }).click();
    assert.equal(await page.locator("#verdict-title").textContent(), "Change denied");
    await page.getByRole("button", { name: "Reset demo" }).click();
    assert.equal(await page.locator("#verdict-title").textContent(), "Change allowed");
    await page.getByRole("button", { name: "Start for real" }).click();
    await expectTitle(page, "Repo Protocol Gate — enforce repository rules");
    assert.deepEqual(await page.evaluate(() => Object.keys(sessionStorage)), []);
  });
});

test("@claim:offline-demo", async () => {
  await withPage({}, async (page, context) => {
    await page.goto(baseURL, { waitUntil: "networkidle" });
    await page.evaluate(() => navigator.serviceWorker.ready);
    await page.reload({ waitUntil: "networkidle" });
    await page.goto(`${baseURL}/demo`, { waitUntil: "networkidle" });
    await context.setOffline(true);
    await page.reload({ waitUntil: "domcontentloaded" });
    assert.equal(await page.locator("#network-status").isVisible(), true);
    assert.equal(await page.locator("#verdict-title").textContent(), "Change allowed");
  });
});

test("@claim:documentation-without-javascript", async () => {
  await withPage({ javaScriptEnabled: false }, async (page) => {
    await page.goto(baseURL, { waitUntil: "domcontentloaded" });
    assert.equal(await page.locator("h1").textContent(), "Enforce repository rules in CI");
    assert.equal(await page.locator("#install-command").textContent(), "cargo install --git https://github.com/B-Divyesh/sf-repo-protocol-gate repo-protocol");
  });
});

test("@claim:routes-and-legal-pages", async () => {
  await withPage({}, async (page) => {
    for (const [path, title, heading] of [
      ["/privacy", "Privacy — Repo Protocol Gate", "Privacy for local repository checks"],
      ["/terms", "Terms — Repo Protocol Gate", "Terms for Repo Protocol Gate"],
      ["/missing-page", "Page not found — Repo Protocol Gate", "Page not found"],
    ]) {
      await page.goto(`${baseURL}${path}`, { waitUntil: "networkidle" });
      await expectTitle(page, title);
      assert.equal(await page.locator("h1").textContent(), heading);
    }
  });
});

async function expectTitle(page, title) {
  assert.equal(await page.title(), title);
}
