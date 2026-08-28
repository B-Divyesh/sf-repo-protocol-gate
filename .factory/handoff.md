# Handoff: Repo Protocol Gate v0.1.0

## What was built

- A Rust single-binary CLI at `cli/` with three non-interactive commands:
  `init`, `validate`, and `check`.
- Versioned `repo-protocol.yaml` rules for protected Git globs, change types,
  trusted change classes, approved generators, required metadata, and
  `changed_any_of` / `changed_all_of` relationships.
- Git range and staged-index inspection, including add/modify/delete/rename
  parsing. When no mode is supplied, the CLI checks staged changes first and
  otherwise checks `HEAD^..HEAD`.
- SHA-256-bound generator evidence read from the same Git snapshot being
  checked. An absolute `--evidence` path supports evidence emitted by a trusted
  CI generator wrapper outside the checkout.
- Human-readable denials that point to `repo-protocol.yaml:<line>`, stable JSON
  reports, and exit codes 0 (allow), 1 (deny), and 2 (input/config/Git error).
- Emergency overrides that require actor + reason, retain the original
  violations, and append a JSONL audit record or emit it to CI stderr.
- A surreal editorial static documentation site with an original generated
  checkpoint scene, responsive image variants, install/reference content, and
  a local-only live policy inspector.
- First-class allowed, denied, empty, malformed-input, keyboard, reduced-motion,
  and offline demo states. A small service worker caches the shell.
- README usage/API contract, CHANGELOG, MIT license, brief, visual thesis, asset
  provenance, caching headers, robots metadata, and sitemap.

The product is free and has no accounts, payment, analytics, telemetry, or
stored user data, so separate `/privacy` and `/terms` pages are not applicable.

## Run and verify

Requirements: Rust stable and Node.js 20+.

```sh
npm install
npm test
npm run check
npm run build
```

The exact deployment build command is `npm run build`. Static output is
`dist/site/` with `index.html` at that root. The optimized CLI is
`target/release/repo-protocol`.

Release packaging was verified with:

```sh
cargo package --manifest-path cli/Cargo.toml --allow-dirty
```

The resulting crate is `target/package/repo-protocol-0.1.0.crate`; it contained
8 files, 55.1 KiB unpacked / 14.5 KiB compressed, and Cargo's clean unpacked
verification build passed. Registry credentials are factory-owned, so nothing
was published.

## Verification results

Final local checks on 2026-08-28:

- `npm test`: passed. Rust: 5 unit tests + 6 end-to-end CLI fixture tests;
  browser: mobile interaction, keyboard, empty/error/offline states, zero
  console errors, and zero serious/critical axe findings.
- `npm run check`: passed `cargo fmt --check`, Clippy with warnings denied, and
  strict TypeScript checking.
- `npm run build`: passed and produced both the release binary and `dist/site/`.
- `npm audit --audit-level=high`: 0 vulnerabilities.
- Factory `verify-url.sh` against the production preview: HTTP 200, load 687 ms,
  title/lang/main present, exactly one h1, all images have alt text, all buttons
  are named, and no browser console errors.
- Lighthouse 12.8.2, mobile throttling against the final production preview:
  performance **99**, accessibility **100**, best practices **100**, SEO
  **100**; FCP **1.5 s**, LCP **1.7 s**, TBT **0 ms**, CLS **0**.
- Shipped initial asset sizes: JS 5.30 KiB, CSS 15.48 KiB, fonts 72.89 KiB;
  responsive hero variants 25–100 KiB. All are below the product budgets.
- Visual inspection completed at 1440×1000 and 390×844. Content stacks without
  horizontal clipping, and touch targets remain at least 44px.

## Trust boundary and known gaps

- The CLI verifies that generator evidence names an allowed generator and is
  bound to the exact checked artifact. It does not cryptographically prove who
  produced an evidence document. For adversarial pull requests, the workflow
  must create evidence in a trusted generator step outside the checkout and
  pass its absolute path; do not accept PR-authored evidence as provenance.
- `--change-class`, override identity, and the evidence path are trusted CI
  inputs and must not be assembled from untrusted pull-request content.
- Linux was the release/test platform in this work order. The implementation
  uses portable Rust and Git commands, but Windows/macOS release binaries were
  not cross-tested here.
- Deployment, registry publication, and release-binary signing remain factory
  operations; no infrastructure, DNS, billing, or registry state was changed.

## Recommended next steps

1. Build and sign Linux, macOS, and Windows binaries from the committed tag.
2. Publish the verified crate and attach binaries to the GitHub release.
3. Deploy `dist/site/` to `repo-protocol-gate.sociobot.in`.
4. In consuming repositories, keep the generator-evidence writer in a trusted
   CI workflow and upload override JSONL as a retained build artifact.
