# Independent product verification — Repo Protocol Gate

## Verdict: FAIL

Candidate `6d03f3753d4ad633e52143b40e752299bd1ff571` was tested on
2026-08-28 against `https://repo-protocol-gate.sociobot.in/`. The site is live
and byte-identical to the candidate build, and the repository's automated
quality gates pass. The candidate nevertheless fails the core enforcement
contract: a caller-supplied `generated` change class can allow a migration that
has no hash-bound evidence entry.

## Scope and environment

- Source: `origin/main` at `6d03f3753d4ad633e52143b40e752299bd1ff571`.
  `git ls-remote origin refs/heads/main` returned the same SHA before this
  report was written.
- Clean-checkout method: detached worktree created directly from the candidate;
  it had no tracked modifications before or after install/build/test.
- Runtime: Node `v22.23.2`, npm `10.9.8`, rustc `1.98.0`, Cargo `1.98.0`, Git
  `2.43.0`, Chromium supplied for Playwright `1.58.2`.
- Browser sizes: desktop `1440x1000`; mobile `390x844`, including a separate
  reduced-motion run.
- No product source was modified during verification.

## Defects

### High — generated-artifact evidence can be bypassed

The documented invariant says generated artifacts are promoted to the
`generated` class only by evidence bound to the exact file SHA-256. In a clean
fixture, QA staged:

- a handwritten `db/migrations/0042_users.sql`;
- the required `db/schema/users.ts` companion;
- an evidence document with generator `drizzle-kit`, valid required metadata,
  and `"changes": []` (no entry or hash for the migration).

With a policy allowing only `generated` changes and the approved generator:

```text
repo-protocol check --staged --change-class generated --json
exit: 0
status: allowed
changes_checked: 3
protected_changes: 1
violations: []
```

The same fixture without `--change-class generated` correctly exits 1. The
implementation uses the ordinary class whenever file evidence is invalid, so
the caller-supplied string becomes `generated`; it then consults document-level
generator and metadata without requiring an evidence entry for the protected
file (`cli/src/lib.rs:657-704`). This defeats the central hash-binding promise
for workflows that set the documented trusted change-class input.

The same code path with no evidence document panics at `cli/src/lib.rs:682` and
exits 101:

```text
thread 'main' panicked at src/lib.rs:682:41:
valid evidence has a document
generated_no_evidence_exit=101
```

That also violates the documented exit-code contract of 0 allow, 1 deny, and 2
input/configuration/Git error.

### Medium — default `--json` output is not valid JSON

In a repository with no staged changes and at least two commits, the documented
automatic `HEAD^..HEAD` mode leaks `git rev-parse --verify HEAD^` to stdout
before the report (`cli/src/lib.rs:490`, `522-527`). Example:

```text
repo-protocol check --change-class human --json
e7a660074f727b416238388b49bfd8bcbd834e42
{
  "status": "allowed",
  ...
}
```

Piping this output to `jq -e .` failed with `Invalid numeric literal`; when the
consumer closed the pipe, the CLI also panicked on the broken stdout pipe. This
breaks the promised single stable JSON report in the normal post-commit mode.
Explicit `--staged --json` and `--base ... --head ... --json` were valid JSON.

### Medium — production cache policy does not match the shipped policy

The candidate ships a rule requesting
`Cache-Control: public, max-age=31536000, immutable` for `/assets/*` and
`no-cache` for `/sw.js`. Fresh live responses instead returned the same policy
for HTML, hashed assets, and the service worker:

```text
Cache-Control: public, must-revalidate, max-age=30
```

Conditional requests do return 304, and service-worker update/offline behavior
works, but immutable caching for content-hashed assets is absent in production.

### Low — two text-link hit targets are 42 CSS px wide

At the tested layouts, the `Try it` navigation link (desktop) and `GitHub`
footer link were `42x44` CSS px. This narrowly misses the attached `44x44`
touch-target baseline. The 24px checkbox is wrapped by a much larger clickable
label and was not counted as an effective-target failure.

### Advisory — no CSP or framing policy

Production sends HSTS, `X-Content-Type-Options`, `Referrer-Policy`, and a
restrictive `Permissions-Policy`. It does not send a Content Security Policy or
an equivalent `frame-ancestors`/`X-Frame-Options` control. The current product
is a data-free static site, so this is recorded as defense-in-depth rather than
a release blocker.

## Clean build and repository gates

All commands were run from the detached candidate worktree after `npm ci`:

| Check | Result |
|---|---|
| `npm ci` | PASS; 25 packages installed, 0 audit vulnerabilities |
| `npm test` | PASS; 5 Rust unit + 6 CLI integration tests, site Playwright/axe smoke |
| `npm run check` | PASS; rustfmt, Clippy with `-D warnings`, strict TypeScript |
| `npm run build` | PASS; release binary and `dist/site/` produced |
| `npm audit --audit-level=high` | PASS; 0 vulnerabilities |
| `cargo package --manifest-path cli/Cargo.toml --locked` | PASS; 8 files, 14,858-byte crate; Cargo verification build passed |

