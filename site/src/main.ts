import "./styles.css";
import { inspect, parseChanges, type InspectionInput } from "./demo";

const required = <T extends Element>(selector: string): T => {
  const element = document.querySelector<T>(selector);
  if (!element) throw new Error(`Missing required element: ${selector}`);
  return element;
};

const form = required<HTMLFormElement>("#gate-form");
const changes = required<HTMLTextAreaElement>("#changes");
const changeClass = required<HTMLSelectElement>("#change-class");
const generator = required<HTMLSelectElement>("#generator");
const ticket = required<HTMLInputElement>("#ticket");
const source = required<HTMLInputElement>("#source");
const hashMatch = required<HTMLInputElement>("#hash-match");
const verdict = required<HTMLElement>(".verdict");
const verdictTitle = required<HTMLElement>("#verdict-title");
const verdictSummary = required<HTMLElement>("#verdict-summary");
const verdictSeal = required<HTMLElement>("#verdict-seal");
const verdictList = required<HTMLOListElement>("#verdict-list");
const networkStatus = required<HTMLElement>("#network-status");
const landingContent = required<HTMLElement>("#landing-content");
const routeContent = required<HTMLElement>("#route-content");
const pageTitle = required<HTMLHeadingElement>("#hero-title");
const heroTitleSlot = required<HTMLElement>("#hero-title-slot");
const demoBanner = required<HTMLElement>("#demo-mode-banner");
const routeAnnouncer = required<HTMLElement>("#route-announcer");
const canonical = required<HTMLLinkElement>('link[rel="canonical"]');

type Preset = Omit<InspectionInput, "changes"> & { changes: string };

const presets: Record<string, Preset> = {
  readme: {
    changes: "M README.md",
    changeClass: "agent",
    generator: "",
    ticket: "",
    source: "",
    hashMatches: false,
  },
  valid: {
    changes: "M db/schema/users.ts\nA db/migrations/0042_users.sql",
    changeClass: "generated",
    generator: "drizzle-kit",
    ticket: "ENG-204",
    source: "db/schema/users.ts",
    hashMatches: true,
  },
  orphan: {
    changes: "A db/migrations/0042_users.sql",
    changeClass: "generated",
    generator: "drizzle-kit",
    ticket: "ENG-204",
    source: "db/schema/users.ts",
    hashMatches: true,
  },
};

function render(input: InspectionInput): void {
  const result = inspect(input);
  verdict.dataset.state = result.status;
  verdictTitle.textContent = result.headline;
  verdictSummary.textContent = result.summary;
  verdictSeal.textContent = result.status === "allowed" ? "✓" : result.status === "denied" ? "×" : "0";
  verdictList.replaceChildren(
    ...result.findings.map((finding) => {
      const item = document.createElement("li");
      item.textContent = finding;
      return item;
    }),
  );
}

function renderError(message: string): void {
  verdict.dataset.state = "error";
  verdictTitle.textContent = "Input needs attention";
  verdictSummary.textContent = message;
  verdictSeal.textContent = "!";
  verdictList.replaceChildren();
  verdict.setAttribute("role", "alert");
}

function runInspection(): void {
  verdict.removeAttribute("role");
  try {
    render({
      changes: parseChanges(changes.value),
      changeClass: changeClass.value,
      generator: generator.value,
      ticket: ticket.value,
      source: source.value,
      hashMatches: hashMatch.checked,
    });
  } catch (error) {
    renderError(error instanceof Error ? error.message : "The proposed diff could not be read.");
  }
}

function loadPreset(name: string, moveToVerdict = true): void {
  const preset = presets[name];
  if (!preset) return;
  changes.value = preset.changes;
  changeClass.value = preset.changeClass;
  generator.value = preset.generator;
  ticket.value = preset.ticket;
  source.value = preset.source;
  hashMatch.checked = preset.hashMatches;
  runInspection();
  if (moveToVerdict) {
    verdict.focus({ preventScroll: true });
    verdict.scrollIntoView({ behavior: "smooth", block: "nearest" });
  }
}

