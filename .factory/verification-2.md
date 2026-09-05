# Verify Repo Protocol Gate before release

## Verdict: FAIL

Independent QA found **5 findings** and **20 untested public claims**. The
repaired enforcement paths pass, and the live site now serves the repaired
static artifact and headers. Release acceptance still fails because the
required claim registry, CLI demo sandbox, plain first screen, and route
structure are missing.

## Candidate and environment

- Implementation candidate: `f6daaae85f0ff42852a1cf1f0654672bfb638f91`
- Last product-code change: `f1cc2546b338c7f7a22803fe5d640368a5a8c2e5`
- Documentation head tested: `ab56678f268a5e24195eeb0895956a243cfd5a7d`
- Prior verification report: `c0811887efc2fde20acff4eb674ef0919bd67a22`
- Live URL: `https://repo-protocol-gate.sociobot.in`
- Verification date: 2026-09-05 UTC
- Clean checkout: a new clone of `origin/main`; `origin/main` resolved to
  `ab56678f268a5e24195eeb0895956a243cfd5a7d` before testing.
- Runtime: Node 22.23.2, npm 10.9.8, rustc/Cargo 1.98.0, Git 2.43.0,
  Playwright 1.58.2 Chromium.
- Browser sizes: desktop 1440×1000 and phone 390×844, each in a fresh browser
  context. A separate phone context used reduced motion.

`ab56678` changes only `.factory/handoff.md`; `f6daaae` changes only browser
tests. The live product bytes therefore correspond to the product code in
`f1cc254` and the implementation candidate in `f6daaae`.

## Findings

### High — the required claim registry is absent

`.factory/claims.json` does not exist. There are no declared claim commands and
no `@claim:<id>` tests, so a verifier cannot run every public claim from the
required manifest. This is a release blocker even though the general test suite
independently substantiates several promises.

The cross-check found 20 distinct public claim units without a manifest entry:

1. deterministic/versioned decisions;
2. protected-path change classes;
3. exact SHA-256 artifact binding;
4. approved generator enforcement;
5. required metadata enforcement;
6. companion-change enforcement;
7. rule and configuration-line reporting;
8. no LLM judge;
9. no CLI network calls or repository-data transfer;
10. stable machine-readable JSON;
11. exit codes 0, 1, and 2;
12. explicit overrides retain denials and write an audit record;
13. `init` refuses replacement without `--force`;
14. the browser demo runs locally and sends nothing away;
15. the local demo keeps working offline;
16. setup takes fifteen minutes;
17. the product is one binary;
18. there are no accounts, telemetry, daemon, or runtime service;
19. the product is free and MIT licensed; and
20. documentation works without JavaScript.

The formal untested-claim count is therefore **20**. Claims that cannot receive
a deterministic sandbox test must be removed or narrowed.

### Medium — the required one-click CLI demo sandbox is missing

The installed artifact has neither `repo-protocol demo` nor `--demo`; both exit
2 as unknown arguments. There is no `examples/` sample repository, no
`.factory/demo.md`, and no self-hosted terminal recording of the real binary.

The browser preview also does not meet the shared demo entry contract:

- there is no first-screen action named “Try it with sample data”;
- `/demo` is not a demo route and renders the home page;
- there is no persistent “Demo — sample data, nothing is saved” label;
- there are no “Reset demo” and “Start for real” actions.

The existing preview does handle realistic states: “Valid migration” allows
one protected migration, “Orphan migration” reports the missing schema change,
empty input says “Nothing to inspect,” and `A ../escape.sql` gives recovery
guidance. Storage inspection found no cookies, local/session storage, or
IndexedDB data, so the preview did not change real data.

### Medium — the first screen does not state the job, audience, and sample action in plain words

The H1 is “Prompts suggest. Gates decide.” It is a metaphor rather than the job
and does not name the engineering-team audience. The primary action is
“Install the gate”; the required sample action and a short explanation of its
result are absent. Other visible mood/metaphor labels include “Inspection
desk,” “Put a proposed diff at the gate,” and “Bring it home.”

The mandatory `.factory/copy-audit.md` is also absent. The README audience
sentence is 35 words, above the 22-word hard cap. No banned marketing words
were found in public site or README copy.

### Medium — required routes, legal pages, and the designed 404 are not implemented

Fresh address-bar visits to `/demo`, `/privacy`, `/terms`, `/404`, and an
unknown path all return HTTP 200 with the home title and home H1. There is no
designed not-found response, route-specific title, privacy page, or terms page.
The sitemap lists only `/`. This is not a finding merely because a deliberate
404 was observed; no deliberate 404 exists.

### Low — required social metadata and footer details are incomplete

The root has a valid 54-character title, description, canonical URL, favicon,
Open Graph title/description/image, and theme color. It lacks Twitter-card
metadata and a 180px Apple touch icon. Its Open Graph image is 1280×853 rather
than the required 1200×630 share image. The footer lacks Privacy, Terms, “Built
by Param Factory,” and a version/build identifier.

## Earlier finding disposition

