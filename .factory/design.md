# Visual thesis: The customs house for code

## Direction and rationale

Repo Protocol Gate uses **surreal editorial scenery**: repository changes are
paper artifacts crossing a monumental checkpoint in an uncanny night
landscape. The metaphor makes a dry CI concern immediately legible—rules are
infrastructure, evidence is inspected, and compliant work continues—without
falling back to a generic shield, lock, dashboard, or gradient hero.

The site is intentionally single-mode. A deep ink night gives the product a
calm, forensic character and lets the generated editorial scene carry depth.
Parchment surfaces echo manifests and diffs; vermilion is reserved for the
gate and primary action; saffron marks paths that passed inspection; celadon
signals evidence and success.

## Tokens

| Role | Token | Value | Use |
|---|---|---:|---|
| Background | `--ink-950` | `#111025` | Painted page background |
| Elevated ink | `--ink-900` | `#191832` | Code wells and quiet panels |
| Surface | `--paper` | `#F3E8CE` | Primary reading fields |
| Text on ink | `--paper` | `#F3E8CE` | Body and headings |
| Muted on ink | `--mist` | `#C9C3B6` | Supporting copy (7.7:1) |
| Ink text | `--ink-950` | `#111025` | Text on paper (15+:1) |
| Accent | `--vermilion` | `#F36A4A` | Primary actions and focus |
| Accent contrast | `--ink-950` | `#111025` | Text on accent (5.4:1) |
| Success | `--celadon` | `#A8C9B8` | Allowed status |
| Warning | `--saffron` | `#F4B942` | Evidence and attention |
| Danger | `--coral` | `#FF8A72` | Denial copy on ink |

All functional combinations meet WCAG AA for normal text. Color always has a
text label or shape companion.

## Typography

- **Newsreader**, self-hosted variable serif, is the editorial display face.
  Its unusual soft curves make the policy feel authored rather than
  machine-generated.
- **IBM Plex Mono**, self-hosted, is used for commands, policy samples, labels,
  and numeric output. It connects the scene back to actual repository work.
- Body text uses the local system sans stack for speed and clarity. Minimum
  body size is 16px and prose measures 45–72 characters.
- Type scale: 16, 18, 22, 32, and `clamp(48, 8vw, 92)` px.

## Space, shape, and layout

- An 8px base rhythm with 4px only for optical alignment.
- Main gutters are `clamp(20px, 5vw, 72px)`; major sections use 96–144px on
  wide screens and 72px on phones.
- Copy sits on an editorial 12-column grid. Rules and results use long ruled
  rows, not generic card grids.
- Corners are mostly square or lightly clipped. The one pill shape is the
  live status seal, echoing an inspection stamp.
- At 390px the hero becomes a vertical cover: copy first, a short image crop,
  then actions. Decorative marginalia disappears; no task content is dropped.

## Interaction grammar

- Actions feel like stamping a document: a 2px downward press and a terse
  status label.
- The demo follows a three-part rhythm—policy, proposed changes, verdict—with
  one obvious “Inspect changes” action. Example presets are real buttons with
  labels, not mystery icons.
- Focus is a 3px saffron outline with offset against both ink and paper.
- Validation and offline messages live in an assertive status strip and always
  include what to do next.

## Motion policy

The scene enters once through small opacity and vertical transforms over
220–420ms, following reading order. Verdict rows slide from the checkpoint
edge over 180ms. Nothing loops. With `prefers-reduced-motion: reduce`, all
transforms and smooth scrolling are removed and state changes are immediate;
depth remains through scale, overlap, texture, and borders.

## Asset plan and provenance

### `site/public/assets/protocol-gate-hero.webp`

- Use case: `stylized-concept`
- Generator: `/opt/fleet/lib/gen-image.sh`
- Deployment: `factory-image` (Azure AI Foundry; recorded by adjacent
  generation metadata before optimization)
- Created: 2026-08-28 for this product; original generated asset, no external
  source material; project use under the factory's generation terms.
- Prompt: “Use case: stylized-concept. Asset type: wide landing-page editorial
  hero for a deterministic repository-policy CLI. Scene/backdrop: a dreamlike
  midnight-indigo desert made of layered paper, with distant repository
  monoliths and a pale moon. Subject: an enormous vermilion checkpoint arch
  precisely sorts floating code sheets; compliant sheets continue as orderly
  saffron ribbons while one altered migration is held at the threshold by a
  crisp geometric seal. Style/medium: sophisticated surreal editorial
  illustration, tactile cut-paper collage with subtle screenprint grain,
  strong graphic silhouettes, handmade texture, restrained dimensional
  shadows. Composition/framing: 3:2 landscape, main arch and sorting action
  concentrated on the right two-thirds, generous calm dark negative space at
  upper-left for webpage copy, clear foreground-middle-background depth.
  Lighting/mood: theatrical moonlight, vigilant but welcoming, precise and
  quietly uncanny. Color palette: midnight ink #14132B, parchment #F3E8CE,
  vermilion #F15A3C, saffron #F4B942, celadon #A8C9B8. Constraints: no words,
  no letters, no logos, no UI screenshot, no gradients, no generic
  cybersecurity shield, no photorealism, no watermark.”
- Output: generated as 1536×1024 PNG, then resized/optimized to responsive
  640, 768, 960, and 1280px WebP variants (25–100 KB; all below 300 KB).
- Alt intent: “A vermilion checkpoint in a paper night landscape sorts code
  sheets onto an orderly golden path.”

All other graphic marks are original CSS geometry or typographic characters;
there are no stock assets or third-party icon sets.