function clearSample(): void {
  changes.value = "";
  changeClass.value = "agent";
  generator.value = "";
  ticket.value = "";
  source.value = "";
  hashMatch.checked = false;
  render({
    changes: [],
    changeClass: "agent",
    generator: "",
    ticket: "",
    source: "",
    hashMatches: false,
  });
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  runInspection();
});

document.querySelectorAll<HTMLButtonElement>("[data-preset]").forEach((button) => {
  button.addEventListener("click", () => loadPreset(button.dataset.preset ?? ""));
});

function updateNetworkStatus(): void {
  networkStatus.hidden = navigator.onLine;
}
window.addEventListener("online", updateNetworkStatus);
window.addEventListener("offline", updateNetworkStatus);
updateNetworkStatus();

const copyButton = required<HTMLButtonElement>("#copy-command");
const copyStatus = required<HTMLElement>("#copy-status");
copyButton.addEventListener("click", async () => {
  const command = required<HTMLElement>("#install-command").textContent ?? "";
  try {
    await navigator.clipboard.writeText(command);
    copyButton.textContent = "Copied";
    copyStatus.textContent = "Install command copied to the clipboard.";
  } catch {
    copyButton.textContent = "Select command above";
    copyStatus.textContent = "Clipboard access was unavailable. Select and copy the command above.";
  }
  window.setTimeout(() => {
    copyButton.textContent = "Copy command";
  }, 2400);
});

type Route = "/" | "/demo" | "/privacy" | "/terms" | "/404";

const normalisePath = (path: string): Route => {
  const value = path.length > 1 ? path.replace(/\/+$/, "") : path;
  if (value === "/" || value === "/demo" || value === "/privacy" || value === "/terms") return value;
  return "/404";
};

function setMetadata(title: string, path: Route, description: string): void {
  document.title = title;
  const url = new URL(path === "/404" ? "/404" : path, window.location.origin).toString();
  canonical.href = url;
  const descriptionTag = document.querySelector<HTMLMetaElement>('meta[name="description"]');
  if (descriptionTag) descriptionTag.content = description;
  const ogUrl = document.querySelector<HTMLMetaElement>('meta[property="og:url"]');
  if (ogUrl) ogUrl.content = url;
}

function restoreLanding(title: string): void {
  landingContent.hidden = false;
  routeContent.hidden = true;
  routeContent.replaceChildren();
  heroTitleSlot.append(pageTitle);
  pageTitle.textContent = title;
}

function renderInformationRoute(
  heading: string,
  eyebrow: string,
  body: string,
  title: string,
  description: string,
  path: Route,
): void {
  landingContent.hidden = true;
  routeContent.hidden = false;
  routeContent.innerHTML = `<div class="legal-page"><p class="eyebrow">${eyebrow}</p><div id="route-title-slot"></div>${body}</div>`;
  required<HTMLElement>("#route-title-slot").append(pageTitle);
  pageTitle.textContent = heading;
  setMetadata(title, path, description);
}

function focusRouteTitle(): void {
  pageTitle.focus({ preventScroll: true });
  routeAnnouncer.textContent = `${document.title}.`;
}

function setDemoStorage(active: boolean): void {
  try {
    if (active) sessionStorage.setItem("demo:repo-protocol-gate", "sample-v1");
    else sessionStorage.removeItem("demo:repo-protocol-gate");
  } catch {
    // The demo stays in memory if browser storage is unavailable.
  }
}

