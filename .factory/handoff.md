# Handoff — independent verification

## Status: FAIL

Tested candidate: `6d03f3753d4ad633e52143b40e752299bd1ff571`

Tested deployment: `https://repo-protocol-gate.sociobot.in/`

Date: 2026-08-28 UTC

The deployment is live and byte-identical to the candidate's production site
artifacts. Build, repository tests, static checks, packaging, browser behavior,
accessibility, privacy, offline reload, and performance are otherwise healthy.
The release is rejected because the CLI's core generated-file enforcement can
be bypassed.

## Blocking evidence

1. **High — unbound generated evidence is accepted.** A staged handwritten
   migration plus its schema companion and a valid-looking evidence document
   whose `changes` array was empty returned exit 0 / `status: allowed` when run
   with `--change-class generated`. The migration had no evidence entry and no
   bound SHA-256. With no evidence document, the same change-class path panics
   and exits 101 rather than returning the documented error code 2. See
   `.factory/verification.md` for the exact fixture and output.
2. **Medium — automatic-range JSON is polluted.** With no staged changes,
   `repo-protocol check --json` prints the resolved `HEAD^` SHA before the JSON
   object, so parsers fail. Explicit `--staged` and `--base/--head` JSON paths
   work.
3. **Medium — live caching is misconfigured.** Hashed assets and `/sw.js` all
   return `Cache-Control: public, must-revalidate, max-age=30`; the committed
   policy requests one-year immutable assets and a non-cached service worker.
4. **Low — touch targets.** The `Try it` and footer `GitHub` text links measure
   42x44 CSS px, two pixels below the 44x44 baseline.
5. **Advisory — response hardening.** HSTS, nosniff, referrer, and permissions
   headers are present; CSP and a framing restriction are absent.

## Verification completed

- Clean detached checkout at the candidate SHA; `origin/main` independently
  confirmed at the same SHA before reporting.
- `npm ci`, `npm test`, `npm run check`, exact `npm run build`, and
  `npm audit --audit-level=high` passed.
- Rust results: 5 unit tests and 6 integration tests passed; formatting,
  Clippy with warnings denied, and strict TypeScript passed.
- `cargo package --manifest-path cli/Cargo.toml --locked` passed. The 14,858-byte
  crate was installed into a clean Cargo root and the installed CLI was used
  for independent fixtures.
- Protected README denial, human allow, valid migration, missing relationship,
  artifact tamper, override boundary/audit, malformed policy, init collision,
  staged snapshot, explicit range, and automatic range paths were exercised.
- Live HTML and every shipped runtime asset matched local build hashes.
- Desktop 1440x1000 and mobile 390x844 passed visual/responsive checks; no
  horizontal overflow, console errors, page errors, or failed requests.
- Keyboard traversal, focus, skip link, empty/error recovery, 200% text,
  reduced motion, and offline reload were exercised. Axe found 0
  serious/critical issues.
- Lighthouse mobile runs: performance 96/98/99; full run 98/100/100/100.
  Median LCP 1.5 s, FCP 1.2–1.8 s, TBT 30–180 ms, CLS 0–0.001. JS 5.30 KiB,
  CSS 15.48 KiB, fonts 72.90 KiB, mobile hero 25.37 KiB.
- Privacy passed: no cross-origin requests, analytics, cookies, local/session
  storage, or IndexedDB. No API, unlock endpoint, sign-in, or backend exists,
  so rate-limiting and Entra checks are not applicable.

## Required next steps

1. Make `generated` effective only when the current artifact has a valid
   evidence entry and matching SHA; return a structured exit-2 error instead
   of panicking when evidence is absent.
2. Silence the internal `git rev-parse` probe so every `--json` mode emits one
   parseable document, then add regression tests for default range mode.
3. Correct production cache rules for hashed assets and `/sw.js` and verify
   them at the public URL.
4. Expand the two 42px text-link hit areas to at least 44px and consider CSP
   with `frame-ancestors`.
5. Re-run the full independent suite documented in
   `.factory/verification.md` before release.
