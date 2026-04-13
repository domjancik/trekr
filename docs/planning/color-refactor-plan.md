# Color Refactor Plan

## Goal

Standardize and consolidate UI color logic so the current default palette is easier to maintain, while creating a clean path for alternate themes such as high-contrast dark/light variants.

Context:

- `docs/planning/handoff-summary.md`
- `docs/planning/implementation-plan.md`
- `docs/dev/architecture.md`

## Why This Is Worth Doing Now

The current renderer already has a recognizable visual language, but the implementation is still mostly color-by-literal:

- `src/app.rs` contains the large majority of `Color::RGB` and `Color::RGBA` calls
- `src/app_ui/branding.rs` owns a small separate branding palette
- `src/ui.rs` includes local inversion logic for text rendered over sampled backgrounds
- only a few helpers exist today, such as `stored_loop_slot_color` and `mapping_badge_palette`

This makes small visual changes expensive and risky:

- the same semantic role appears in multiple hard-coded shades
- active/inactive/selected/disabled treatments are encoded inline instead of centrally
- contrast behavior is inferred from local context instead of theme rules
- future theme work would require broad search-and-replace instead of swapping one theme definition

## Current State Summary

From the current codebase, color usage appears in a few broad groups:

1. **Foundation surfaces**
   - app background
   - cards/panels
   - borders/dividers
   - footer/overlay backgrounds

2. **Shared chrome states**
   - primary text
   - muted text
   - active tab / inactive tab
   - selected row / focused field
   - hover / discoverability emphasis
   - disabled controls

3. **Domain accents**
   - page accents (`Timeline`, `Mappings`, `MIDI I/O`, `Routing`)
   - transport chip fills
   - track-state indicators (arm, record, mute, solo)
   - mapping-source badges (key/MIDI/OSC, built-in vs user)
   - stored loop slot colors

4. **Timeline-specific rendering**
   - content background layers
   - note/region colors
   - playhead and ruler ticks
   - queued/emphasized loop markers
   - recording preview / stacked clip states

5. **Branding-specific rendering**
   - startup logo reveal colors
   - build metadata text
   - divider color

## Problems To Solve In The Refactor

### 1. Separate semantic intent from raw color values

Current code often answers "what RGB should this rectangle be?" instead of "what role is this element playing?"

### 2. Make contrast explicit

The code currently uses inversion in a few places, but high-contrast support will need predictable text/outline choices for:

- text on accent fills
- text over sampled timeline content
- subtle borders that may disappear in alternate themes
- state chips that currently rely on low-contrast tonal differences

### 3. Reduce duplicated state styling logic

Many areas independently encode variants like:

- active vs inactive
- enabled vs disabled
- selected vs unselected
- emphasized vs queued vs neutral

### 4. Keep theme work compatible with the current lightweight renderer

The renderer should stay simple and allocation-light. Theme selection should be plain data and cheap helper calls, not a heavy styling framework.

## Refactor Targets

### A. Add a dedicated theme module

Introduce a focused module such as `src/theme.rs` that owns shareable color definitions and derived styling helpers.

Recommended contents:

- `AppTheme`
- `ThemeColors` or nested groups for semantic roles
- small typed style structs for common widgets
- pure helper functions for contrast and derived states

Example shape:

- `theme.surface.app_bg`
- `theme.surface.panel`
- `theme.text.primary`
- `theme.text.muted`
- `theme.state.selected_fill`
- `theme.page.timeline_accent`
- `theme.transport.play_on`
- `theme.transport.play_off`
- `theme.mapping.badge_key_builtin`
- `theme.timeline.playhead`

The exact structure can stay pragmatic; the important part is central semantic ownership.

### B. Replace ad hoc helpers with theme-backed helpers

Current helpers should become theme-aware rather than global literals.

Likely examples:

- `stored_loop_slot_color(slot_index)` -> `theme.stored_loop_slot(slot_index)`
- `mapping_badge_palette(badge)` -> `theme.mapping_badge_palette(badge)`
- branding logo colors -> theme-backed branding palette or a dedicated branding palette object

### C. Introduce a minimal color utility layer

Small utility helpers are worth centralizing if they remain simple and deterministic.

Useful candidates:

- `rgb(...)` / `rgba(...)` constructors if they improve readability
- relative contrast/luminance helpers
- `ideal_text_on(bg)` for choosing black/white or light/dark text
- explicit high-contrast fallback helpers
- optional shade/tint helpers if used sparingly

Avoid a broad "design token math" layer unless the migration proves it necessary.

### D. Define theme-safe semantic groups first

To avoid a giant one-shot rewrite, group colors by usage domain and migrate in stages.

Recommended migration order:

1. global surfaces, borders, and text
2. page tabs and footer/overlay chrome
3. transport strip and chip styles
4. mappings and routing page controls
5. timeline track headers, markers, and recording states
6. branding/startup visuals

### E. Make high-contrast themes a first-class target

The refactor should make these future presets straightforward:

- default dark theme (current visual direction)
- high-contrast dark: black background, white foreground, restrained accents
- high-contrast light: white background, black foreground, restrained accents

That means semantic tokens should not assume:

- dark-only surfaces
- low-contrast borders
- pastel-on-dark text treatments
- text inversion as the only legibility strategy

## Proposed Module Design

### 1. Base theme data

Use plain structs with `Color` values.

Suggested pattern:

```text
AppTheme
- surface
- text
- chrome
- page
- transport
- mapping
- timeline
- branding
```