function renderRoute(path: Route, moveFocus = true): void {
  demoBanner.hidden = true;
  if (path === "/") {
    restoreLanding("Enforce repository rules in CI");
    setMetadata(
      "Repo Protocol Gate — enforce repository rules",
      path,
      "Check protected repository changes in CI with versioned rules, generated-file evidence, metadata, and companion changes.",
    );
  } else if (path === "/demo") {
    restoreLanding("Inspect sample repository changes");
    demoBanner.hidden = false;
    setDemoStorage(true);
    loadPreset("valid", false);
    setMetadata(
      "Demo — Repo Protocol Gate",
      path,
      "Try an approved sample migration in the local Repo Protocol Gate demo.",
    );
  } else if (path === "/privacy") {
    renderInformationRoute(
      "Privacy for local repository checks",
      "Privacy",
      `<p>Repo Protocol Gate runs its checks on the machine where you invoke it.</p>
       <h2>CLI data</h2><p>The CLI reads the Git change set, your policy, and optional evidence file. It does not need an account, a server, or a network request to make a decision.</p>
       <h2>Website data</h2><p>The documentation site has no analytics or advertising scripts. The sample form runs in the browser. Demo state uses only the <code>demo:</code> browser-session namespace and is removed when you choose Start for real.</p>
       <h2>Contact</h2><p>Questions about this policy can be raised in the project repository.</p>`,
      "Privacy — Repo Protocol Gate",
      "Read how Repo Protocol Gate keeps CLI checks and the sample demo local.",
      path,
    );
  } else if (path === "/terms") {
    renderInformationRoute(
      "Terms for Repo Protocol Gate",
      "Terms",
      `<p>Repo Protocol Gate is free software under the MIT License.</p>
       <h2>Your responsibility</h2><p>Review policies before relying on them in CI. Treat change-class inputs, overrides, and generator evidence as trusted workflow inputs.</p>
       <h2>No warranty</h2><p>The MIT License provides the software as is. It does not replace code review, backups, or access controls.</p>
       <h2>Source and license</h2><p>Read the complete <a href="https://github.com/B-Divyesh/sf-repo-protocol-gate/blob/main/LICENSE">MIT License</a> in the repository.</p>`,
      "Terms — Repo Protocol Gate",
      "Read the terms and MIT License for Repo Protocol Gate.",
      path,
    );
  } else {
    renderInformationRoute(
      "Page not found",
      "404",
      `<p>The address does not match a Repo Protocol Gate page.</p><a class="not-found-link" href="/">Go to the documentation</a>`,
      "Page not found — Repo Protocol Gate",
      "The requested Repo Protocol Gate page was not found.",
      path,
    );
  }
  if (moveFocus) focusRouteTitle();
  if (path === "/demo") {
    window.requestAnimationFrame(() => required<HTMLElement>("#demo").scrollIntoView({ behavior: "smooth", block: "start" }));
  }
}

function navigate(path: string): void {
  const route = normalisePath(path);
  const target = route === "/404" ? path : route;
  window.history.pushState({}, "", target);
  renderRoute(route);
}

document.addEventListener("click", (event) => {
  const target = event.target instanceof Element ? event.target.closest<HTMLAnchorElement>("a[href]") : null;
  if (!target || event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
  const url = new URL(target.href, window.location.href);
  if (url.origin !== window.location.origin || url.hash || !["/", "/demo", "/privacy", "/terms"].includes(url.pathname)) return;
  event.preventDefault();
  navigate(url.pathname);
});

window.addEventListener("popstate", () => renderRoute(normalisePath(window.location.pathname)));
required<HTMLButtonElement>("#reset-demo").addEventListener("click", () => {
  loadPreset("valid", false);
  setDemoStorage(true);
  routeAnnouncer.textContent = "Demo reset to the approved migration sample.";
  required<HTMLElement>("#demo").scrollIntoView({ behavior: "smooth", block: "start" });
});
required<HTMLButtonElement>("#start-real").addEventListener("click", () => {
  setDemoStorage(false);
  clearSample();
  navigate("/");
  routeAnnouncer.textContent = "Demo data discarded. You can now enter a change to inspect.";
});

renderRoute(normalisePath(window.location.pathname), false);

if ("serviceWorker" in navigator && import.meta.env.PROD) {
  window.addEventListener("load", () => {
    void navigator.serviceWorker.register("/sw.js");
  });
}
