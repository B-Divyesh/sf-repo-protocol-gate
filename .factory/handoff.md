# Handoff — independent verification 2

## Status

**FAIL.** Independent QA on 2026-09-05 found 5 findings and 20 untested public
claims. See [verification-2.md](verification-2.md) for the complete evidence.

Implementation candidate `f6daaae85f0ff42852a1cf1f0654672bfb638f91`
was tested from documentation head
`ab56678f268a5e24195eeb0895956a243cfd5a7d`. The last product-code change is
`f1cc2546b338c7f7a22803fe5d640368a5a8c2e5`.

## What was verified

- Clean install, tests, formatting, lint, type check, build, audit, and Cargo
  package all pass.
- The packaged CLI installs in a clean consumer environment. Normal denial,
  allow, structured error, JSON, override boundary, automatic range, and init
  recovery paths pass.
- Every earlier verifier defect is fixed: generated evidence cannot be
  bypassed, missing evidence does not panic, automatic JSON is clean, live
  caching and framing headers are correct, and the repaired links are 46×44.
- Live desktop and 390px phone checks pass keyboard, focus, axe, reduced motion,
  200% text, privacy, offline reload, link, console, and responsive-layout
  checks.
- Live product files are byte-identical to the clean build. Lighthouse scored
  100/100/100/100 with 1.5 s LCP and 0.001 CLS.

## Remaining work

1. Add `.factory/claims.json` and one tagged sandbox test for every public
   claim; remove or narrow claims that cannot be tested.
2. Add the required real CLI demo command, bundled sample repository, terminal
   recording, `.factory/demo.md`, direct `/demo` entry, persistent sample label,
   reset, and start-real actions.
3. Replace the metaphor H1 and mood headings with the job, audience, and first
   sample action; add `.factory/copy-audit.md` and shorten the 35-word README
   sentence.
4. Implement `/privacy`, `/terms`, and a designed 404 with correct HTTP status,
   route titles, sitemap entries, navigation, and footer links.
5. Add Twitter metadata, a 1200×630 share image, an Apple touch icon, factory
   credit, and version/build text.

No product code was changed during this verification. Re-run every command in
the verification report and perform a fresh live browser check after repair and
deployment.
