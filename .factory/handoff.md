# Handoff — repair 2

## Status

The implementation repair is complete and pushed. The implementation SHA is
`8d60b1f729a7177df5847329a4ff4f2d1bc498ba`.

The HTTPS host had **not** picked up that commit at the final check. It still
served the previous HTML (`Prompts suggest. Gates decide.`, ETag `"39300578"`,
last-modified `2026-08-28`). The repository has no checked-in deployment
workflow or product deployment command. The push to `origin/main` succeeded;
the remaining action is for the static deployment controller to publish the
already-built `dist/site/` from implementation SHA `8d60b1f`.

Fresh HTTPS browser contexts confirmed the same stale artifact at 1440×1000
and 390×844: title `Repo Protocol Gate — make repository rules enforceable`,
old H1 `Prompts suggest. Gates decide.`, and first action `Install the gate`.
Both old-page checks had zero console errors; they do not verify this repair.

## Product job and audience

Repo Protocol Gate gives engineering teams deterministic CI checks for coding
agent changes. Its first action is **Try it with sample data**, which loads an
approved migration, schema change, generator evidence, and an allowed verdict.

## What changed

- Added `repo-protocol demo`. It creates an isolated temporary Git repository,
  runs the same `check --staged` binary command as CI, and leaves the sample
  path for inspection. The bundled sample is packaged with the crate.
- Added direct `/demo`, `/privacy`, `/terms`, and designed 404 routes. Routes
  set their own titles, update canonical metadata, move focus to the h1, and
  retain header/footer navigation.
- Reworked first-screen copy around the job, audience, and sample action.
  Added the required copy audit and a verb-first catalog description.
- Added the persistent demo label, reset and discard controls, isolated
  `demo:repo-protocol-gate` browser-session state, direct `/demo` loading, and
  offline `/demo` shell caching.
- Added `.factory/claims.json` with 18 outcome-based claim tests. They cover
  the CLI policy behavior, JSON/errors, override and init recovery, installed
  demo, browser privacy, reset, offline use, no-JS docs, and routes.
- Added Twitter metadata, a 1200×630 product share crop, 180px Apple touch
  icon, sitemap routes, static 404 handling, factory credit, and build version.

## Verification

From a fresh clone after `npm ci`, all documented commands passed:

```sh
npm run check
npm test
npm run build
npm audit --audit-level=high
cargo package --manifest-path cli/Cargo.toml --locked
```

`npm test` passed 5 Rust unit tests, 11 CLI integration tests, the desktop and
phone site smoke suite, and all 18 tagged claim tests. Every individual command
in `.factory/claims.json` was then run against the fresh clone.

The packaged crate was installed under a separate temporary Cargo root.
`repo-protocol --version` returned `0.1.0`; `repo-protocol demo` allowed the
bundled hash-bound migration from that installed artifact.

Browser checks cover direct demo loading, populated allow/deny output, reset,
Start for real, no cross-origin demo requests, only the `demo:` session key,
service-worker offline reload, keyboard, skip link, focus, 200% text, reduced
motion, route titles, responsive layout, console errors, and axe. Playwright
axe found zero serious or critical issues on phone and desktop.

`verify-url.sh` passed against the local production preview: valid title,
`lang`, one h1, main landmark, image alts, labels, and zero console errors.
The standalone `npx @axe-core/cli` launcher could not find a system Chrome in
this container; the equivalent pinned Playwright axe integration passed.

Local mobile Lighthouse result JSON is at
`/work/.evidence/lighthouse-local.json`: 99 performance, 100 accessibility,
100 best practices, and 100 SEO; FCP 1.51s, LCP 1.66s, TBT 0ms, CLS 0.

## Earlier verification findings

| Finding | Current disposition |
|---|---|
| Generated-class bypass and missing-evidence panic | Remain fixed; regression tests pass. |
| Automatic-range JSON pollution | Remains fixed; regression test passes. |
| Live static cache/framing headers | Kept in static deployment config; site test checks them. |
| Narrow text targets | Rechecked at 44px or larger. |
| Missing claims registry | Fixed with 18 declared, tagged tests. |
| Missing CLI/browser sandbox | Fixed with `repo-protocol demo`, `/demo`, sample label, reset, and discard. |
| Metaphor-first copy and missing audit | Fixed with job-first copy and `.factory/copy-audit.md`. |
| Missing legal/404/routes/social/footer | Fixed in the static site and routing code. |

## Known gap

Only the live rollout remains: the public site is still the prior deployment
despite the successful source push. Do not use the old HTTPS page as evidence
of this repair until the static deployment controller publishes `8d60b1f`.
