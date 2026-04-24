# Theme Authoring Procedure

This document captures the working procedure and concrete learnings from the high-contrast theme session so future theme work can be done confidently and repeatably.

## Scope

Use this procedure when:

- adding a new `ThemePreset`
- refining an existing theme
- doing accessibility or high-contrast passes
- fixing page-specific readability regressions after palette changes

Primary theme entry points:

- `src/theme.rs`
- `src/app/shell/ui.rs`
- `src/app_ui/branding.rs`
- `src/app/timeline/page.rs`
- `src/app/timeline/track_ui.rs`
- `src/app/timeline/recording.rs`
- `src/app/timeline/fx_ui.rs`
- `src/app/mapping/page.rs`
- `src/app/routing_ui.rs`
- `src/app/midi_io_page.rs`

## Core principle

Do not judge a theme from palette constants alone. Always validate it in rendered UI captures, page by page, state by state.

## Recommended workflow

### 1. Add or adjust theme tokens first

Start in `src/theme.rs`.

- keep theme structure semantic, not page-fragment literal
- prefer assigning meaning:
  - active/inactive chrome
  - fill vs border
  - emphasis vs muted
  - danger/record
  - positive/play
  - neutral idle
- prefer using `theme.text_on_fill(...)` or `contrasting_text_color(...)` for text on filled chips and badges

### 2. Wire theme into the owning UI family

Do not dump new color decisions into generic helpers if ownership is page-specific.

Move fixes to the owning family:

- shell chrome/tabs/footer/transport: `src/app/shell/`, `src/app_ui/branding.rs`
- timeline page chrome/status/subcolumns: `src/app/timeline/`
- mappings page: `src/app/mapping/page.rs`
- routing page: `src/app/routing_ui.rs`
- MIDI I/O page: `src/app/midi_io_page.rs`

### 3. Capture A/B screenshots early

Always capture both:

```powershell
cargo run -- capture-ui --state-mode demo --capture-dir artifacts\archive\theme-ab-default
cargo run -- capture-ui --state-mode demo --theme <theme-name> --capture-dir artifacts\archive\theme-ab-candidate
```

Use archive captures while iterating. Do not overwrite tracked screenshots unless the visual change is intentional and ready to ship.

### 4. Compare page by page

Always compare at least:

- `timeline.png`
- `mappings.png`
- `midi-io.png`
- `routing.png`

For each page, ask:

- what is active?
- what is inactive?
- what is selected?
- what is clickable?
- what is disabled?
- what is semantic accent vs mere decoration?
- what text is sitting on top of a fill?

### 5. Fix highest-severity readability failures first

Severity order used successfully in this session:

1. white-on-white or black-on-black text/icon loss
2. active and inactive states looking too similar
3. content hierarchy loss
4. lingering old-theme dark blocks inside the new theme
5. accent overuse that makes the theme feel inconsistent

### 6. Re-capture after every meaningful pass

Do not trust local reasoning after a multi-file pass. Re-capture and compare again.

### 7. Validate code

At minimum:

```powershell
cargo check
```

Use targeted tests when appropriate. In this repo, full `cargo test` can still hit a Windows `midir` crash, so theme work should rely on `cargo check`, targeted tests, and rendered screenshots.

## Concrete learnings from the high-contrast session

### 1. Pure black/white is not enough for all states

A black/white theme still needs neutrals.

Use near-white and mid-gray for:

- inactive chips
- inactive tabs
- footer idle chips
- subtle panel separation
- disabled or non-selected states

Otherwise active/inactive states collapse together.

### 2. Text on fills must be contrast-driven

Many regressions came from keeping a fixed white text color on newly light fills.

Use:

- `contrasting_text_color(fill, theme)`
- or `theme.text_on_fill(fill)`

especially for:

- transport chips
- footer tokens
- badges
- state chips
- routing controls
- mapping cells with filled backgrounds

### 3. Borders matter more in high-contrast themes

Light themes lose separation faster than dark themes.

Add or keep visible borders around:

- chips
- badges
- tabs
- routing/meta badges
- small controls

### 4. Timeline content must be tested separately from timeline chrome

The timeline has multiple layers:

- shell chrome
- transport chrome
- track status indicators
- subcolumn headers
- note lanes
- note blocks
- loop markers
- FX bands
- recording controls

A theme can look good in the shell and still fail in the actual musical content.

### 5. Timeline note visibility is the first real test

In the high-contrast pass, pale note blocks on near-white lanes made the page unreadable even though the chrome looked acceptable.

For light themes:

- note blocks should usually be dark or very strong colored fills
- lane backgrounds should be lighter than the notes
- guides should be visible but weaker than note blocks

### 6. Active vs inactive must remain distinct without relying only on color hue

Successful fixes used differences in:

- lightness/value
- border strength
- fill density
- text contrast

not just accent hue.

### 7. Accent colors should be sparse and semantic

The most workable high-contrast result used accent only where it communicated meaning:

- record/danger
- play/positive
- routing family distinctions
- mappings write/direct states
- stored loop or timeline emphasis

Avoid using accent on every surface.

### 8. Tab accents must work in both active and inactive states

A tab accent can disappear in either direction:

- black accent on active black tab
- white accent on inactive light tab

For mixed-state tabs, use a middle neutral or a hue/value that survives both backgrounds.

### 9. Mute/idle states are especially easy to collapse

Mute in high-contrast needed its own stronger neutral so active mute did not look the same as inactive mute.

When a state is intentionally subdued, check that it is still distinguishable from “off”.

### 10. Routing and mappings need separate passes

Both pages had leftover assumptions from the dark theme:

- unselected labels were too light
- chips assumed white text
- footer tokens assumed dark backgrounds
- row fills and text pairings were not contrast-safe

Do not assume one global palette tweak fixes them.

## Page-specific checklist

### Timeline

Check:

- active tab accent visible both active and inactive
- transport chips readable
- `Track All` / `Reset Song Loop` readable
- track status indicators distinct
- mute active distinct from mute inactive
- note lanes visible
- note blocks clearly visible
- guides visible but secondary
- recording controls readable
- FX bands readable
- stored loop slots readable

### Mappings

Check:

- top mode/learn/direct badges readable
- column headers readable
- selected row readable
- idle rows readable
- device/source/target/scope text readable
- enabled on/off readable
- footer tokens readable

### Routing

Check:

- top meta badges readable
- inactive field titles readable
- selected field titles readable
- value chips readable
- toggle chips readable
- adjust buttons readable
- affordance labels readable
- family panels still feel grouped

### MIDI I/O

Check:

- selected row clearly different from idle row
- default badge readable
- selected badge readable
- list body not white-on-white
- headers still have enough identity

## Shipping checklist

Before considering a theme pass done:

1. `cargo check`
2. capture default theme archive
3. capture candidate theme archive
4. visually compare all main pages
5. fix state collisions:
   - active vs inactive
   - selected vs idle
   - enabled vs disabled
   - muted vs unmuted
6. keep tracked screenshots unchanged unless the visual update is intentional
7. if intentionally updating tracked screenshots, regenerate and review them per `AGENTS.md`

## Useful commands

Run app with theme:

```powershell
cargo run -- run --theme high-contrast-light
```

Capture screenshots with theme:

```powershell
cargo run -- capture-ui --state-mode demo --theme high-contrast-light --capture-dir artifacts\archive\theme-pass
```

Default comparison capture:

```powershell
cargo run -- capture-ui --state-mode demo --capture-dir artifacts\archive\theme-pass-default
```
