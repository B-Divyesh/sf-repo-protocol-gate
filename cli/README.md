# repo-protocol

The single-binary CLI for [Repo Protocol Gate](https://repo-protocol-gate.sociobot.in).

```sh
cargo install repo-protocol
repo-protocol init
repo-protocol check --staged
repo-protocol demo
```

See the repository README for the complete policy format, generator evidence,
CI usage, exit codes, and audited override workflow. The CLI has no telemetry
and makes no network requests.

`repo-protocol demo` creates a temporary sample Git repository, runs the real
`check --staged` command on its approved Drizzle migration, and prints the
folder path for inspection.
