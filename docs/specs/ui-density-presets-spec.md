# UI Density Presets Spec

## Status

Planned behavior. This spec defines the next density-control layer that will sit alongside the existing theme and UI scaling systems.

Related docs:

- `README.md`
- `docs/README.md`
- `docs/specs/product-spec.md`
- `docs/specs/ui-scaling-spec.md`
- `docs/dev/architecture.md`
- `docs/dev/theme-authoring-procedure.md`
- `docs/planning/implementation-plan.md`
- `docs/planning/handoff-summary.md`

## Purpose

`trekr` already supports:

- color/theme selection through `ThemePreset`
- renderer scaling through `--ui-scale` and `--ui-scaling`
- a fixed-fit UI strategy intended for small-form-factor and mobile-class targets

The current code still hardcodes many layout constants directly in render and hit-test code, for example:

- `src/ui.rs` surface gutters and strip gaps
- page-level `inset_rect(...)` values such as `24`, `12`, and `10`
- page header heights such as `28` and `48`
- page-specific row gaps, chip paddings, and border-adjacent spacing

This makes it hard to intentionally tune the app for space-constrained devices without scattering device-specific changes across pages.

This spec introduces explicit density presets so frame border size, panel padding, chip spacing, and related layout metrics can be controlled centrally without mixing those concerns into the color theme definitions.

## Goals

- Add a density system that is theme-adjacent but structurally separate from `ThemePreset`.
- Introduce first-class presets:
  - `default`
  - `compact`
  - `touch`
  - `tiny`
- Make density app-global rather than page-local.
- Reuse the existing app/action architecture so density selection can later be surfaced through keyboard, MIDI mapping, pointer, and touch like other durable controls.
- Keep layout behavior coherent by changing related metrics together instead of exposing only one-off border-width knobs.
- Preserve the product direction from `docs/specs/product-spec.md` and `docs/dev/architecture.md`: fixed-fit layout, low overhead, and good behavior on small hardware.

## Non-Goals

This spec does not require:

- per-page or per-widget freeform user resizing
- arbitrary numeric sliders for every spacing token in V1
- separate density per page
- density being stored inside the color theme definitions
- replacing the existing `--ui-scale` or `--ui-scaling` systems
- a new font engine or custom text rasterizer

## Design Summary

The app should treat visual configuration as three parallel layers:

1. `ThemePreset`
   - controls color/styling semantics
2. `UiDensityPreset`
   - controls layout compactness and touch target sizing
3. UI scale / scaling mode
   - controls logical-to-output scaling and interpolation behavior

In short:

- theme answers "what colors and semantic visual roles are used?"
- density answers "how much room does the UI consume?"
- UI scale answers "how large is the rendered logical UI on the display?"

## Preset Model

### Presets

The app should expose these density presets:

- `Default`
  - current general desktop baseline
- `Compact`
  - denser than default while preserving comfortable desktop use
- `Touch`
  - more generous hit targets and spacing for pointerless or finger-led use
- `Tiny`
  - aggressively space-saving mode for very constrained displays

### Core Principle

Density is a coordinated preset, not a single border-width value.

When density changes, the app should adjust a coherent set of metrics together, including at minimum:

- outer surface gutter
- page-frame inset
- page/frame border thickness
- top-strip/header heights
- inter-panel gaps
- row gaps
- chip/button minimum heights
- inline badge padding
- affordance hit-box padding
- optional per-page minimum target sizes

The app may also include density-owned text spacing decisions where needed, but theme-owned color semantics must remain in `ThemePreset`.

## Proposed Data Model

Add a dedicated density layer near `src/theme.rs`, for example:

- `src/ui_density.rs`
- or `src/layout_metrics.rs`

Recommended public types:

- `UiDensityPreset`
- `UiMetrics`

Recommended shape:

```rust
pub enum UiDensityPreset {
    Default,
    Compact,
    Touch,
    Tiny,
}

pub struct UiMetrics {
    pub surface_gutter_px: i32,
    pub page_inset_x_px: i32,
    pub page_inset_y_px: i32,
    pub frame_border_px: u32,
    pub tabs_height_px: u32,
    pub page_gap_px: i32,
    pub panel_gap_px: i32,
    pub row_gap_px: i32,
    pub chip_min_height_px: u32,
    pub touch_target_min_px: u32,
    // optional page-specific metric groups as needed
}
```

The exact field list may evolve, but the ownership rule should remain stable:

- colors live in `Theme`
- spacing/sizing live in `UiMetrics`

## UX Flow

### Initial Control Surface

The first implementation should support density selection through launch/config surfaces before building in-app settings UI.

Required first entry points:

- CLI argument, for example `--ui-density <default|compact|touch|tiny>`
- optional environment variable fallback, for example `TREKR_UI_DENSITY`
- deterministic screenshot capture path should accept the same density input

This mirrors the current theme and UI scale pattern:

- `--theme ...`
- `--ui-scale ...`
- `--ui-scaling ...`

