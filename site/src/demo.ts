export type Change = {
  status: "A" | "M" | "D";
  path: string;
};

export type InspectionInput = {
  changes: Change[];
  changeClass: string;
  generator: string;
  ticket: string;
  source: string;
  hashMatches: boolean;
};

export type Inspection = {
  status: "allowed" | "denied" | "empty";
  headline: string;
  summary: string;
  findings: string[];
};

export function parseChanges(raw: string): Change[] {
  const lines = raw
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);

  return lines.map((line, index) => {
    const match = /^(A|M|D)\s+([^\s].*)$/.exec(line);
    if (!match) {
      throw new Error(`Line ${index + 1} must start with A, M, or D, followed by a repository path.`);
    }
    const path = match[2].trim();
    if (path.startsWith("/") || path.includes("..") || path.includes("\\")) {
      throw new Error(`Line ${index + 1} must use a safe repository-relative path.`);
    }
    return { status: match[1] as Change["status"], path };
  });
}

export function inspect(input: InspectionInput): Inspection {
  if (input.changes.length === 0) {
    return {
      status: "empty",
      headline: "Nothing to inspect",
      summary: "Add at least one proposed path. An empty diff passes without invoking a rule.",
      findings: [],
    };
  }

  const findings: string[] = [];
  const hasSchemaChange = input.changes.some((change) => change.path.startsWith("db/schema/"));

  for (const change of input.changes) {
    if ((change.path === "README.md" || change.path === "repo-protocol.yaml") && input.changeClass !== "human") {
      findings.push(`${change.path}: change class “${input.changeClass}” is not allowed; expected human.`);
    }

    const migrationIsProtected =
      change.path.startsWith("db/migrations/") && (change.status === "A" || change.status === "M");
    if (!migrationIsProtected) continue;

    if (input.changeClass !== "generated") {
      findings.push(`${change.path}: change class “${input.changeClass}” is not allowed; expected generated.`);
      continue;
    }
    if (input.generator !== "drizzle-kit") {
      findings.push(`${change.path}: generator “${input.generator || "none"}” is not allowed; expected drizzle-kit.`);
    }
    const missing = [!input.ticket.trim() && "ticket", !input.source.trim() && "source"].filter(Boolean);
    if (missing.length) {
      findings.push(`${change.path}: generator evidence is missing metadata: ${missing.join(", ")}.`);
    }
    if (!input.hashMatches) {
      findings.push(`${change.path}: generator evidence hash does not match the artifact.`);
    }
    if (!hasSchemaChange) {
      findings.push(`${change.path}: requires a companion change matching db/schema/**.`);
    }
  }

  if (findings.length) {
    return {
      status: "denied",
      headline: "Change denied",
      summary: `${findings.length} policy ${findings.length === 1 ? "violation" : "violations"} must be resolved.`,
      findings,
    };
  }

  const protectedCount = input.changes.filter(
    (change) =>
      change.path === "README.md" ||
      change.path === "repo-protocol.yaml" ||
      (change.path.startsWith("db/migrations/") && (change.status === "A" || change.status === "M")),
  ).length;
  return {
    status: "allowed",
    headline: "Change allowed",
    summary: protectedCount
      ? `${protectedCount} protected ${protectedCount === 1 ? "change satisfies" : "changes satisfy"} every rule.`
      : "No protected paths changed.",
    findings: ["Every matching rule returned allow."],
  };
}
