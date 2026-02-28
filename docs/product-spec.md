# Product Spec

## Product Summary

`trekr` is a MIDI-first tracker/player/looper. It presents the full song and the active loop region at the same time, with both views fit entirely into fixed canvases by default.

The initial release focuses on:

- linear timeline sequencing
- per-track MIDI routing
- MIDI passthrough and control mapping
- loop-region recording with quantized commit on release

Audio is planned as a second-phase feature. The timeline, routing, and track model should be designed so audio clips can later overlay or accompany MIDI-driven external instruments.

## Primary Use Cases

- Sequence multiple external instruments from one transport.
- Route each track to its own MIDI channel and optionally its own output device.
- Monitor and pass through live MIDI input while playing.
- Record loops by holding record and releasing on the desired musical boundary.
- View the full arrangement and the current loop region simultaneously.

## UX Model

### Timeline Views

The app shows two synchronized timeline panes:

1. Full song view
2. Detail loop view

Default behavior:

- full song view always fits the entire song into the pane
- detail view always fits the entire selected loop region into the pane
- content stays fixed in the pane instead of scrolling in the default mode
- the playhead moves through both panes
- changing the loop region immediately updates the detail pane range

Tracks may be shown as rows or columns. If the track count becomes too high, edge tracks may compress in the non-primary axis before scrolling is introduced in a later iteration.

### Editing Model

The default mode prioritizes overview and performance over precision editing:

- select-first interaction model
- fixed-fit panes
- no required scrolling in V1
- no micro-zoom in V1

Potential later improvements:

- optional micro-zoom inside the loop pane
- optional scroll-assisted navigation for very high track counts

## Track Model

### Track Types

V1 requires MIDI tracks. The model should also reserve space for later audio or hybrid tracks:

- `midi`
- `audio` (planned)
- `hybrid` (planned)

### Track State

Each track should support:

- arm
- mute
- solo
- monitor/passthrough enable
- input device selection
- input channel filter
- output device selection
- output MIDI channel mapping

### Regions

The timeline uses linear regions rather than scene launching:

- regions are placed directly on the timeline
- loops are time ranges, not free-floating clips
- V1 does not require scene or clip-launch workflows

## Recording

### Record Modes

V1 focuses on MIDI recording:

- latch record
- hold-to-record
- overdub/replace behavior for MIDI regions

### Hold To Record

Hold-to-record behavior:

- user holds the record control to capture input
- recording begins immediately or on the current quantize rule depending on transport settings
- releasing record commits the region to the nearest quantize boundary
- the resulting region becomes part of the selected track timeline

Supported quantize targets:

- off
- pulse
- 1/16
- 1/8
- 1/4
- bar

## MIDI Features

### Input

- accept MIDI note input
- accept MIDI control input beyond notes
- allow omni or channel-specific capture
- support per-track input selection or global input capture

### Output And Routing

- route each track to a chosen MIDI output device
- route each track to a chosen MIDI channel
- support live passthrough
- support remap/filter/transpose in the passthrough path

### MIDI Control Mapping

Non-note MIDI can control:

- transport
- record
- track arm/mute/solo
- loop in/out selection
- region actions
- macro controls
- future synth/effect parameters

## Audio Planning Constraints

Audio is not first-class in V1, but the design must leave room for it:

- audio may later overlay or accompany MIDI tracks on the same timeline
- external-instrument workflows may have more MIDI tracks than audio return channels
- routing must account for grouped or shared audio returns from external gear
- the engine must eventually support instrument-control-first workflows where audio capture is secondary

## Platform Goals

Primary targets:

- Linux small PCs
- Windows small PCs
- macOS laptops/desktops

Secondary target:

- iOS/Android if supported by the chosen native stack without sacrificing the desktop real-time model

## MVP Scope

### Include

- transport
- full song overview pane
- loop detail pane
- fixed-fit default layout
- track row/column orientation toggle
- MIDI tracks
- MIDI record/overdub
- hold-to-record with nearest-quantize commit
- per-track MIDI routing
- MIDI passthrough
- MIDI learn/control mapping
- project save/load

### Exclude For Now

- plugin hosting as a user-facing feature
- audio-first workflows
- advanced piano-roll editing
- scene launching
- micro-zoom editing
- heavy scroll-based navigation