### Future In-App Control

After the CLI-backed implementation is stable, density should become a normal app-level control that can be changed from inside the app.

Preferred future UX:

- expose density on a settings-style surface or utility page, not as a hidden debug-only toggle
- represent it as a discrete preset cycle/select control, not free numeric entry
- when changed in-app, apply immediately to all pages
- show a short status confirmation naming the selected preset

## Action Model Reuse

The repository already centers durable behavior on `AppAction`, with keyboard, MIDI, pointer, and touch converging on that layer.

Density control should follow the same rule.

### Required Reuse Pattern

Future in-app density changes should resolve through canonical actions such as:

- `AppAction::CycleUiDensity`
- or `AppAction::SetUiDensity(UiDensityPreset)`

This keeps density aligned with the current product direction:

- keyboard can trigger it directly
- mappings can bind to it later
- pointer/touch affordances can call the same action
- status messaging and persistence can stay centralized

### V1 Scope Rule

The first implementation may be CLI/config-only, but it should still be built around a central app field and setter so later `AppAction` integration is straightforward.

Recommended app-level shape:

- `ui_density_preset: UiDensityPreset`
- `fn ui_metrics(&self) -> &'static UiMetrics`
- `fn set_ui_density_preset(&mut self, preset: UiDensityPreset)`

## Scope Behavior

Density is app-global.

### Required Scope Rules

- One active density preset applies to the entire rendered app.
- Density does not vary by page.
- Density does not vary by track.
- Density does not vary by theme preset.
- Capture UI uses the same density rules as interactive run mode.
- Pointer hit-testing must use the same density-derived bounds as rendering.

### Rationale

Current rendering and hit-testing are tightly paired in page helpers such as:

- `handle_pointer_down(...)`
- `page_frame_layout(...)`
- page-specific rectangle builders in timeline, mappings, MIDI I/O, routing, and direct mapping code

Allowing page-local density would create mismatch risk between:

- rendered control size
- pointer/touch hit area
- discoverability/direct-mapping highlight placement

So the first density system should remain global and deterministic.

## Conflict and Replacement Rules

Density control needs explicit precedence rules so it behaves predictably beside theme, scaling, persistence, and capture flows.

### Precedence

Recommended precedence order:

1. explicit CLI argument
2. explicit in-app user selection during the current session
3. environment variable default
4. application default preset

If persisted session-level UI preferences are added later, the intended stable order should be:

1. explicit CLI argument
2. explicit in-app current-session override
3. persisted user preference
4. environment variable default
5. application default preset

### Replacement Rule

Selecting a density preset replaces the entire active metrics bundle.

It must not:

- merge old and new metric values
- keep page-specific leftovers from the previous preset
- apply only frame border width while leaving old spacing metrics behind

### Theme Interaction Rule

Changing theme must not implicitly change density.

Changing density must not implicitly change theme.

The only allowed coupling is that pages may choose different visual treatments because the available rectangle sizes changed.

### UI Scale Interaction Rule

`--ui-scale` and density solve different problems and must both remain available:

- density changes logical spacing/layout metrics
- UI scale changes output scaling of the logical frame

A user may legitimately run:

- `--ui-density tiny --ui-scale 2.0`
- `--ui-density touch --ui-scale 1.0`

These combinations must be treated as valid rather than contradictory.

## Desktop vs Touch Behavior

Density presets are available on all platforms, but expected defaults differ by interaction mode.

### Desktop Expectations

- `Default` is the general desktop baseline.
- `Compact` should be the main denser desktop mode.
- `Tiny` is allowed on desktop for very constrained displays, but readability and hit-area regressions are expected risks and must be validated carefully.
- Mouse/pointer precision allows smaller controls than touch, so desktop can tolerate denser presets.

### Touch Expectations

- `Touch` is the preferred touch-first density.
- `Touch` should preserve larger minimum interactive targets for tabs, transport chips, routing toggles, mapping rows/fields, and timeline header affordances.
- `Tiny` must still work on touch-capable devices when explicitly selected, but it is not the recommended default for finger-led interaction.
- Touch presentation must avoid relying on hover-only recovery for cramped controls.

### Shared Rule

Touch vs desktop should change recommended defaults and validation thresholds, not the meaning of the preset names.

For example:

- `Touch` means the same metrics bundle everywhere
- it may simply be the default on a touch-first device profile later

## Page-Level Behavioral Requirements

Density presets must preserve the current product shape:

- fixed-fit paired timeline columns
- page shell for `Timeline`, `Mappings`, `MIDI I/O`, and `Routing`
- footer/discoverability/direct-mapping UI
- action-driven navigation and selection

### Timeline

Density changes must preserve:

- readable track-pair structure
- visible transport strip grouping
- stable active-track hierarchy
- clickable/tappable transport chips and timeline header controls
- discoverability and direct-mapping target rectangles that still match the rendered controls

