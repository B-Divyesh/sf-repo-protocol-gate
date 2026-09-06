# Verify Repo Protocol Gate before release

## Verdict: FAIL

Independent QA found **5 findings** and **7 untested public claims**. The
implementation candidate is now live and the core CLI and demo paths work, but
release acceptance requires zero findings and zero untested claims.

## Candidate and live identity

- Implementation reviewed: `8d60b1f729a7177df5847329a4ff4f2d1bc498ba`
- Documentation reviewed: `90eed532bda1c24e4b53b7b0a9122406740b01a6`
- Live URL: `https://repo-protocol-gate.sociobot.in`
- Verification date: 2026-09-06 UTC
- Clean checkout: a new clone of `origin/main`; the remote resolved to
  `90eed532bda1c24e4b53b7b0a9122406740b01a6`. The only difference from the
  implementation candidate is `.factory/handoff.md`.
- Runtime: Node 22.23.2, npm 10.9.8, rustc/Cargo 1.98.0, Git 2.43.0,
  Playwright 1.58.2 Chromium.

Fresh HTTPS returned the candidate H1, assets, and headers. Root HTML SHA-256
was `fac908a620461255d052ebef826fd3689d83df9e93a20b4e18d7296419828d01`
both locally and live. HTML, JavaScript, CSS, fonts, images, icons, terminal
asset, service worker, robots, sitemap, and both static 404 files all matched
the clean build byte for byte. The prior stale-deployment gap is resolved.

## Findings

### High — seven public claims lack complete claim tests

Every one of the 18 commands in `.factory/claims.json` exits successfully, but
source inspection shows seven declared or public claim units without the
required outcome coverage:

1. `deterministic-versioned-decisions` runs one README denial. It does not
   repeat the same input, exercise policy-version rejection, or prove the
   “without an LLM judge” part of its combined claim.
2. `json-and-exit-codes` exercises allow exit 0 and input-error exit 2, but not
   the promised denial exit 1.
3. `one-binary-cli` only looks for demo wording in `--help`; it does not assert
   that packaging or installation produces one binary.
4. `local-no-network-cli` points proxy variables at a closed port. It does not
   record attempted connections or prove the public no-telemetry and no-data-
   transfer statements.
5. `browser-demo-local` records requests only while loading `/demo`. It never
   edits or submits the form whose data the claim says is not sent away.
6. `routes-and-legal-pages` does not visit `/demo` and checks only title and H1
   on the legal routes, not their promised route content.
7. “The documentation site has no analytics or advertising scripts” appears
   on `/privacy` but has no corresponding claim entry.

These are counted as **7 untested claims**. Independent QA found no telemetry,
LLM, or cross-origin browser request, and packaging produced one executable;
those observations do not replace the required tagged regression assertions.
The two unavailable-install statements below are false claims that QA directly
tested, so they are not added to the untested count.

### Medium — two documented consumer install options are unavailable

The first command in `cli/README.md`, `cargo install repo-protocol`, failed in a
clean consumer root with exit 101: crates.io has no package named
`repo-protocol`. The top-level README also says users can download a release
binary, but the repository's GitHub releases API returned an empty list.

The live site's source-install command does work:

```text
cargo install --git https://github.com/B-Divyesh/sf-repo-protocol-gate repo-protocol
repo-protocol 0.1.0
Sample result: allowed.
```

Until registry publication or release assets exist, the unavailable paths
must be labelled as future publication steps or replaced by the working Git
install.

### Medium — 200% text clips content and creates horizontal scrolling

At a 390×844 viewport, setting root text size to 200% increased document width
from 390 to 450 CSS pixels. The H1 was visibly clipped on the right, and footer
navigation and the terminal transcript extended beyond the viewport. This
fails the attached requirement that text resize to 200% without loss.

Evidence: `/work/.evidence/live-phone-200-percent.png`.

### Medium — section links are dead on legal and client-side 404 pages

On desktop `/privacy`, selecting **What it checks** changed the URL to
`/privacy#checks` but left the privacy H1 and route content in place; `#checks`
remained hidden. **Install** behaves the same way. The links use route-relative
hashes even when the landing sections are hidden. The same defect applies to
`/terms` and the client-side missing-page route.

The Demo, Privacy, Terms, back/forward, and static 404 paths otherwise work.

### Low — demo banner controls are 36 CSS pixels high

**Reset demo** and **Start for real** measured 36 pixels high on both desktop
and phone. The required minimum interactive target is 44×44 CSS pixels. The
original 42-pixel Demo and GitHub targets from the first verification now meet
the requirement; these newly added banner controls do not.

## Job, audience, and first action

Before scrolling in fresh 1440×1000 and 390×844 contexts, the page states:

- Job: **Enforce repository rules in CI**.
- Audience: engineering teams letting coding agents change repositories.
- First action: **Try it with sample data**, with an explanation that it loads
  an approved migration and policy.

The H1, audience text, action, and three facts were all within the initial
viewport at both sizes. There was no normal-size horizontal overflow.

## Demo and browser paths

The live one-click demo passed its functional checks:

- `/demo` showed the persistent “Demo — sample data, nothing is saved” label.
- The populated sample contained `db/migrations/0042_users.sql`, its schema
  companion, `drizzle-kit`, ticket `ENG-204`, source metadata, and a matching
  hash. It reported “Change allowed” for one protected change.
- The blocked README preset denied the change. Empty input showed “Nothing to
  inspect”; malformed and unsafe paths produced an announced error with a
  recovery instruction. Space activated the valid preset from the keyboard.
