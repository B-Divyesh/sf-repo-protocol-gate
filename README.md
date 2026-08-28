# Repo Protocol Gate

Repo Protocol Gate turns repository instructions into deterministic checks that
run before an agent-authored change can merge. It protects paths, verifies
hash-bound generator evidence, requires metadata, and enforces companion
changes without an LLM in the decision path.

It is for engineering teams that let coding agents modify application
repositories and need rules such as “humans own the README” and “migrations
must come from Drizzle after the schema changes” to hold in CI.

## Install

Download a release binary for your platform, or build from source:

```sh
cargo install --path cli
repo-protocol --help
```

The crate starts at `0.1.0`. It has no telemetry, network calls, daemon, or
runtime service.

## Usage

Create `repo-protocol.yaml`:

```yaml
version: 1
default_change_class: agent

rules:
  - id: humans-own-readme
    description: Agents must not rewrite the project contract.
    paths: [README.md]
    allow:
      change_classes: [human]

  - id: generated-migrations
    description: Migrations follow a reviewed schema change.
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
  audit_log: -
```

Validate the policy, then check a commit range:

```sh
repo-protocol validate
repo-protocol check --base origin/main --head HEAD
```

To check the Git index before committing:

```sh
repo-protocol check --staged
```

Generated changes carry evidence in `.repo-protocol/evidence.json`. Each
listed artifact is bound to its exact content; editing it after generation
invalidates the evidence.

```json
{
  "version": 1,
  "generator": "drizzle-kit",
  "metadata": {
    "ticket": "ENG-204",
    "source": "db/schema/users.ts"
  },
  "changes": [
    {
      "path": "db/migrations/0042_add_users.sql",
      "sha256": "87f754b197f55fe3d4ef65d56901c06e03c1c56a74179ea503b3fce2f99c2a8a"
    }
  ]
}
```

`--json` returns a stable, machine-readable report. Results use exit code `0`
for allow, `1` for policy denial, and `2` for invalid input, configuration, or
Git state.

```sh
repo-protocol check --staged --json
```

An emergency override is explicit and leaves an audit record. Both fields are
required when the policy says so:

```sh
repo-protocol check --staged \
  --override-reason "Restore production during INC-481" \
  --actor "oncall@example.com"
```

By default the record is written to stderr (`audit_log: -`) for CI capture.
Set `audit_log` to a file path if your workflow uploads that file as an
artifact. Overrides never hide the original denials.

Run `repo-protocol init` to write a commented starter policy. It refuses to
overwrite an existing policy unless `--force` is supplied.

### GitHub Actions

```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0
- run: repo-protocol check --base "${{ github.event.pull_request.base.sha }}" --head HEAD
  env:
    REPO_PROTOCOL_CHANGE_CLASS: agent
```

`REPO_PROTOCOL_CHANGE_CLASS` is an explicit trust input. Set it from trusted
workflow context, not pull-request content. Evidence can promote only its
hash-bound files to the `generated` class.

## Policy reference

- `paths` and relationship values are Git-style globs. A bare directory-like
  pattern such as `docs/**` matches descendants.
- `change_types` accepts `added`, `modified`, `deleted`, or `renamed` and
  defaults to all four.
- `allow.change_classes` is required. Common classes are `agent`, `human`, and
  `generated`, but teams may define their own names.
- `allow.generators` applies to `generated` evidence and is always checked
  when present.
- `require.metadata` requires non-empty keys in the evidence document.
- `require.changed_any_of` requires at least one companion change;
  `changed_all_of` requires a match for every listed pattern.
- Rules compose: every matching rule must allow the change.

## Develop and verify

Requirements: Rust stable and Node.js 20+.

```sh
npm install
npm test
npm run build
```

The static docs build lands in `dist/site/`; the release binary lands in
`target/release/repo-protocol`. Create the publishable crate archive with:

```sh
cargo package --manifest-path cli/Cargo.toml
```

Run the docs locally with `npm run dev`. The live documentation is at
https://repo-protocol-gate.sociobot.in.

## Privacy and security

Repo Protocol Gate runs locally, reads only the configured Git diff and policy
evidence, and sends nothing over the network. The documentation demo runs
entirely in the browser and stores nothing. Treat change class flags and
override actor values as trusted CI inputs.

## License

MIT. See [LICENSE](LICENSE).
