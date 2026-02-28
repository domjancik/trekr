# Product Spec

## Product Summary

`trekr` is a MIDI-first tracker/player/looper. It presents each visible track as a paired vertical view, with a full-range column and a loop-detail column shown side by side.

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
- View each track's full arrangement and current loop region simultaneously.

## UX Model

### Timeline Views

The default view is organized as alternating track columns:

`full 1 | detail 1 | full 2 | detail 2 | ...`

Default behavior:

- each track has a full-range column
- each track has a detail column for that track's loop region
- both columns fit their full ranges into the available height
- content stays fixed in the pane instead of scrolling in the default mode
- the playhead moves downward through all visible columns
- the active track should be visually highlighted

The default timeline flow is vertical-time:

- time proceeds downward
- tracks appear as side-by-side columns
- the playhead traverses downward through those columns

Commands may target the currently active track by default. Absolute track addressing should also be supported for mappings and shortcuts. If the track count becomes too high, columns may narrow before scrolling is introduced in a later iteration.

### Editing Model

The default mode prioritizes overview and performance over precision editing:

- select-first interaction model
- fixed-fit panes
- no required scrolling in V1
- no micro-zoom in V1

Potential later improvements:

- optional micro-zoom inside the loop pane
- optional scroll-assisted navigation for very high track counts

### Utility Pages

V1 should include focused utility pages alongside the timeline view so routing and control state are inspectable without guessing.

### Mappings Page

The mappings page should show the currently configured control bindings across all supported input surfaces:

- keyboard shortcuts
- MIDI note/control mappings
- OSC address mappings

The page should make the following visible:

- input source type
- input identifier such as key, MIDI message, or OSC address
- target action
- target scope such as current track or absolute track
- conflict state if multiple bindings overlap
- enabled/disabled state

V1 may prioritize display-first with lightweight editing, but the structure should support later in-app reassignment and learn flows.

### MIDI I/O Page

The MIDI I/O page should expose device selection and availability for the full application:

- detected MIDI input devices
- detected MIDI output devices
- connection/availability state
- currently selected default inputs if global capture is enabled
- currently available outputs for track routing

This page is the operator-facing view for confirming that the hardware topology matches the live setup before editing or playback.

### Active Track Routing Page

The routing page should focus on the currently active track and surface the core per-track routing controls:

- active track identity
- selected MIDI input device
- input channel mode: all or specific channel
- selected MIDI output device
- output MIDI channel
- passthrough state

Commands should continue to work in active-track-relative form by default, while absolute track addressing remains available through mappings and future batch routing tools.

## Track Model

### Track Types

V1 requires MIDI tracks. The model should also reserve space for later audio or hybrid tracks:

- `midi`
- `audio` (planned)
- `hybrid` (planned)

### Track State

Each track should support:

- arm
- independent loop enable
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
- each track may maintain its own loop region and loop-enable state
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
- active-track selection
- track arm/mute/solo
- current-track loop enable
- absolute track-targeted actions
- loop in/out selection
- region actions
- macro controls
- future synth/effect parameters

Mappings should be representable across keyboard, MIDI, and OSC using the same action model so the mappings page can describe one unified command surface rather than separate subsystems.

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
- active-track model
- fixed-fit default layout
- fixed-fit paired track columns with `full | detail` per track
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