Keep the first pass explicit rather than overly generic.

### 2. Shared widget style structs

Where several renderers share the same shape, add small style structs instead of returning loose tuples.

Examples:

- `ChipStyle { fill, text }`
- `TabStyle { fill, text, border }`
- `BadgeStyle { fill, text }`
- `IndicatorStyle { fill, outline }`
- `PanelStyle { fill, border, title }`

This will remove many local `(fill, text)` and nested `if` chains from `src/app.rs`.

### 3. Theme lookup methods for stateful UI

Prefer methods that encode semantics clearly, for example:

- `theme.page_tab(page, active)`
- `theme.transport_chip(action_state)`
- `theme.track_indicator(kind, active)`
- `theme.mapping_field(selected, enabled)`
- `theme.loop_marker(emphasized, queued)`

That keeps render code focused on layout and state, not color selection trivia.

### 4. Contrast policy helpers

Add a small explicit policy for text and line visibility.

Likely helpers:

- `theme.text_on_accent(fill)`
- `theme.text_on_sampled(fill)` or `ideal_text_on(fill)`
- `theme.visible_border_against(fill)` when a border must remain visible in high-contrast themes

`draw_text_fitted_inverted` can remain for sampled timeline backgrounds, but it should stop being the main contrast strategy for normal UI chrome.

## Migration Plan

### Phase 1: Inventory and semantic naming

Deliver:

- new theme module with a default theme
- first-pass semantic palette naming
- comments documenting intended roles, not just current visuals

Exit criteria:

- new code can reference a single default theme object
- no behavior change required yet

### Phase 2: Shared chrome extraction

Move these first because they are broad and low-risk:

- app background
- panel/card fills
- panel borders/dividers
- primary/muted text
- footer and overlay chips
- page tab colors

Exit criteria:

- common shell/chrome stops using inline color literals in `src/app.rs`

### Phase 3: Reusable widget styles

Convert repeated widget patterns to theme-backed style helpers:

- transport chips
- badges
- mapping fields
- routing fields
- simple state pills/chips

Exit criteria:

- repeated inline fill/text color branching is replaced by shared style lookups

### Phase 4: Timeline palette extraction

Migrate the densest and most stateful rendering area:

- track headers
- track indicators
- loop markers
- recording overlays
- stored loop slot colors
- note/region/playhead accents

Exit criteria:

- timeline render code still reads clearly, with semantic theme calls instead of literal RGBs

### Phase 5: Branding and startup palette cleanup

Move branding into either:

- the main theme under a `branding` section, or
- a separate branding palette that still derives contrast decisions from the active theme

Exit criteria:

- branding no longer behaves as a disconnected mini-palette unless intentionally desired

### Phase 6: Alternate theme support

After the default theme migration is stable, add preset constructors:

- `AppTheme::default_dark()`
- `AppTheme::high_contrast_dark()`
- `AppTheme::high_contrast_light()`

Exit criteria:

- theme choice can be swapped at app construction time without touching renderer code

## Suggested Code Touchpoints

Primary files likely involved:

- `src/app.rs`
- `src/app_ui/branding.rs`
- `src/ui.rs`
- new `src/theme.rs`
- `src/lib.rs` or equivalent module export file

Possible follow-up if theme selection becomes persistent:

- `src/state.rs`
- persisted app settings structures
- CLI/bootstrap configuration

## Testing And Verification

### Unit tests

Add focused tests for deterministic theme behavior:

- stored loop slot palette remains stable by slot index
- mapping badge palette resolves correctly by source kind / built-in state
- contrast helper returns readable text colors for known fills
- high-contrast presets preserve expected black/white roles

### Screenshot verification

The existing screenshot flow is a good regression tool after migration.

Use:

- `powershell -ExecutionPolicy Bypass -File .\scripts\capture-ui-screens.ps1 -StateMode demo`
- `powershell -ExecutionPolicy Bypass -File .\scripts\review-ui-screens.ps1 -StateMode demo`

Focus visual review on:

- active vs inactive tab clarity
- transport chip readability
- timeline header and marker contrast
- mapping/routing selected-field hierarchy
- overlay/footer chip readability

## Refactor Guardrails

- keep the default look visually close to the current screenshots during the extraction phases
- do not couple theming to a heavyweight UI abstraction
- do not push theme logic into unrelated model modules
- prefer semantic naming over overfitting to current pages
- keep theme data cheap to copy/reference
- avoid removing useful special cases in timeline readback/inversion until semantic contrast rules fully cover them

## Recommended First Implementation Slice

The best first PR for this refactor should be intentionally narrow:

1. add `src/theme.rs` with the default theme and a few style structs
2. migrate app background, panel, border, text, and page tab colors
3. migrate footer/overlay chips
4. keep screenshots visually equivalent

That yields immediate structure without forcing the timeline and transport palette to move all at once.

## Follow-On Work After The First Slice

1. transport chip palette consolidation
2. mapping and routing widget style consolidation
3. timeline-specific palette extraction
4. branding palette cleanup
5. add high-contrast preset themes
6. optionally persist theme selection once multiple themes exist

## Definition Of Done For The Full Refactor

The color refactor is complete when:

- UI rendering no longer depends primarily on scattered inline RGB literals
- shared semantic color roles live in one theme-oriented module
- repeated widget states use common style helpers
- the default theme preserves current app readability and identity
- at least one alternate high-contrast theme can be introduced without renderer-wide rewrites
