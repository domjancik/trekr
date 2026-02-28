# trekr

Native MIDI-first tracker/player/looper for small PCs with a portable path to mobile-class targets.

## Docs

- `docs/product-spec.md`: product behavior, UX model, workflows, and MVP scope.
- `docs/architecture.md`: engine architecture, portability constraints, and stack options.
- `docs/implementation-plan.md`: milestone order, module breakdown, and delivery sequence.

## Current Direction

- Primary target: small-form-factor desktop systems.
- Secondary target: iOS/Android if the chosen stack supports it cleanly.
- V1 focus: MIDI sequencing, routing, passthrough, and loop-based recording.
- Audio follows MIDI-first V1 and should layer onto the same timeline and routing model later.
- Chosen implementation stack: Rust with a lightweight native rendering and I/O stack.
- SDL3 is built from source in the current scaffold so local builds do not depend on a preinstalled SDL runtime.

## Current Runnable Slice

`cargo run` opens a native SDL3 window with:

- fixed-fit full-song and loop-detail panes
- default vertical-time layout with tracks as side-by-side columns
- optional alternate across-time layout for comparison while prototyping
- a moving playhead
- track compaction behavior for higher track counts

Current controls:

- `Space`: toggle vertical-time columns / across-time rows
- `Escape`: quit