`Tiny` may reduce whitespace and border weight, but it must not silently make key transport or track-header controls unselectable.

### Mappings

Density changes must preserve:

- readable row selection state
- field boundaries in write mode
- learn/direct-map chips remaining activatable
- lookup panel readability when opened

### MIDI I/O

Density changes must preserve:

- distinct list-row selection
- readable default/offline badges
- pointer/touch row hit areas matching the visible rows

### Routing

Density changes must preserve:

- grouping of `Signal`, `Input FX`, and `Output FX`
- clickable/tappable value areas and toggles
- readable compact FX field layout

## Implementation Strategy

### Phase 1: Introduce central metrics without visible behavior change

- Add `UiDensityPreset` and `UiMetrics`.
- Create a `Default` preset that reproduces current layout closely.
- Route existing layout helpers through metrics accessors.
- Keep screenshots as close as practical to the current baseline.

### Phase 2: Replace hardcoded literals in shared helpers and page layouts

Start with the most repeated values and layout chokepoints:

- `src/ui.rs`
- shell/page-frame helpers
- page-level insets and top strips
- row and panel spacing helpers

### Phase 3: Add additional presets

- `Compact`
- `Touch`
- `Tiny`

Tune them against the tracked screenshot set and touch/pointer behavior.

### Phase 4: Expose runtime control

- CLI support first
- later app action and UI control surface
- optional persistence after behavior is stable

## Acceptance Criteria

### Architectural

- The codebase has a dedicated density preset type separate from `ThemePreset`.
- The app resolves rendering metrics from a central density-owned structure.
- Core page layout no longer depends on scattered magic numbers for frame/border/inset sizing where density control is intended.

### Behavior

- The app supports the presets `default`, `compact`, `touch`, and `tiny`.
- One selected preset applies globally across all pages.
- Theme selection and density selection work independently.
- UI scale and density selection work independently.
- Pointer/touch hit-testing remains aligned with rendered control bounds after density changes.
- Direct mapping/discoverability overlays remain aligned with the controls they describe.

### UX

- `Default` remains close to the current general desktop baseline.
- `Compact` visibly reduces whitespace while preserving normal desktop usability.
- `Touch` visibly enlarges key interactive targets compared with `Default`.
- `Tiny` visibly reduces frame borders and related spacing further than `Compact`.
- The tracked main pages remain usable in all four presets:
  - Timeline
  - Mappings
  - MIDI I/O
  - Routing

### Validation

Implementation should be considered complete only when all of the following succeed:

- `cargo check`
- relevant layout and CLI tests
- `cargo run -- capture-ui --state-mode demo --capture-dir artifacts/screenshots` with default density
- renderer-owned screenshot review for at least:
  - default
  - compact
  - touch
  - tiny

## Likely Code Touch Points

### `src/theme.rs`

- Keep color theme ownership here.
- Do not move density into the existing theme structs.
- Potentially colocate or neighbor the new density preset definitions for discoverability.

### `src/ui.rs`

Current shared layout helpers include hardcoded values such as surface gutters and strip gaps.
This is a primary density integration point for:

- `surface_rect(...)`
- `inset_rect(...)`
- `split_top_strip(...)`
- other reusable row/column helpers where density-sensitive spacing exists

### `src/app/mod.rs`

Likely additions:

- app field for the selected density preset
- getter for active metrics
- setter / future action integration
- bootstrap summary updates
- persistence/load wiring if density later becomes persistent

### `src/cli.rs`

Likely changes:

- parse `--ui-density <default|compact|touch|tiny>`
- include density in help text
- include density in suggested command reconstruction
- add tests matching current `--theme` and `--ui-scale` coverage

### Page-owned UI modules

Likely files:

- `src/app/midi_io_page.rs`
- `src/app/routing_ui.rs`
- timeline page/layout files under `src/app/`
- direct mapping and discoverability UI helpers

These currently use local literal insets, gaps, and row heights that should become metrics-driven.

### Pointer / touch hit testing

Likely files:

- `src/app/input.rs`
- page-specific pointer handlers
- direct mapping target builders

These must be updated together with rendering so the interaction rectangles remain correct.

### State and persistence

Likely files:

- `src/state.rs`
- app startup/load/save wiring

Persistence is optional for the first implementation, but these will matter if density becomes a durable user preference.

### Documentation

Likely docs to update when implementation lands:

- `README.md`
- `docs/README.md`
- `docs/specs/ui-scaling-spec.md`
- `docs/dev/theme-authoring-procedure.md`
- `docs/planning/handoff-summary.md`

## Open Questions

- Should persisted density be part of app state, or remain launch/config driven until an in-app settings surface exists?
- Should touch-capable device profiles eventually default to `Touch`, or should all platforms keep `Default` unless explicitly overridden?
- Should `Tiny` reduce only spacing/borders, or should it also tighten selected text placements and badge paddings more aggressively than other presets?
- Should density become directly mappable in V1 runtime UI, or only after a broader settings/control surface exists?
