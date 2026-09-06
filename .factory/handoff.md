# Handoff — independent verification 3

## Status

**FAIL.** Independent QA found 5 findings and 7 untested public claims. The
implementation reviewed is `8d60b1f729a7177df5847329a4ff4f2d1bc498ba`;
the documentation head reviewed is
`90eed532bda1c24e4b53b7b0a9122406740b01a6`.

The previously pending live rollout is complete. Fresh phone and desktop
contexts at `https://repo-protocol-gate.sociobot.in` served bytes identical to
the clean candidate build.

## What QA verified

- Ran `npm ci`, `npm run check`, `npm test`, `npm run build`, npm audit, Cargo
  packaging, and all 18 declared claim commands individually from a fresh
  clone. Every command passed.
- Installed the packaged crate into a clean Cargo root, then ran version,
  help, and the bundled demo. Repeated the live site's exact Git install in a
  second clean consumer root; it worked and allowed the sample migration.
- Verified the live job-first landing page, realistic one-click sample, sample
  label, reset, discard, normal/empty/invalid/recovery states, keyboard, route
  focus, legal pages, designed 404, offline reload, reduced motion, browser
  privacy, console, axe, metadata, headers, cache policy, and build parity.
- Lighthouse mobile scored 100 in performance, accessibility, best practices,
  and SEO. FCP was 1.2s, LCP 1.6s, TBT 20ms, and CLS 0.001.

## Findings to repair

1. Seven claim units have incomplete tagged tests or no claim entry.
2. `cargo install repo-protocol` is unavailable and GitHub has no downloadable
   release, although both options are documented.
3. At 200% text size the 390px layout becomes 450px wide and clips content.
4. What-it-checks and Install header links are dead on legal/client 404 routes.
5. Reset demo and Start for real are 36px high instead of at least 44px.

Full evidence and exact claim gaps are in `.factory/verification-3.md`.
Machine evidence is under `/work/.evidence/`.

## How to reproduce

```sh
npm ci
npm run check
npm test
npm run build
npm audit --audit-level=high
cargo package --manifest-path cli/Cargo.toml --locked
```

For consumer verification, install from the packaged crate or use the working
site command:

```sh
cargo install --git https://github.com/B-Divyesh/sf-repo-protocol-gate repo-protocol
repo-protocol demo
```

Do not declare PASS until all five findings are repaired and every public claim
has one complete tagged test.
