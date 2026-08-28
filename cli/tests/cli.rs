use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const POLICY: &str = r#"version: 1
default_change_class: agent
rules:
  - id: humans-own-contracts
    paths: [README.md, repo-protocol.yaml]
    allow:
      change_classes: [human]
  - id: generated-migrations
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
  audit_log: audit.jsonl
"#;

fn fixture() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("repo-protocol-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.email", "test@example.com"]);
    git(&root, &["config", "user.name", "Protocol Test"]);
    fs::write(root.join("repo-protocol.yaml"), POLICY).unwrap();
    fs::write(root.join("README.md"), "# Original\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "initial"]);
    root
}

fn git(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
}

fn gate(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_repo-protocol"))
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}

#[test]
fn documented_readme_change_is_denied_with_source_line() {
    let root = fixture();
    fs::write(root.join("README.md"), "# Agent rewrite\n").unwrap();
    git(&root, &["add", "README.md"]);

    let output = gate(&root, &["check", "--staged", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "denied");
    assert_eq!(report["violations"][0]["rule_id"], "humans-own-contracts");
    assert_eq!(report["violations"][0]["config_line"], 4);
    assert_eq!(report["violations"][0]["path"], "README.md");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn documented_hash_bound_generated_change_is_allowed() {
    let root = fixture();
    fs::create_dir_all(root.join("db/migrations")).unwrap();
    fs::create_dir_all(root.join("db/schema")).unwrap();
    fs::create_dir_all(root.join(".repo-protocol")).unwrap();
    let migration = b"CREATE TABLE users(id int);\n";
    fs::write(root.join("db/migrations/0042_users.sql"), migration).unwrap();
    fs::write(root.join("db/schema/users.sql"), "table users\n").unwrap();
    let hash = format!("{:x}", Sha256::digest(migration));
    let evidence = serde_json::json!({
        "version": 1,
        "generator": "drizzle-kit",
        "metadata": { "ticket": "ENG-204", "source": "db/schema/users.sql" },
        "changes": [{ "path": "db/migrations/0042_users.sql", "sha256": hash }]
    });
    fs::write(
        root.join(".repo-protocol/evidence.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    git(&root, &["add", "."]);

    let output = gate(&root, &["check", "--staged", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "allowed");
    assert_eq!(report["protected_changes"], 1);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn edited_generated_file_invalidates_evidence() {
    let root = fixture();
    fs::create_dir_all(root.join("db/migrations")).unwrap();
    fs::create_dir_all(root.join("db/schema")).unwrap();
    fs::create_dir_all(root.join(".repo-protocol")).unwrap();
    fs::write(root.join("db/migrations/0042_users.sql"), "handwritten\n").unwrap();
    fs::write(root.join("db/schema/users.sql"), "table users\n").unwrap();
    let evidence = serde_json::json!({
        "version": 1,
        "generator": "drizzle-kit",
        "metadata": { "ticket": "ENG-204", "source": "db/schema/users.sql" },
        "changes": [{
            "path": "db/migrations/0042_users.sql",
            "sha256": "0000000000000000000000000000000000000000000000000000000000000000"
        }]
    });
    fs::write(
        root.join(".repo-protocol/evidence.json"),
        serde_json::to_vec(&evidence).unwrap(),
    )
    .unwrap();
    git(&root, &["add", "."]);

    let output = gate(&root, &["check", "--staged"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("evidence hash mismatch"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn generated_class_cannot_bypass_missing_hash_bound_entry() {
    let root = fixture();
    fs::create_dir_all(root.join("db/migrations")).unwrap();
    fs::create_dir_all(root.join("db/schema")).unwrap();
    fs::create_dir_all(root.join(".repo-protocol")).unwrap();
    fs::write(root.join("db/migrations/0042_users.sql"), "handwritten\n").unwrap();
    fs::write(root.join("db/schema/users.sql"), "table users\n").unwrap();
    let evidence = serde_json::json!({
        "version": 1,
        "generator": "drizzle-kit",
        "metadata": { "ticket": "ENG-204", "source": "db/schema/users.sql" },
        "changes": []
    });
    fs::write(
        root.join(".repo-protocol/evidence.json"),
        serde_json::to_vec(&evidence).unwrap(),
    )
    .unwrap();
    git(&root, &["add", "."]);

    let output = gate(
        &root,
        &["check", "--staged", "--change-class", "generated", "--json"],
    );
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "denied");
    assert!(
        report["violations"][0]["message"]
            .as_str()
            .unwrap()
            .contains("hash-bound entry")
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn generated_class_without_evidence_is_a_structured_input_error() {
    let root = fixture();
    fs::create_dir_all(root.join("db/migrations")).unwrap();
    fs::write(root.join("db/migrations/0042_users.sql"), "handwritten\n").unwrap();
    git(&root, &["add", "."]);

    let output = gate(
        &root,
        &["check", "--staged", "--change-class", "generated", "--json"],
    );
    assert_eq!(output.status.code(), Some(2));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "error");
    assert!(
        report["message"]
            .as_str()
            .unwrap()
            .contains("requires an evidence document")
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn automatic_range_json_is_a_single_parseable_document() {
    let root = fixture();
    fs::write(root.join("README.md"), "# Human update\n").unwrap();
    git(&root, &["add", "README.md"]);
    git(&root, &["commit", "-qm", "human update"]);

    let output = gate(&root, &["check", "--change-class", "human", "--json"]);
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "allowed");
    assert_eq!(report["changes_checked"], 1);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn generator_metadata_and_relationship_are_all_enforced() {
    let root = fixture();
    fs::create_dir_all(root.join("db/migrations")).unwrap();
    fs::create_dir_all(root.join(".repo-protocol")).unwrap();
    let migration = b"CREATE TABLE users(id int);\n";
    fs::write(root.join("db/migrations/0042_users.sql"), migration).unwrap();
    let hash = format!("{:x}", Sha256::digest(migration));
    let evidence = serde_json::json!({
        "version": 1,
        "generator": "manual-script",
        "metadata": { "ticket": "ENG-204" },
        "changes": [{ "path": "db/migrations/0042_users.sql", "sha256": hash }]
    });
    fs::write(
        root.join(".repo-protocol/evidence.json"),
        serde_json::to_vec(&evidence).unwrap(),
    )
    .unwrap();
    git(&root, &["add", "."]);

    let output = gate(&root, &["check", "--staged"]);
    assert_eq!(output.status.code(), Some(1));
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(report.contains("generator `manual-script` is not allowed"));
    assert!(report.contains("missing metadata: source"));
    assert!(report.contains("companion change matching one of: db/schema/**"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_override_keeps_denials_and_writes_audit() {
    let root = fixture();
    fs::write(root.join("README.md"), "# Emergency edit\n").unwrap();
    git(&root, &["add", "README.md"]);

    let output = gate(
        &root,
        &[
            "check",
            "--staged",
            "--json",
            "--override-reason",
            "Restore production during INC-481",
            "--actor",
            "oncall@example.com",
        ],
    );
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "overridden");
    assert_eq!(report["violations"].as_array().unwrap().len(), 1);
    let audit = fs::read_to_string(root.join("audit.jsonl")).unwrap();
    assert!(audit.contains("oncall@example.com"));
    assert!(audit.contains("INC-481"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn json_mode_keeps_configuration_errors_machine_readable() {
    let root = fixture();
    fs::write(root.join("repo-protocol.yaml"), "version: 99\nrules: []\n").unwrap();

    let output = gate(&root, &["validate", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "error");
    assert!(
        report["message"]
            .as_str()
            .unwrap()
            .contains("unsupported policy version")
    );

    fs::remove_dir_all(root).unwrap();
}
