use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "repo-protocol",
    version,
    about = "Enforce repository protocol before changes merge",
    long_about = "Repo Protocol Gate checks Git changes against versioned path, generator, metadata, and relationship rules. It is deterministic, non-interactive, and makes no network requests.",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check staged files or a Git commit range against the policy
    Check(CheckArgs),
    /// Parse and validate a policy without inspecting Git changes
    Validate(ValidateArgs),
    /// Write a commented starter policy
    Init(InitArgs),
}

#[derive(Debug, Args)]
struct CheckArgs {
    /// Policy file to load
    #[arg(long, default_value = "repo-protocol.yaml")]
    config: PathBuf,

    /// Generator evidence document
    #[arg(long, default_value = ".repo-protocol/evidence.json")]
    evidence: PathBuf,

    /// Base Git revision (compared as BASE...HEAD)
    #[arg(long, conflicts_with = "staged")]
    base: Option<String>,

    /// Head Git revision used with --base
    #[arg(long, default_value = "HEAD", requires = "base")]
    head: String,

    /// Inspect the Git index instead of a commit range
    #[arg(long)]
    staged: bool,

    /// Trust class for ordinary changes (prefer the environment in CI)
    #[arg(long, env = "REPO_PROTOCOL_CHANGE_CLASS")]
    change_class: Option<String>,

    /// Emit one JSON report on stdout
    #[arg(long)]
    json: bool,

    /// Explicit emergency override reason
    #[arg(long, requires = "actor")]
    override_reason: Option<String>,

    /// Identity accountable for an emergency override
    #[arg(long, requires = "override_reason")]
    actor: Option<String>,
}

#[derive(Debug, Args)]
struct ValidateArgs {
    /// Policy file to load
    #[arg(long, default_value = "repo-protocol.yaml")]
    config: PathBuf,

    /// Emit a JSON validation report
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct InitArgs {
    /// Policy file to create
    #[arg(long, default_value = "repo-protocol.yaml")]
    config: PathBuf,

    /// Replace an existing file
    #[arg(long)]
    force: bool,
}

fn main() {
    let cli = Cli::parse();
    let wants_json = matches!(
        &cli.command,
        Command::Check(CheckArgs { json: true, .. })
            | Command::Validate(ValidateArgs { json: true, .. })
    );
    let code = match cli.command {
        Command::Check(args) => repo_protocol::run_check(repo_protocol::CheckOptions {
            config: args.config,
            evidence: args.evidence,
            base: args.base,
            head: args.head,
            staged: args.staged,
            change_class: args.change_class,
            json: args.json,
            override_reason: args.override_reason,
            actor: args.actor,
        }),
        Command::Validate(args) => repo_protocol::run_validate(&args.config, args.json),
        Command::Init(args) => repo_protocol::run_init(&args.config, args.force),
    };

    if let Err(error) = code {
        if wants_json {
            println!(
                "{}",
                serde_json::json!({ "status": "error", "message": error.to_string() })
            );
        } else {
            eprintln!("repo-protocol: {error}");
        }
        std::process::exit(2);
    }
}