- Reset restored the approved sample and kept the sample label. Start for real
  removed the sole `demo:repo-protocol-gate` session key and cleared the form.
  Local storage, IndexedDB, and cookies remained empty.
- All browser requests stayed on the product origin. No console errors or page
  errors occurred.
- An offline reload of `/demo` showed the offline notice and retained the
  allowed sample. Reduced motion matched and reduced animation to 0.01ms.
- Privacy and Terms had distinct titles, H1s, and content. An excluded missing
  asset deliberately returned HTTP 404 with the designed page, working return
  link, and zero axe violations. A missing SPA route rendered its designed
  missing-page state through the expected navigation fallback.

Screenshots are in `/work/.evidence/live-desktop-landing.png`,
`/work/.evidence/live-phone-landing.png`, and the corresponding demo-output
files.

## Declared claim commands

All commands were run individually from the clean clone. Their process result
was PASS; the incomplete assertions described above still prevent acceptance.

| Claim ID | Command result |
|---|---|
| deterministic-versioned-decisions | PASS |
| protected-change-classes | PASS |
| hash-bound-generated-artifacts | PASS |
| generator-metadata-relationships | PASS |
| line-level-denials | PASS |
| json-and-exit-codes | PASS |
| audited-overrides | PASS |
| init-preserves-policy | PASS |
| bundled-cli-demo | PASS |
| one-binary-cli | PASS |
| local-no-network-cli | PASS |
| free-mit-license | PASS |
| no-account-required | PASS |
| browser-demo-local | PASS |
| demo-reset-and-discard | PASS |
| offline-demo | PASS |
| documentation-without-javascript | PASS |
| routes-and-legal-pages | PASS |

## Clean build and installed artifact

| Check | Result |
|---|---|
| `npm ci` | PASS; 25 packages, 0 vulnerabilities |
| `npm run check` | PASS; rustfmt, Clippy with warnings denied, strict TypeScript |
| `npm test` | PASS; 5 unit, 11 CLI integration, site/axe, and 18 claim tests |
| `npm run build` | PASS; release binary and `dist/site/` produced |
| `npm audit --audit-level=high` | PASS; 0 vulnerabilities |
| `cargo package --manifest-path cli/Cargo.toml --locked` | PASS; 12 files, 14.8 KiB compressed; verification build passed |
| Every `.factory/claims.json` command | PASS individually; 18/18 |

The packaged crate installed under a separate Cargo root. `--version` returned
`0.1.0`; `--help` listed all four non-interactive commands. `demo` created a
`/tmp/repo-protocol-demo-*` Git repository and allowed the bundled hash-bound
migration. The exact live-site Git install also succeeded from a second clean
consumer root and ran the same sample.

Normal, invalid, boundary, and recovery CLI paths are covered by the clean
suite: allow and deny, missing or mismatched evidence, invalid policy version,
automatic and explicit ranges, override auditing, and init preservation. No
backend, tenant, account, API rate-limit, restart-persistence, payment, or
SQLite checks apply to this local CLI and static site. Adding AI would conflict
with the brief's deterministic no-LLM decision constraint.

## Accessibility, metadata, privacy, and performance

- `/opt/fleet/lib/verify-url.sh` passed: HTTPS 200, 983ms load, title, `lang`,
  one H1, main landmark, image alts, labelled buttons, and no console errors.
- Playwright axe found zero violations on fresh desktop, phone, and static 404
  pages. The standalone axe launcher was also attempted, but its bundled
  ChromeDriver supports Chrome 152 while the supplied Chromium is 145; this is
  an environment mismatch, not product evidence.
- Skip link was the first keyboard target. Its focus outline was a visible 3px
  saffron ring. Route focus and back navigation worked.
- Root metadata, canonical, Open Graph, Twitter, 1200×630 share image, 180px
  Apple icon, robots, sitemap, CSP/framing headers, route titles, legal pages,
  and designed 404 are present.
- Live cache headers are correct: hashed assets are immutable for one year and
  `/sw.js` is `no-cache`.
- Lighthouse mobile scored **100 performance / 100 accessibility / 100 best
  practices / 100 SEO**. FCP was 1.2s, LCP 1.6s, TBT 20ms, CLS 0.001, and total
  transfer 120 KiB.
- Built JavaScript is 9,780 bytes, CSS 17,593 bytes, fonts 72,896 bytes, and
  the 640px hero 25,370 bytes; all budgets pass.

## Earlier finding disposition

| Earlier item | Current disposition |
|---|---|
| Generated-class evidence bypass | Fixed; both missing-entry and hash-mismatch regressions pass. |
| Panic with generated class and no evidence | Fixed; structured JSON error, exit 2. |
| Automatic-range JSON pollution | Fixed; one parseable JSON document. |
| Live cache and framing policy | Fixed; candidate headers are live. |
| 42px Demo and GitHub targets | Fixed for those links; new 36px demo controls are Finding 5. |
| Missing claim registry | Partly fixed; 18 entries exist, but seven claim units remain incomplete or unlisted (Finding 1). |
| Missing CLI and browser sandbox | Fixed; installed CLI demo and isolated `/demo` work. |
| Metaphor-first first screen and missing copy audit | Fixed; job, audience, action, facts, and audit are present. |
| Missing legal, route, 404, social, and footer structure | Mostly fixed; route-relative section links remain dead on non-landing pages (Finding 4). |
| Live site served the prior August build | Fixed; live bytes match implementation `8d60b1f`. |

## Acceptance conclusion

The repaired implementation is deployed and the core policy gate, installed
demo, browser sample, privacy behavior, offline path, metadata, and performance
all work. It is not release-ready because five findings remain, including
seven public claims without complete claim coverage. Final verdict: **FAIL**.
