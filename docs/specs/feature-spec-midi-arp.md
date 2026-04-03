# Feature Spec: MIDI Arpeggiator

## Summary

This spec defines the focused behavior of the MIDI `Arp` effect.

`Arp` is a signal-aware MIDI effect:

- it works on playback signal
- it works on live held input
- it works on cloned signal
- it respects chain order

It should stay compact in timeline/routing UI while using musical parameter labels rather than raw engine ticks.

Related docs:

- `docs/specs/feature-spec-midi-track-effects.md`
- `docs/specs/feature-spec-timeline-control-contexts.md`
- `docs/planning/handoff-summary.md`

## Core Behavior

`Arp` consumes incoming held notes and emits stepped note events over time.

It should behave consistently for:

- recorded playback material
- live input while notes are held
- cloned source signal

It should not be limited to a one-shot chord explosion model.

## Parameters

Initial parameter set:

- `Rate`
- `Order`
- `Gate`

### Rate

- shown in musical notation
- examples:
  - `1/64`
  - `1/32`
  - `1/16`
  - `1/8`
  - `1/4`
  - `1/2`
  - `1 Bar`
- raw tick counts should stay internal

### Order

Initial supported values:

- `Up`
- `Down`
- `UpDown`
- `AsPlayed`

### Gate

- note length as a percentage of the current arp step
- default `100%`

## Signal / Chain Semantics

- effects before `Arp` shape the held-note pool that reaches the arp
- `Arp` emits stepped note events from that pool
- effects after `Arp` process those emitted arp notes normally

Examples:

- `Clone -> Transpose -> Arp`
  - arp sees transposed cloned notes
- `Filter -> Arp -> Velocity`
  - filter limits the arp pool and velocity reshapes arp output

## Playback Semantics

- playback arp should respect real playback timing and loop behavior
- a held recorded chord should keep producing arp steps while the chord remains held in playback
- cloned playback signal feeding arp should behave the same as native playback signal feeding arp

## Live Semantics

- held live notes should generate arp steps while held
- held live notes should continue to generate arp steps while transport playback is stopped
- stopped-mode live arp should use a dedicated live FX clock rather than advancing song playback state
- when Link is enabled while transport is stopped, arp timing should follow Link tempo without forcing playback to start
- releasing all held notes should stop future steps and send note-off for the currently sounding arp note if needed
- `AsPlayed` should follow press order for live input
- restarting playback may reset stopped-mode arp phase

## Recording Semantics

- input-chain arp affects monitoring when `Monitor Input FX` is enabled
- input-chain arp is recorded only in `Record Post Input FX`
- output-chain arp affects playback/output only and does not alter recorded source notes

## Acceptance Criteria

- arp rate is shown with musical labels, not raw ticks
- playback chords can produce repeated arp steps over their held duration
- live held notes can produce timed arp output
- live held notes can produce timed arp output even while playback is stopped
- clone-fed arp behaves like direct source-fed arp
- chain ordering around arp remains consistent
- muting/stopping does not leave arp notes hanging