The release binary is 2,167,488 bytes. Its normal runtime dependency tree has
no HTTP client or telemetry library.

## Packaged CLI and end-to-end behavior

The generated crate was installed into a new Cargo root from
`target/package/repo-protocol-0.1.0`; `repo-protocol --version` returned `0.1.0`
and `--help` described all three commands and non-interactive use.

Independent fixtures produced these results:

| Scenario | Expected | Observed |
|---|---:|---:|
| Agent changes protected README | deny / 1 | deny / 1, rule and config line included |
| Human changes protected README | allow / 0 | allow / 0 |
| Valid hash-bound migration with any/all companions | allow / 0 | allow / 0 |
| Required `changed_all_of` companion absent | deny / 1 | deny / 1 with missing path |
| Staged artifact edited after evidence hash | deny / 1 | deny / 1 with expected and actual hashes |
| Unstaged working-tree edit while checking index | evaluate staged snapshot | PASS; prior staged artifact remained allowed |
| Invalid policy version 99 in JSON mode | error / 2 | error / 2 with parseable JSON |
| Override reason length 11 (minimum 12) | error / 2 | error / 2 |
| Override reason length 12 with actor | override / 0 | override / 0; original denial retained; one JSONL audit record |
| `init`, repeat without force, then `--force` | 0, 2, 0 | 0, 2, 0; resulting policy validated |
| Explicit commit range | enforce range | deny / 1 as agent; allow / 0 as human |
| Automatic previous-commit mode | enforce `HEAD^..HEAD` | enforcement correct, but stdout polluted as described above |

This confirms the ordinary protected-path, relationship, hash-mismatch,
override, version, staged-snapshot, and commit-range paths. The high-severity
generated-class case prevents acceptance.

## Live deployment identity and browser QA

- Root HTML SHA-256 was identical locally and live:
  `584abd158e89bf47dcd64338a58ae557c2dd0727f4eb6eb66819d30cbc4aee4a`.
- The built JS, CSS, both font files, all four responsive hero images, service
  worker, favicon, robots file, and sitemap were fetched from production and
  each matched the candidate byte-for-byte. The main JS hash was
  `f2a5b31c080e33fbec36f7ad0c52627fdd52b44c02189b5849b3d15b77fc2ec8`.
- HTTP redirects to HTTPS. Root and all required assets returned 200; the site
  uses Brotli where requested. Navigation fallback returns the app shell for
  unknown routes.
- Desktop and 390px visual inspection showed intentional responsive stacking,
  readable content, no clipped task content, and zero horizontal overflow.
- Semantic smoke: correct title and `lang=en`, exactly one `h1` and one `main`,
  meaningful image alt, ordered headings, and labeled controls.
- Keyboard-only traversal reached every visible control. Enter/Space activated
  presets and submit actions, the skip link was first, and the designed 3px
  saffron focus ring was visible. Empty input produced `Nothing to inspect`;
  malformed and unsafe paths produced an announced `Input needs attention`
  message with recovery guidance.
- At 200% root text size on the 390px layout there was no horizontal overflow
  and the primary action remained visible.
- Axe: 0 serious or critical findings on desktop and mobile.
- Console/page/request errors: 0 / 0 / 0.
- Reduced motion: media query matched; smooth scrolling became `auto`, and
  animation/transition duration dropped to `0.01ms`.
- Privacy: all observed requests stayed on the product origin; there were no
  analytics calls, cookies, local/session storage values, or IndexedDB
  databases. The only browser persistence was the first-party offline cache.
- Service worker: registration and explicit update completed; an offline reload
  rendered the full shell, displayed the offline notice, and the local demo
  still denied the README example.
- Sign-in and rate-limit checks are not applicable: the product has no account,
  unlock flow, API, or server-side endpoint. Browser traffic consisted only of
  static document/assets and the service worker.

## Performance and budgets

Three fresh Lighthouse 12.8.2 mobile runs scored 96, 98, and 99 for performance.
The full-category run scored **98 performance / 100 accessibility / 100 best
practices / 100 SEO**. Across the runs LCP was 2.6 s, 1.5 s, and 1.5 s (median
1.5 s); FCP was 1.2–1.8 s; TBT was 30–180 ms; CLS was 0–0.001. One cold run
crossed the 2.5 s LCP target by 0.1 s, but the median and two repeat runs passed.
There is no field-data INP measurement; Lighthouse TBT remained at or below
180 ms.

Shipped uncompressed budget values all pass:

- JavaScript: 5,299 bytes (budget 200 KB)
- CSS: 15,476 bytes (budget 50 KB)
- Fonts: 72,896 bytes total (budget 120 KB)
- Mobile hero: 25,370 bytes (budget 300 KB)
- Lighthouse transferred bytes: 118,857 bytes across 7 requests

## Acceptance conclusion

The deployed documentation and most CLI behavior are polished and verifiable,
but the candidate must not ship as an enforcement gate until generated status
requires valid evidence for the exact protected artifact and every JSON mode
emits only one JSON document. Production caching should also be corrected and
rechecked. Final result: **FAIL**.
