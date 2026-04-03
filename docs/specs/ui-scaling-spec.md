# UI Scaling Spec

## Status

Implemented behavior for the current SDL3 renderer path.

Related docs:

- `docs/specs/product-spec.md`
- `docs/dev/architecture.md`
- `docs/planning/handoff-summary.md`

## Purpose

`trekr` must remain usable on displays where the natural SDL-reported display scale or an explicit user override is not an integer value.

The current implementation prioritizes:

- avoiding visibly duplicated or uneven pixel blocks at fractional scales
- preserving crisp integer scaling when interpolation is not needed
- keeping the default behavior inexpensive enough for regular interactive use

## Scope

This spec covers:

- interactive windowed and fullscreen UI scaling behavior
- the `--ui-scale` and `--ui-scaling` CLI surface
- renderer-level expectations for integer vs fractional scaling

This spec does not define a future custom scaler, font-system replacement, or shader pipeline.

## User-Facing Controls

### UI Scale Override

The app supports:

- detected display scale from SDL
- optional CLI override through `--ui-scale <number>`

Requirements:

- the effective UI scale must be at least `1.0`
- values below `1.0` are invalid
- when provided, `--ui-scale` overrides the detected display scale

### UI Scaling Mode

The app supports:

- `--ui-scaling auto`
- `--ui-scaling nearest`
- `--ui-scaling linear`

Default:

- `auto`

Requirements:

- `auto` must be the default because it preserves the cheaper/crisper direct path for integer scales and enables interpolation only when fractional scaling is present
- `nearest` must force the direct non-interpolated path
- `linear` must force the interpolated path even when the scale is an integer

## Rendering Requirements

### Effective Scale

For interactive window rendering:

- the effective UI scale is the explicit override when present, otherwise the SDL display scale
- the effective scale is clamped to a minimum of `1.0`
- the logical viewport size is derived from output pixels divided by the effective scale

### Integer-Scale Behavior

When the interactive window is using an integer scale and `--ui-scaling auto` or `nearest`:

- the app should continue using the direct SDL renderer scaling path
- pixels should remain crisp rather than filtered
- no interpolation is required

### Fractional-Scale Behavior

When the interactive window is using a fractional scale and `--ui-scaling auto`, or when `--ui-scaling linear` is selected:

- the app must render the frame to an offscreen texture at logical size
- that texture must be copied to the window using SDL linear texture scaling
- the result should avoid the visibly duplicated/uneven pixel stepping seen with direct fractional renderer scaling

### Current Sampling Model

The current SDL-based implementation supports only:

- nearest
- linear

For this spec, `linear` is the active bilinear-style option used for 2D upscaling in the current renderer path.

The current implementation does not provide:

- bicubic scaling
- Lanczos scaling
- anisotropic filtering
- text-only sampling separate from the rest of the frame

## Backend-Specific Notes

### Windowed / Fullscreen Renderer Path

Requirements:

- scaling mode selection applies to the interactive SDL renderer path used for windowed and fullscreen operation
- the app must preserve pointer/input behavior while applying the selected scaling mode

### KMSDRM Surface Console Path

The KMSDRM surface console path already uses surface scaling with linear mode when scaling the rendered frame to the window surface.

This spec does not require the KMSDRM path to expose a separate user-facing scaling mode toggle beyond the current global UI scale inputs.

### Capture UI Path

Renderer-owned screenshot capture is deterministic and renders from the app itself.

Requirements:

- capture output remains supported with the current scaling implementation
- scaling mode changes must not break screenshot generation

## Text Clarity Non-Goals

The current implementation improves fractional-scale presentation by filtering the whole frame.

It does not yet attempt to:

- render text in a separate crisp pass
- snap text to output-pixel positions independently from the rest of the UI
- introduce a custom scaler beyond SDL nearest/linear behavior

Those remain possible future improvements, but are outside the currently implemented scope.

## Acceptance Criteria

- fractional UI scales no longer show the earlier obvious duplicated-pixel artifact from direct renderer scaling
- integer scales remain available without forced filtering
- default behavior is `--ui-scaling auto`
- `--ui-scaling nearest` and `--ui-scaling linear` are accepted by the CLI
- `cargo check` and `cargo test` pass with the implemented scaling behavior
- `cargo run -- capture-ui --state-mode demo --capture-dir artifacts/screenshots` succeeds
