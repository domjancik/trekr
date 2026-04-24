---
name: trekr-theme-authoring
description: Create or refine Trekr themes and high-contrast UI styling by editing src/theme.rs and page-owned UI files, then validating with default-versus-candidate screenshot A/B captures across timeline, mappings, MIDI I/O, and routing.
---

# Trekr Theme Authoring

Use this skill when:

- adding a new Trekr theme preset
- refining an existing theme
- fixing readability issues caused by color changes
- doing accessibility or high-contrast passes

Do not rely on palette edits alone. This skill is screenshot-driven.

## Primary files

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

Read `docs/dev/theme-authoring-procedure.md` before large theme work.

## Workflow

1. Adjust semantic theme tokens in `src/theme.rs`.
2. Move page-specific fixes to the owning page family rather than generic dumping-ground helpers.
3. Prefer `contrasting_text_color(fill, theme)` or `theme.text_on_fill(fill)` for text on filled surfaces.
4. Capture A/B screenshots:

```powershell
cargo run -- capture-ui --state-mode demo --capture-dir artifacts\archive\theme-ab-default
cargo run -- capture-ui --state-mode demo --theme <theme-name> --capture-dir artifacts\archive\theme-ab-candidate
```

5. Compare at least:
   - `timeline.png`
   - `mappings.png`
   - `midi-io.png`
   - `routing.png`
6. Fix the worst failures first:
   - white-on-white / black-on-black
   - active vs inactive collapse
   - selected vs unselected collapse
   - old-theme dangling surfaces
7. Re-capture after each meaningful pass.
8. Validate with:

```powershell
cargo check
```

Use targeted tests if needed. Full `cargo test` may still hit the repo's Windows `midir` crash.

## Theme heuristics that worked

- Use semantic accents sparingly; reserve them for meaning.
- In light themes, use near-white and mid-gray for idle states instead of pure white everywhere.
- Add borders to small chips and badges in high-contrast themes.
- Ensure tab accents survive both active and inactive backgrounds.
- For timeline content, note visibility matters more than chrome polish.
- Mute, disabled, and idle states need distinct value contrast, not just “less colorful”.

## Page-specific reminders

### Timeline

Check:

- transport chips
- track status indicators
- mute active vs inactive
- note lanes and note blocks
- recording controls
- FX bands
- stored loop slots

### Mappings

Check:

- mode/learn/direct badges
- column headers
- selected and idle rows
- enabled on/off cells
- footer tokens

### Routing

Check:

- meta badges
- unselected field titles
- toggle chips
- value chips
- affordance labels

### MIDI I/O

Check:

- selected vs idle rows
- default/selected badges
- white-on-white list failures

## Artifact policy

Use archive captures while iterating. Do not update tracked screenshots unless the change is intentional and ready to ship.
