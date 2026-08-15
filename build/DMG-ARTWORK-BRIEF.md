# Lighthouse — DMG background artwork brief

Target: `build/background.png` (540×380) + `build/background@2x.png` (1080×760),
combined into `build/background.tiff`. Convention: `~/.config/agent-rules/DMG.md`.

Brand pulled from `build/icon.png` and `social-preview.png` — not invented.

---

## The concept

**The beam is the arrow.**

Standard DMGs put a chevron between the app icon and the Applications folder.
Lighthouse doesn't need one: it has a beam. The lighthouse stands in the gap between
the two icons and sweeps its light rightward onto the Applications folder. The
install gesture and the product metaphor become the same image.

It's also true to what the tool does. A lighthouse doesn't move ships — it tells you
what's out there so you don't collide. That's port awareness.

## Layout map — 540×380, origin top-left

Icons are fixed by house convention at **(140, 225)** and **(400, 225)**, 100px.
Everything below is built around them.

```
 0                    270                    540
 0 ┌─────────────────────────────────────────┐
   │  NIGHT SKY — dark navy→teal, faint grid │
   │                                          │
   │              ╱▔▔╲   ← lighthouse lamp    │
   │   beam ◄────┤ ▓▓ ├────► beam sweeps      │
160│         tower rises from mist            │
   ├─────────────────────────────────────────┤
   │  ░░░ PALE MIST BAND — icons sit here ░░░ │
225│   [app icon]      ▓▓      [Applications] │  ← y=225 centers
   │    label            ▓      label          │
300├─────────────────────────────────────────┤
   │  calm dark water, no detail              │
380└─────────────────────────────────────────┘
```

**Three horizontal bands. This is the whole design.**

| Band | y range | Treatment |
|---|---|---|
| Sky | 0–160 | Dark. All the drama lives here — gradient, grid, lamp, beam |
| **Mist** | **160–300** | **Pale. Non-negotiable — see below** |
| Water | 300–380 | Dark again, calm, zero detail |

### Why the mist band is non-negotiable

Finder draws icon filename labels in the **system** label color. In Dark Mode that's
white; in **Light Mode it's near-black**. Dark text on a dark harbor is unreadable,
and you cannot control it from the artwork.

So the band from **y=160 to y=300** must be light enough for near-black text — target
**60–75% luminance**, a soft fog bank lit from within by the beam. Not white, not a
hard-edged rectangle. Feather both edges into the dark so it reads as weather, not
as a UI panel.

Everywhere the icons and their labels live is pale. Everywhere else is dark.

### Lighthouse placement

Centered at **x≈270**, base in the mist, tower rising into the sky band. That's the
gap between the two icons — it collides with neither. Tower occupies roughly
x=240–300 only; keep it narrow.

Beam sweeps **left-to-right**, brightest toward x=400 where the Applications folder
sits. A faint counter-beam left is fine (the icon has two), but the right beam must
dominate — it carries the directional cue.

---

## Prompt — paste into the generator

> Flat vector illustration, minimal geometric style. A stylized lighthouse standing
> in a calm night harbor, viewed straight-on. The lighthouse tower is steel blue
> (#2E6DA4) with a simple triangular roof and three small white windows in the lamp
> room. A warm amber (#F5A623) beam of light sweeps from the lamp toward the right
> side of the frame, widening as it goes, with a soft glow. The upper half of the
> image is a deep near-black navy night sky (#0B1418) with a subtle darker grid
> pattern, fading to dark teal. Across the middle-lower area, a soft pale fog bank
> lit warmly from within by the beam — light, hazy, muted cream and pale grey,
> feathered at the edges. Below the fog, calm dark water with no detail. Clean flat
> shapes, no texture, no gradients on the tower itself. Wide cinematic composition
> with the lighthouse centered and open empty space on both the left and right sides.

**Negative / avoid:**

> no text, no lettering, no numbers, no watermark, no logos, no arrows, no UI
> elements, no people, no boats, no birds, no rocks, no photorealism, no 3D render,
> no heavy texture, no busy detail in the lower half, no vignette

**Critical instruction to the generator:** the left and right thirds must stay
**empty** — that's where the icons land. If it fills them with rocks, boats, or
foliage, regenerate. Empty space is the requirement, not a flaw.

### Optional detail, if it comes out clean

Faint port-number buoy markers along the horizon in the **sky band only** —
`:3000`, `:5432`, `:5173` — at very low opacity, like distant harbor markers.
Matches the social preview. **Skip it if it lands below y=160.** Clutter near the
labels is worse than a missing joke.

## Palette — sampled from your own assets

| Role | Hex |
|---|---|
| Tower blue | `#2E6DA4` |
| Beam amber | `#F5A623` |
| Sky, darkest | `#0B1418` |
| Sky, teal shift | `#10202A` |
| Mist / cream | `#F7F2E3` at 60–75% |
| Status green (unused, reserve) | `#34D399` |

---

## If you're generating in Google Flow

Flow is video-first. Ask for a **still**, or pull a single frame — a moving beam
will produce motion blur that ruins a background. Per `DMG.md`, Finder cannot
animate a folder background anyway; save the motion for a first-run window.

**Generators will not output 1080×760.** Of Flow's five ratios only two are
landscape — **choose 16:9**, at the highest resolution offered.

Not because it's closest (4:3 at 1.333 is nearer the 1.421 target than 16:9 at
1.778) but because of **which direction the crop takes**:

| Source | vs target | Crop removes |
|---|---|---|
| 4:3 (1.333) | too narrow | **top and bottom** — shifts every band boundary |
| **16:9 (1.778)** | too wide | **left and right** — full height survives |

This design is calibrated on y-coordinates. The mist band has to land exactly under
the icons. A 16:9 source crops away the side margins this brief already told the
generator to leave empty, and the surplus width gives room to recenter a lighthouse
that lands off-center. 4:3 costs 6% of the height, right where it hurts.

## Post-processing

```bash
./build/make-dmg-background.sh raw-generated.png
```

Handles it end to end: center-crop to the window aspect, downscale to `@2x` and
`@1x`, build `background.tiff`, warn if the source was too small to avoid upscaling,
and sample the two label rows to check they're light enough. Defaults to 540×380;
pass `WIDTH HEIGHT` to override.

Reusable across every project — it reads the source dimensions and does the math.

Then add the `dmg` block from `~/.config/agent-rules/DMG.md` to `package.json`
(this repo currently has `"dmg": null`).

## Acceptance checks

- [ ] Sample the pixels at **(140, 285)** and **(400, 285)** — the label rows. Both
      must be **light**. This is the one check that matters
- [ ] Left third (x<190) and right third (x>350) empty above the mist
- [ ] Lighthouse confined to x≈240–300, not spilling under either icon
- [ ] Right-hand beam clearly brighter than any left-hand beam
- [ ] Exactly 1080×760 and 540×380 — no rounding
- [ ] `background.tiff` under ~500 KB
- [ ] Build it, mount it, **switch macOS between Light and Dark Mode with the DMG
      open.** Labels legible in both, or the mist band isn't pale enough
