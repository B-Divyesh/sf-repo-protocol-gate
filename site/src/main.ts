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

form.addEventListener("submit", (event) => {
  event.preventDefault();
  runInspection();
});

document.querySelectorAll<HTMLButtonElement>("[data-preset]").forEach((button) => {
  button.addEventListener("click", () => {
    const preset = presets[button.dataset.preset ?? ""];
    if (!preset) return;
    changes.value = preset.changes;
    changeClass.value = preset.changeClass;
    generator.value = preset.generator;
    ticket.value = preset.ticket;
    source.value = preset.source;
    hashMatch.checked = preset.hashMatches;
    runInspection();
    verdict.focus({ preventScroll: true });
    verdict.scrollIntoView({ behavior: "smooth", block: "nearest" });
  });
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

if ("serviceWorker" in navigator && import.meta.env.PROD) {
  window.addEventListener("load", () => {
    void navigator.serviceWorker.register("/sw.js");
  });
}
