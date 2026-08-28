//! Deterministic enforcement engine used by the `repo-protocol` CLI.
//!
//! The public surface is intentionally limited to the three CLI operations.

use globset::{Glob, GlobMatcher};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const POLICY_VERSION: u32 = 1;
const EVIDENCE_VERSION: u32 = 1;

pub const STARTER_POLICY: &str = r#"# Repo Protocol Gate policy
version: 1
default_change_class: agent

rules:
  - id: humans-own-readme
    description: Agents must not rewrite the project contract.
    paths: [README.md, repo-protocol.yaml]
    allow:
      change_classes: [human]

  - id: generated-migrations
    description: Migrations must come from the approved generator after a schema change.
    paths: [db/migrations/**]
    change_types: [added, modified]
    allow:
      change_classes: [generated]
      generators: [drizzle-kit]
    require:
      metadata: [ticket, source]
      changed_any_of: [db/schema/**]

override:
  minimum_reason_length: 12
  require_actor: true
  audit_log: "-"
"#;

#[derive(Debug)]
pub struct AppError(String);

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for AppError {}

type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub struct CheckOptions {
    pub config: PathBuf,
    pub evidence: PathBuf,
    pub base: Option<String>,
    pub head: String,
    pub staged: bool,
    pub change_class: Option<String>,
    pub json: bool,
    pub override_reason: Option<String>,
    pub actor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    version: u32,
    #[serde(default = "default_change_class")]
    default_change_class: String,
    #[serde(default)]
    rules: Vec<Rule>,
    #[serde(default, rename = "override")]
    override_policy: OverridePolicy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    id: String,
    #[serde(default)]
    description: String,
    paths: Vec<String>,
    #[serde(default = "all_change_types")]
    change_types: Vec<ChangeType>,
    allow: Allow,
    #[serde(default)]
    require: Requirements,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Allow {
    change_classes: Vec<String>,
    #[serde(default)]
    generators: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Requirements {
    #[serde(default)]
    metadata: Vec<String>,
    #[serde(default)]
    changed_any_of: Vec<String>,
    #[serde(default)]
    changed_all_of: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct OverridePolicy {
    minimum_reason_length: usize,
    require_actor: bool,
    audit_log: String,
}

impl Default for OverridePolicy {
    fn default() -> Self {
        Self {
            minimum_reason_length: 12,
            require_actor: true,
            audit_log: "-".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

fn all_change_types() -> Vec<ChangeType> {
    vec![
        ChangeType::Added,
        ChangeType::Modified,
        ChangeType::Deleted,
        ChangeType::Renamed,
    ]
}

fn default_change_class() -> String {
    "agent".into()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    version: u32,
    generator: String,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
    changes: Vec<EvidenceChange>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceChange {
    path: String,
    sha256: String,
}

#[derive(Debug, Clone)]
struct Change {
    change_type: ChangeType,
    path: String,
    old_path: Option<String>,
}

#[derive(Debug)]
struct CompiledRule<'a> {
    rule: &'a Rule,
    line: usize,
    paths: Vec<GlobMatcher>,
    any: Vec<GlobMatcher>,
    all: Vec<(String, GlobMatcher)>,
}

#[derive(Debug, Serialize)]
struct Report {
    status: Decision,
    changes_checked: usize,
    protected_changes: usize,
    violations: Vec<Violation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    override_record: Option<OverrideRecord>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Decision {
    Allowed,
    Denied,
    Overridden,
}

#[derive(Debug, Serialize)]
struct Violation {
    rule_id: String,
    config_line: usize,
    path: String,
    message: String,
}

#[derive(Clone, Debug, Serialize)]
struct OverrideRecord {
    version: u32,
    decision: &'static str,
    actor: String,
    reason: String,
    unix_timestamp: u64,
    violations: usize,
}

pub fn run_init(path: &Path, force: bool) -> AppResult<()> {
    if path.exists() && !force {
        return Err(AppError(format!(
            "{} already exists; use --force to replace it",
            path.display()
        )));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| io_error("create", parent, error))?;
    }
    fs::write(path, STARTER_POLICY).map_err(|error| io_error("write", path, error))?;
    println!("Created {}", path.display());
    println!("Next: review the trust classes, then run `repo-protocol validate`.");
    Ok(())
}

pub fn run_validate(path: &Path, json: bool) -> AppResult<()> {
    let (policy, raw) = load_policy(path)?;
    let compiled = compile_policy(&policy, &raw)?;
    if json {
        println!(
            "{}",
            serde_json::json!({
                "status": "valid",
                "version": policy.version,
                "rules": compiled.len(),
                "config": path,
            })
        );
    } else if compiled.is_empty() {
        println!(
            "Valid {} — version 1, no protection rules (all changes pass).",
            path.display()
        );
    } else {
        println!(
            "Valid {} — version 1, {} rule{}.",
            path.display(),
            compiled.len(),
            if compiled.len() == 1 { "" } else { "s" }
        );
        for rule in compiled {
            let detail = if rule.rule.description.is_empty() {
                "No description"
            } else {
                &rule.rule.description
            };
            println!("  line {} [{}] {detail}", rule.line, rule.rule.id);
        }
    }
    Ok(())
}

pub fn run_check(options: CheckOptions) -> AppResult<()> {
    let (policy, raw) = load_policy(&options.config)?;
    let compiled = compile_policy(&policy, &raw)?;
    let changes = git_changes(&options)?;
    let evidence = load_evidence(&options.evidence, &options)?;
    validate_evidence(evidence.as_ref())?;
    let ordinary_class = options
        .change_class
        .as_deref()
        .unwrap_or(&policy.default_change_class);
    validate_name("change class", ordinary_class)?;

    let mut report = evaluate(
        &compiled,
        &changes,
        evidence.as_ref(),
        ordinary_class,
        &options,
    )?;

    if let Some(reason) = options.override_reason.as_deref() {
        if report.violations.is_empty() {
            return Err(AppError(
                "override supplied, but the change has no policy denials".into(),
            ));
        }
        let actor = options.actor.as_deref().unwrap_or_default().trim();
        if policy.override_policy.require_actor && actor.is_empty() {
            return Err(AppError("override requires a non-empty --actor".into()));
        }
        if reason.trim().chars().count() < policy.override_policy.minimum_reason_length {
            return Err(AppError(format!(
                "override reason must be at least {} characters",
                policy.override_policy.minimum_reason_length
            )));
        }
        let record = OverrideRecord {
            version: 1,
            decision: "overridden",
            actor: actor.into(),
            reason: reason.trim().into(),
            unix_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| AppError("system clock is before the Unix epoch".into()))?
                .as_secs(),
            violations: report.violations.len(),
        };
        write_audit(&policy.override_policy.audit_log, &options.config, &record)?;
        report.status = Decision::Overridden;
        report.override_record = Some(record);
    }

    print_report(&report, &options.config, options.json)?;
    if report.status == Decision::Denied {
        std::process::exit(1);
    }
    Ok(())
}

fn load_policy(path: &Path) -> AppResult<(Policy, String)> {
    let raw = fs::read_to_string(path).map_err(|error| io_error("read", path, error))?;
    let policy: Policy = serde_yaml::from_str(&raw).map_err(|error| {
        let location = error
            .location()
            .map(|at| format!(":{}:{}", at.line(), at.column()))
            .unwrap_or_default();
        AppError(format!("{}{}: {error}", path.display(), location))
    })?;
    Ok((policy, raw))
}

fn compile_policy<'a>(policy: &'a Policy, raw: &str) -> AppResult<Vec<CompiledRule<'a>>> {
    if policy.version != POLICY_VERSION {
        return Err(AppError(format!(
            "unsupported policy version {}; expected {POLICY_VERSION}",
            policy.version
        )));
    }
    validate_name("default_change_class", &policy.default_change_class)?;
    if policy.override_policy.minimum_reason_length < 8 {
        return Err(AppError(
            "override.minimum_reason_length must be at least 8".into(),
        ));
    }
    if policy.override_policy.audit_log.trim().is_empty() {
        return Err(AppError("override.audit_log cannot be empty".into()));
    }

    let mut ids = BTreeSet::new();
    let mut compiled = Vec::with_capacity(policy.rules.len());
    for rule in &policy.rules {
        validate_name("rule id", &rule.id)?;
        if !ids.insert(rule.id.as_str()) {
            return Err(AppError(format!("duplicate rule id `{}`", rule.id)));
        }
        if rule.paths.is_empty() {
            return Err(AppError(format!("rule `{}` must include paths", rule.id)));
        }
        if rule.allow.change_classes.is_empty() {
            return Err(AppError(format!(
                "rule `{}` must allow at least one change class",
                rule.id
            )));
        }
        if rule.change_types.is_empty() {
            return Err(AppError(format!(
                "rule `{}` must include at least one change type",
                rule.id
            )));
        }
        let unique_types: BTreeSet<_> = rule
            .change_types
            .iter()
            .map(|kind| format!("{kind:?}"))
            .collect();
        if unique_types.len() != rule.change_types.len() {
            return Err(AppError(format!(
                "rule `{}` repeats a change type",
                rule.id
            )));
        }
        for class in &rule.allow.change_classes {
            validate_name("change class", class)?;
        }
        for generator in &rule.allow.generators {
            validate_name("generator", generator)?;
        }
        for key in &rule.require.metadata {
            validate_name("metadata key", key)?;
        }
        let paths = compile_globs(&rule.id, "paths", &rule.paths)?;
        let any = compile_globs(
            &rule.id,
            "require.changed_any_of",
            &rule.require.changed_any_of,
        )?;
        let all_matchers = compile_globs(
            &rule.id,
            "require.changed_all_of",
            &rule.require.changed_all_of,
        )?;
        let all = rule
            .require
            .changed_all_of
            .iter()
            .cloned()
            .zip(all_matchers)
            .collect();
        compiled.push(CompiledRule {
            rule,
            line: rule_line(raw, &rule.id),
            paths,
            any,
            all,
        });
    }
    Ok(compiled)
}

fn compile_globs(rule: &str, field: &str, patterns: &[String]) -> AppResult<Vec<GlobMatcher>> {
    patterns
        .iter()
        .map(|pattern| {
            Glob::new(pattern)
                .map(|glob| glob.compile_matcher())
                .map_err(|error| AppError(format!("rule `{rule}` {field} `{pattern}`: {error}")))
        })
        .collect()
}

fn rule_line(raw: &str, id: &str) -> usize {
    raw.lines()
        .position(|line| {
            line.trim_start()
                .strip_prefix("- id:")
                .map(str::trim)
                .map(|value| value.trim_matches(['\'', '"']) == id)
                .unwrap_or(false)
        })
        .map(|line| line + 1)
        .unwrap_or(1)
}

fn validate_name(label: &str, value: &str) -> AppResult<()> {
    let valid = !value.is_empty()
        && value.len() <= 80
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-/@".contains(character));
    if valid {
        Ok(())
    } else {
        Err(AppError(format!(
            "invalid {label} `{value}`; use 1–80 letters, numbers, `.`, `_`, `-`, `/`, or `@`"
        )))
    }
}

fn git_changes(options: &CheckOptions) -> AppResult<Vec<Change>> {
    let mut command = Command::new("git");
    command.args(["diff", "--name-status", "-z", "--diff-filter=ACMDR"]);
    if options.staged {
        command.arg("--cached");
    } else if let Some(base) = options.base.as_deref() {
        command.arg(format!("{base}...{}", options.head));
    } else if has_staged_changes()? {
        command.arg("--cached");
    } else if git_success(["rev-parse", "--verify", "HEAD^"])? {
        command.arg("HEAD^..HEAD");
    } else {
        return Err(AppError(
            "no staged changes and HEAD has no parent; use --staged after adding files or provide --base"
                .into(),
        ));
    }
    let output = command
        .output()
        .map_err(|error| AppError(format!("could not run git: {error}")))?;
    if !output.status.success() {
        return Err(AppError(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    parse_name_status(&output.stdout)
}

fn has_staged_changes() -> AppResult<bool> {
    let status = Command::new("git")
        .args(["diff", "--cached", "--quiet", "--exit-code"])
        .status()
        .map_err(|error| AppError(format!("could not run git: {error}")))?;
    match status.code() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(AppError("git could not inspect staged changes".into())),
    }
}

fn git_success<const N: usize>(args: [&str; N]) -> AppResult<bool> {
    Command::new("git")
        .args(args)
        .status()
        .map(|status| status.success())
        .map_err(|error| AppError(format!("could not run git: {error}")))
}

fn parse_name_status(bytes: &[u8]) -> AppResult<Vec<Change>> {
    let fields: Vec<&[u8]> = bytes
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .collect();
    let mut changes = Vec::new();
    let mut cursor = 0;
    while cursor < fields.len() {
        let status = std::str::from_utf8(fields[cursor])
            .map_err(|_| AppError("git returned a non-UTF-8 change status".into()))?;
        cursor += 1;
        let kind = match status.as_bytes().first() {
            Some(b'A') => ChangeType::Added,
            Some(b'M') => ChangeType::Modified,
            Some(b'D') => ChangeType::Deleted,
            Some(b'R') => ChangeType::Renamed,
            _ => {
                return Err(AppError(format!(
                    "unsupported git change status `{status}`"
                )));
            }
        };
        let old_or_path = next_path(&fields, &mut cursor)?;
        let (old_path, path) = if kind == ChangeType::Renamed {
            (Some(old_or_path), next_path(&fields, &mut cursor)?)
        } else {
            (None, old_or_path)
        };
        changes.push(Change {
            change_type: kind,
            path,
            old_path,
        });
    }
    Ok(changes)
}

fn next_path(fields: &[&[u8]], cursor: &mut usize) -> AppResult<String> {
    let field = fields
        .get(*cursor)
        .ok_or_else(|| AppError("git returned a truncated name-status record".into()))?;
    *cursor += 1;
    std::str::from_utf8(field)
        .map(str::to_owned)
        .map_err(|_| AppError("a changed path is not valid UTF-8".into()))
}

fn load_evidence(path: &Path, options: &CheckOptions) -> AppResult<Option<Evidence>> {
    let raw = if path.is_absolute() {
        if !path.exists() {
            return Ok(None);
        }
        fs::read(path).map_err(|error| io_error("read", path, error))?
    } else {
        let path = path.to_str().ok_or_else(|| {
            AppError("the evidence path must be valid UTF-8 when read from Git".into())
        })?;
        let spec = snapshot_spec(path, options)?;
        let output = Command::new("git")
            .args(["show", &spec])
            .output()
            .map_err(|error| AppError(format!("could not run git: {error}")))?;
        if !output.status.success() {
            return Ok(None);
        }
        output.stdout
    };
    serde_json::from_slice(&raw)
        .map(Some)
        .map_err(|error| AppError(format!("{}: {error}", path.display())))
}

fn validate_evidence(evidence: Option<&Evidence>) -> AppResult<()> {
    let Some(evidence) = evidence else {
        return Ok(());
    };
    if evidence.version != EVIDENCE_VERSION {
        return Err(AppError(format!(
            "unsupported evidence version {}; expected {EVIDENCE_VERSION}",
            evidence.version
        )));
    }
    validate_name("generator", &evidence.generator)?;
    let mut paths = BTreeSet::new();
    for change in &evidence.changes {
        if change.path.is_empty()
            || Path::new(&change.path).is_absolute()
            || change.path.contains("..")
        {
            return Err(AppError(format!("invalid evidence path `{}`", change.path)));
        }
        if !paths.insert(&change.path) {
            return Err(AppError(format!("evidence repeats path `{}`", change.path)));
        }
        if change.sha256.len() != 64 || !change.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(AppError(format!(
                "evidence sha256 for `{}` must be 64 hexadecimal characters",
                change.path
            )));
        }
    }
    Ok(())
}

fn evaluate(
    rules: &[CompiledRule<'_>],
    changes: &[Change],
    evidence: Option<&Evidence>,
    ordinary_class: &str,
    options: &CheckOptions,
) -> AppResult<Report> {
    let all_paths: Vec<&str> = changes
        .iter()
        .flat_map(|change| std::iter::once(change.path.as_str()).chain(change.old_path.as_deref()))
        .collect();
    let mut violations = Vec::new();
    let mut protected = BTreeSet::new();

    for change in changes {
        for compiled in rules {
            if !compiled.rule.change_types.contains(&change.change_type)
                || !change_matches(&compiled.paths, change)
            {
                continue;
            }
            protected.insert(change.path.as_str());
            let evidence_state = evidence_for(change, evidence, options)?;
            let effective_class = if evidence_state.valid {
                "generated"
            } else {
                ordinary_class
            };

            if !compiled
                .rule
                .allow
                .change_classes
                .iter()
                .any(|allowed| allowed == effective_class)
            {
                let message = evidence_state.problem.unwrap_or_else(|| {
                    format!(
                        "change class `{effective_class}` is not allowed; expected {}",
                        compiled.rule.allow.change_classes.join(", ")
                    )
                });
                push_violation(&mut violations, compiled, change, message);
                continue;
            }

            if effective_class == "generated" {
                let evidence = evidence.expect("valid evidence has a document");
                if !compiled.rule.allow.generators.is_empty()
                    && !compiled.rule.allow.generators.contains(&evidence.generator)
                {
                    push_violation(
                        &mut violations,
                        compiled,
                        change,
                        format!(
                            "generator `{}` is not allowed; expected {}",
                            evidence.generator,
                            compiled.rule.allow.generators.join(", ")
                        ),
                    );
                }
                let missing: Vec<&str> = compiled
                    .rule
                    .require
                    .metadata
                    .iter()
                    .filter(|key| {
                        evidence
                            .metadata
                            .get(key.as_str())
                            .map(|value| value.trim().is_empty())
                            .unwrap_or(true)
                    })
                    .map(String::as_str)
                    .collect();
                if !missing.is_empty() {
                    push_violation(
                        &mut violations,
                        compiled,
                        change,
                        format!(
                            "generator evidence is missing metadata: {}",
                            missing.join(", ")
                        ),
                    );
                }
            }

            if !compiled.any.is_empty()
                && !compiled
                    .any
                    .iter()
                    .any(|matcher| all_paths.iter().any(|path| matcher.is_match(path)))
            {
                push_violation(
                    &mut violations,
                    compiled,
                    change,
                    format!(
                        "requires a companion change matching one of: {}",
                        compiled.rule.require.changed_any_of.join(", ")
                    ),
                );
            }
            for (pattern, matcher) in &compiled.all {
                if !all_paths.iter().any(|path| matcher.is_match(path)) {
                    push_violation(
                        &mut violations,
                        compiled,
                        change,
                        format!("requires a companion change matching `{pattern}`"),
                    );
                }
            }
        }
    }

    Ok(Report {
        status: if violations.is_empty() {
            Decision::Allowed
        } else {
            Decision::Denied
        },
        changes_checked: changes.len(),
        protected_changes: protected.len(),
        violations,
        override_record: None,
    })
}

struct EvidenceState {
    valid: bool,
    problem: Option<String>,
}

fn evidence_for(
    change: &Change,
    evidence: Option<&Evidence>,
    options: &CheckOptions,
) -> AppResult<EvidenceState> {
    let Some(document) = evidence else {
        return Ok(EvidenceState {
            valid: false,
            problem: None,
        });
    };
    let Some(entry) = document
        .changes
        .iter()
        .find(|entry| entry.path == change.path)
    else {
        return Ok(EvidenceState {
            valid: false,
            problem: None,
        });
    };
    if change.change_type == ChangeType::Deleted {
        return Ok(EvidenceState {
            valid: false,
            problem: Some("deleted files cannot carry valid generator evidence".into()),
        });
    }
    let bytes = target_file_bytes(&change.path, options)?;
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual.eq_ignore_ascii_case(&entry.sha256) {
        Ok(EvidenceState {
            valid: true,
            problem: None,
        })
    } else {
        Ok(EvidenceState {
            valid: false,
            problem: Some(format!(
                "generator evidence hash mismatch (expected {}, found {actual})",
                entry.sha256
            )),
        })
    }
}

fn target_file_bytes(path: &str, options: &CheckOptions) -> AppResult<Vec<u8>> {
    let spec = snapshot_spec(path, options)?;
    let output = Command::new("git")
        .args(["show", &spec])
        .output()
        .map_err(|error| AppError(format!("could not run git: {error}")))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(AppError(format!(
            "could not read `{path}` from the checked Git snapshot: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn snapshot_spec(path: &str, options: &CheckOptions) -> AppResult<String> {
    Ok(
        if options.staged || (options.base.is_none() && has_staged_changes()?) {
            format!(":{path}")
        } else {
            format!("{}:{path}", options.head)
        },
    )
}

fn change_matches(matchers: &[GlobMatcher], change: &Change) -> bool {
    matchers.iter().any(|matcher| {
        matcher.is_match(&change.path)
            || change
                .old_path
                .as_deref()
                .map(|path| matcher.is_match(path))
                .unwrap_or(false)
    })
}

fn push_violation(
    violations: &mut Vec<Violation>,
    compiled: &CompiledRule<'_>,
    change: &Change,
    message: String,
) {
    let path = change
        .old_path
        .as_deref()
        .map(|old| format!("{old} → {}", change.path))
        .unwrap_or_else(|| change.path.clone());
    violations.push(Violation {
        rule_id: compiled.rule.id.clone(),
        config_line: compiled.line,
        path,
        message,
    });
}

fn write_audit(log: &str, config: &Path, record: &OverrideRecord) -> AppResult<()> {
    let json = serde_json::to_string(record)
        .map_err(|error| AppError(format!("could not encode override audit: {error}")))?;
    if log == "-" {
        eprintln!("repo-protocol override audit: {json}");
        return Ok(());
    }
    let path = if Path::new(log).is_absolute() {
        PathBuf::from(log)
    } else {
        config.parent().unwrap_or_else(|| Path::new(".")).join(log)
    };
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| io_error("create", parent, error))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| io_error("open", &path, error))?;
    writeln!(file, "{json}").map_err(|error| io_error("write", &path, error))
}

fn print_report(report: &Report, config: &Path, json: bool) -> AppResult<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(report)
                .map_err(|error| AppError(format!("could not encode report: {error}")))?
        );
        return Ok(());
    }

    match report.status {
        Decision::Allowed if report.changes_checked == 0 => {
            println!("ALLOW — no changes to inspect.");
        }
        Decision::Allowed if report.protected_changes == 0 => {
            println!(
                "ALLOW — {} change{} inspected; no protected paths changed.",
                report.changes_checked,
                plural(report.changes_checked)
            );
        }
        Decision::Allowed => {
            println!(
                "ALLOW — {} protected change{} satisfied every rule.",
                report.protected_changes,
                plural(report.protected_changes)
            );
        }
        Decision::Denied | Decision::Overridden => {
            let label = if report.status == Decision::Denied {
                "DENY"
            } else {
                "OVERRIDE"
            };
            println!(
                "{label} — {} policy violation{}.",
                report.violations.len(),
                plural(report.violations.len())
            );
            for violation in &report.violations {
                println!(
                    "  {}:{} [{}] {}: {}",
                    config.display(),
                    violation.config_line,
                    violation.rule_id,
                    violation.path,
                    violation.message
                );
            }
            if report.status == Decision::Overridden {
                println!("Override accepted and audited; the original denials remain above.");
            } else {
                println!("Fix the change or use an accountable emergency override.");
            }
        }
    }
    Ok(())
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

fn io_error(action: &str, path: &Path, error: std::io::Error) -> AppError {
    AppError(format!("could not {action} {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(raw: &str) -> (Policy, String) {
        (serde_yaml::from_str(raw).unwrap(), raw.to_string())
    }

    #[test]
    fn parses_nul_delimited_rename() {
        let changes =
            parse_name_status(b"R100\0README.md\0docs/README.md\0M\0src/lib.rs\0").unwrap();
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].change_type, ChangeType::Renamed);
        assert_eq!(changes[0].old_path.as_deref(), Some("README.md"));
        assert_eq!(changes[0].path, "docs/README.md");
    }

    #[test]
    fn config_reports_rule_source_line() {
        let (policy, raw) = policy(
            "version: 1\nrules:\n  - id: readme\n    paths: [README.md]\n    allow:\n      change_classes: [human]\n",
        );
        let compiled = compile_policy(&policy, &raw).unwrap();
        assert_eq!(compiled[0].line, 3);
    }

    #[test]
    fn rejects_unknown_policy_fields() {
        let parsed: Result<Policy, _> = serde_yaml::from_str("version: 1\nsurprise: true\n");
        assert!(parsed.is_err());
    }

    #[test]
    fn starter_policy_is_valid() {
        let policy: Policy = serde_yaml::from_str(STARTER_POLICY).unwrap();
        assert_eq!(compile_policy(&policy, STARTER_POLICY).unwrap().len(), 2);
    }

    #[test]
    fn bare_path_glob_does_not_match_nearby_file() {
        let matcher = Glob::new("README.md").unwrap().compile_matcher();
        assert!(matcher.is_match("README.md"));
        assert!(!matcher.is_match("docs/README.md"));
        assert!(!matcher.is_match("README.md.bak"));
    }
}
