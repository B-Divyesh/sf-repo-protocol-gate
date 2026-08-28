# Handoff — release-blocker repair

## Status

Repair commit: `f6daaae765ef14e0e50f4f3411ef512c1f87fd0e`

The three release-blocking product defects from the independent verification
report have been repaired and covered by regressions. The repair commits are
pushed to `origin/main`:

- `f1cc254 fix: harden generated evidence enforcement`
- `f6daaae test: cover desktop and privacy smoke paths`

## What changed

1. **Generated evidence is now non-bypassable.** `generated` is a derived
   class only: a protected artifact receives it only when its own checked Git
   snapshot matches a hash-bound evidence entry. Supplying
   `--change-class generated` cannot promote a file with an empty, missing, or
   mismatched entry. A completely absent evidence document now returns the
   documented structured exit-2 error rather than panicking.
2. **Default-range JSON is clean.** The internal `git rev-parse HEAD^` probe
   now captures its output, so automatic `HEAD^..HEAD` JSON mode emits exactly
   one parseable report.
3. **Static deployment policy is expressible by the deployed platform.**
   `staticwebapp.config.json` now carries the immutable one-year `/assets/*`
   policy, `no-cache` for `/sw.js`, document revalidation, and a self-only CSP
   with `frame-ancestors 'none'` / `X-Frame-Options: DENY`. The previous
   `_headers` file remains compatible with hosts that use it.
4. **Touch targets meet the baseline.** Header and footer text links have a
   44px minimum width and height.

## Regression coverage

The CLI integration suite now proves:

- an empty evidence `changes` array cannot allow a handwritten migration even
  when the caller passes `--change-class generated`;
- absent generated evidence returns parseable JSON with exit code 2 and no
  panic;
- automatic-range `--json` output parses as one JSON document.

The browser suite now proves 44×44 targets, a desktop denial path, 390px
keyboard interactions and offline reload, desktop and mobile axe checks, no
console errors, same-origin-only browser requests, and the exact production
cache/CSP configuration copied to `dist/site`.

## Verification evidence (2026-08-28 UTC)

```sh
npm ci
npm test
npm run check
npm run build
npm audit --audit-level=high
cargo package --manifest-path cli/Cargo.toml --locked --allow-dirty
```

All passed.

- Rust: 5 unit tests + 9 CLI integration tests passed.
- Browser: Playwright ran desktop 1440×1000 and mobile 390×844 checks;
  keyboard Enter/Space, offline reload, privacy request interception, visible
  target sizing, and axe (zero serious/critical issues) passed.
- Static build: `dist/site/` produced. Shipped assets are 5.30 KB JS,
  15.53 KB CSS, and 72.89 KB fonts, all below budget.
- Type/lint: rustfmt, Clippy with warnings denied, and strict TypeScript passed.
- Security audit: `npm audit --audit-level=high` reported 0 vulnerabilities.
- Packaging: Cargo packaged and verified 8 files (58.8 KiB unpacked / 15.0 KiB
  compressed). A clean consumer install of that packaged source returned
  `repo-protocol 0.1.0` and its expected `check` help text.

## Deployment and live check

`f6daaae` was pushed to `origin/main`. The static output is ready at
`dist/site/` and includes the Azure Static Web Apps deployment configuration.

I also invoked the installed `swa` CLI with the production output. Azure
authenticated the user-assigned identity, but this container has neither a
deployment token nor an app-id mapping; the CLI stops at an interactive
"Choose your Static Web App" list. No selection was made and no credential
file remains in the repository. At the final live check, the custom domain was
still serving the prior candidate (HTML SHA-256
`584abd158e89bf47dcd64338a58ae557c2dd0727f4eb6eb66819d30cbc4aee4a` and
the old 30-second cache policy), so the live cache/identity assertion must be
re-run after the factory maps this work order to its Static Web App or the
branch deployment completes.

## Known gap / next step

The source, package, and static artifact are release-ready. The only remaining
external action is selecting the correct Static Web App in the factory
deployment context, then verify `/assets/*` returns
`Cache-Control: public, max-age=31536000, immutable`, `/sw.js` returns
`Cache-Control: no-cache`, and the public bytes match `dist/site/`.