| Earlier item | Current evidence | Disposition |
|---|---|---|
| Generated-class bypass with an empty evidence list | `generated_class_cannot_bypass_missing_hash_bound_entry` passed; exit 1 with a hash-bound-entry denial | Fixed |
| Panic when generated class has no evidence document | `generated_class_without_evidence_is_a_structured_input_error` passed; parseable JSON, exit 2, no panic | Fixed |
| Automatic-range JSON was polluted | `automatic_range_json_is_a_single_parseable_document` passed; installed artifact output also parsed with `jq` | Fixed |
| Live asset and service-worker caching | `/assets/index-BFXemlDB.js` returned `public, max-age=31536000, immutable`; `/sw.js` returned `no-cache`; HTML revalidates | Fixed |
| 42px “Try it” and “GitHub” links | Both measured 46×44 CSS px on desktop; visible phone targets also met the effective 44px baseline | Fixed |
| No CSP or framing control | Live responses include the shipped self-only CSP, `frame-ancestors 'none'`, and `X-Frame-Options: DENY` | Fixed |

## Clean build and declared commands

| Command | Result |
|---|---|
| `npm ci` | PASS; 25 packages, 0 vulnerabilities |
| `npm install` | PASS; lockfile unchanged |
| `npm test` | PASS; 5 unit tests, 9 CLI integration tests, desktop/mobile site smoke and axe |
| `npm run check` | PASS; rustfmt, Clippy with warnings denied, strict TypeScript |
| `npm run build` | PASS; release binary and `dist/site/` produced |
| `npm audit --audit-level=high` | PASS; 0 vulnerabilities |
| `cargo package --manifest-path cli/Cargo.toml --locked` | PASS; 8 files, 58.8 KiB unpacked / 15.0 KiB compressed |
| Every command in `.factory/claims.json` | NOT RUN; the required file is missing |

The build produced 5,299 bytes of JavaScript, 15,528 bytes of CSS, 72,896
bytes of fonts, and a 25,370-byte mobile hero. All static budgets pass.

## Installed CLI verification

The packaged crate was installed under a new Cargo root. The exact website
command, `cargo install --git https://github.com/B-Divyesh/sf-repo-protocol-gate repo-protocol`,
also completed from a separate clean Cargo home at `ab56678`.

| Path | Observed result |
|---|---|
| `--version` and `--help` | `repo-protocol 0.1.0`; all three commands and non-interactive options documented |
| `init`, repeat, `--force` | exits 0, 2, 0; generated policy validates with two rules |
| Protected policy changed as agent | denial, exit 1, parseable JSON with rule and line |
| Same change as human | allow, exit 0, parseable JSON |
| Generated class without evidence | structured error, exit 2, parseable JSON |
| Default `HEAD^..HEAD` range | allow for the tested documentation commit; one parseable JSON document |
| Override reason length 11 | structured error, exit 2 |
| Override reason length 12 with actor | overridden, exit 0; original denial retained and JSON audit emitted |
| Empty/missing/mismatched generated evidence and valid generated migration | all expected paths passed in the clean integration suite |

Normal, invalid, boundary, and recovery behavior therefore passes for the real
CLI. No backend, tenant, sign-in, API rate limit, restart-persistence, payment,
or SQLite checks apply to this local single-binary product. AI is not missed
leverage: the brief explicitly requires deterministic enforcement with no LLM
in the decision path.

## Live deployment, browser, privacy, and accessibility

- The live HTML SHA-256 is
  `d8969506596ae98ea9458e4b3b1771145329cecf5306596d27bb55e686e1b351`,
  exactly matching the clean build. The HTML, JS, CSS, fonts, four hero images,
  favicon, robots file, sitemap, and service worker all match byte for byte.
- HTTP redirects to HTTPS. Root, assets, fonts, robots, sitemap, and both
  GitHub links returned 200 after redirects.
- `/opt/fleet/lib/verify-url.sh` passed: load 641 ms, no console errors, title,
  `lang=en`, one H1, main landmark, alt text, and labeled buttons present.
- Playwright axe returned zero violations at desktop and phone sizes. Lighthouse
  accessibility was 100.
- Keyboard traversal reached every visible interactive control without a trap.
  Focus used a visible 3px saffron outline with 4px offset. Enter denied the
  README sample; Space allowed the valid migration sample.
- All effective visible link/button targets were at least 44px. The 24px
  checkbox sits inside a much larger clickable label.
- The phone layout had no horizontal overflow. A 200% root text-size check kept
  the primary actions visible with no horizontal overflow.
- Reduced motion matched: scrolling became `auto` and animation/transition
  durations became 0.01 ms.
- A fresh service-worker context reloaded offline, showed the offline status,
  and allowed the valid local sample. An explicit service-worker-ready reload
  completed before disconnecting.
- Browser requests stayed on `repo-protocol-gate.sociobot.in`; there were no
  cookies, local/session storage values, IndexedDB databases, console errors,
  or page errors. The only browser persistence was the first-party offline
  cache.

Screenshots and machine output are in `/work/.evidence/`.

## Performance

One fresh Lighthouse 12.8.2 mobile run scored **100 performance / 100
accessibility / 100 best practices / 100 SEO**. FCP was 1.1 s, LCP 1.5 s, TBT
30 ms, CLS 0.001, and total transfer was 116 KiB. These measurements and the
static asset sizes pass the attached budgets.

## Acceptance conclusion

The repaired product behavior and deployment-header findings are resolved.
The release still has five contract findings and twenty untested public claims.
Final verdict: **FAIL**.
