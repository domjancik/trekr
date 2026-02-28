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

## Recommended Initial Choice

Pick Option A if:

- Pi Zero-class support is a hard constraint
- the smallest runtime footprint matters more than language safety

Pick Option B if:

- small PCs are the real primary target
- mobile optionality matters
- you want stronger safety guarantees in engine code

Given the current requirements, Option B is reasonable if "from Pi Zero to laptop" is aspirational and the first actual deployments are small PCs. Option A is safer if the floor really is Pi Zero-class hardware from day one.
