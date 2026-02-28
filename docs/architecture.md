# Architecture And Stack Options

## Core Technical Direction

The app should be a native real-time application. Browser-based stacks are a poor fit for the lowest-end target devices because the audio path, MIDI scheduling, and always-visible dual-pane timeline need low and predictable overhead.

Core principles:

- keep audio and MIDI scheduling off the UI thread
- use fixed-size queues between real-time and non-real-time systems
- represent transport time in samples internally
- derive musical positions from the tempo map
- precompute summarized visual data for fixed-fit overview rendering

## Engine Shape

Recommended runtime structure:

- real-time engine thread for transport, playback, and scheduled MIDI output
- MIDI input thread for timestamped input capture
- UI/render thread for interaction and drawing
- background worker for project I/O and waveform/summary cache generation

Messaging between the real-time engine and UI should avoid locks in the callback path.

## Action Model

The application should be action-driven at the input boundary.

Required rule:

- keyboard, MIDI, touch, OSC, remote control, and internal automation should all map into the same action layer

That means:

- device-specific handlers translate raw events into canonical `AppAction` values
- app state changes happen by applying actions, not by embedding device logic in UI code
- transport, loop, track, and mapping behavior stay consistent across control surfaces

This is a core architectural requirement because MIDI control and remapping are central product features.

## Data Model

Core entities:

- `Project`
- `Transport`
- `Track`
- `Region`
- `LoopRegion`
- `MidiMapping`
- `DeviceRouting`

The same model should later support audio regions without replacing the MIDI-first timeline structure.

## Portability Options

### Option A: C++17 + SDL3 + miniaudio + RtMidi + custom 2D timeline renderer

Why it fits:

- strongest option for very low-end devices
- minimal runtime overhead
- broad native portability
- easy to keep the real-time path allocation-free

Suggested responsibilities:

- SDL3 for windowing, input, 2D rendering, and platform glue
- miniaudio for audio I/O
- RtMidi for MIDI I/O
- custom timeline renderer for the tracker UI

This is the best fit if Raspberry Pi Zero-class hardware remains a hard requirement.

### Option B: Rust + SDL3 bindings + CPAL + midir + custom 2D timeline renderer

Why it fits:

- memory-safety benefits
- still suitable for native real-time work
- portable across desktop and has a cleaner path to mobile-capable ecosystems

Suggested responsibilities:

- SDL3 bindings for windowing/input/render setup
- CPAL for audio I/O
- midir for MIDI I/O
- custom timeline renderer for the tracker UI

This is viable if the practical low-end target is small PCs first, with Pi-class support being desirable rather than absolute.

## Rust Performance

Rust is not inherently much lower performance than C++ for this kind of app.

In the steady-state real-time engine, Rust and C++ are usually in the same performance class when:

- allocations are kept out of the callback path
- data layouts are simple and cache-friendly
- synchronization is minimal
- rendering stays lightweight

The real differences are usually elsewhere:

- ecosystem maturity for specific low-level audio/MIDI edge cases
- build complexity and compile times
- how much control you want over every byte of the runtime

The likely gap on tiny hardware comes more from library and rendering choices than from the language itself. A Rust app with a simple native renderer can still perform very well. A heavy UI framework in either language will erase that advantage quickly.

## Chosen Stack

The project should proceed with:

- Rust
- SDL3 bindings for windowing, input, and lightweight native rendering
- CPAL for audio I/O
- midir for MIDI I/O
- custom fixed-fit timeline rendering instead of a heavy retained-mode UI framework

Reasoning:

- primary deployment is now small PCs and similar embedded desktop-class systems
- iOS/Android remains a future option worth preserving
- the app can still perform well on Orange Pi Zero 2W-class hardware and above if the renderer stays simple and the real-time path remains allocation-free

## Low-End Device Constraints

The Rust decision is acceptable only if the implementation keeps the low-end target explicit.

Required constraints:

- MIDI-first playback and rendering must stay responsive on Orange Pi Zero 2W-class devices and above
- default timeline rendering should be nearly static and summary-based, not continuously re-laid out
- track rendering should prefer precomputed summaries and cheap redraws
- no browser engine
- no heavyweight GPU-first UI framework
- no allocations or locks in the real-time callback path
- track-count ceilings may be lower on the smallest devices, but interaction latency should remain tight

Expected degradations on low-end devices are acceptable in these areas:

- lower maximum simultaneous track count
- lower waveform detail density once audio arrives
- reduced nonessential animation or visual polish

## Plugin Architecture

V1 does not need user-facing plugin hosting, but some internal interfaces should be plugin-shaped now if they reduce future migration cost.

Recommended extension points:

- MIDI processors
- instrument sources
- audio effects
- control mappings

These should begin as internal modules behind stable interfaces, not as third-party plugin loading.

That gives two benefits:

- current routing and processing code stays modular
- later expansion toward synths, effects, or DAW-like capabilities is easier

## Decision Summary

Rust is the selected implementation language.

This remains compatible with the product goals because:

- the practical low-end floor is Orange Pi Zero 2W-class hardware and above
- lower track ceilings on smaller devices are acceptable
- the primary performance requirement is snappy MIDI timing and mostly static overview rendering
- the architecture explicitly avoids the categories of framework overhead most likely to hurt low-end performance

## Ableton Link Readiness

Ableton Link is compatible with the current architecture and should be treated as a near-term sync layer, not a separate product line.

Why it fits:

- transport is already modeled independently from rendering and pages
- the action model lets external sync decisions enter through the same command path as keyboard or MIDI control
- timing already distinguishes transport state from purely visual state

Required constraints for Link integration:

- Link must attach to the global transport, not to per-track loop state directly
- per-track loops remain local track behavior layered on top of a Link-driven transport phase
- tempo, play state, and phase authority must be explicit: local, external, or shared
- Link beat time must convert into internal tick time without bypassing transport logic

Important caveat:

- Ableton Link is a tempo/phase sync system, not full arrangement-position sync

That means the practical design should be:

- Link controls shared tempo
- Link controls shared beat phase
- optional shared start/stop participation
- arrangement position and per-track loop semantics remain app-defined
