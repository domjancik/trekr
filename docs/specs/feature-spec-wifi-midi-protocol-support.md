# Feature Spec: Wi-Fi MIDI Protocol Support (AKAI MPC First)

## Purpose

Define a protocol-level Wi-Fi MIDI plan for `trekr`, with AKAI MPC wireless remote MIDI as the first validated target and RTP-MIDI as the primary generic network transport.

This spec is grounded in the current repo behavior and code paths in:

- `README.md`
- `docs/specs/product-spec.md`
- `docs/dev/architecture.md`
- `docs/dev/current-mappings.md`
- `docs/planning/handoff-summary.md`
- `src/midi_io.rs`
- `src/mapping.rs`
- `src/app.rs`
- `src/pages.rs`
- `src/state.rs`

## External Research Snapshot

### AKAI MPC (priority target)

Observed current vendor guidance:

- AKAI documents wireless remote MIDI control for standalone MPC hardware via an **Akai Network MIDI Driver** and MPC output port set to **Remote**.
- Setup requires MPC and host on the same network (Wi-Fi or Ethernet), then DAW-side MIDI ports labeled as Akai network/remote ports.

Source:

- https://support.akaipro.com/en/support/solutions/articles/69000867660-akai-pro-mpc-series-wireless-remote-midi-control-in-a-daw

### Generic network MIDI protocol baseline

- RTP-MIDI payload is standardized in RFC 6295.
- Apple’s MIDI Network Driver doc describes the common AppleMIDI session model on top of RTP-MIDI payloads (Bonjour advertisement, UDP control+data ports, invitation/accept handshake).
- Windows interoperability commonly uses rtpMIDI (Tobias Erichsen), compatible with Apple network MIDI.

Sources:

- https://www.rfc-editor.org/rfc/rfc6295
- https://developer.apple.com/library/archive/documentation/Audio/Conceptual/MIDINetworkDriverProtocol/MIDI/MIDI.html
- https://www.tobias-erichsen.de/software/rtpmidi.html

### Sync vs control distinction

- Ableton recommends Link for app sync use cases and virtual MIDI network for MIDI message streaming.
- `trekr` already has Link transport integration; Wi-Fi MIDI should complement it, not replace it.

Source:

- https://help.ableton.com/hc/en-us/articles/209071169-Setting-up-a-virtual-MIDI-network

## Current Baseline in Repo

- MIDI I/O currently relies on `midir` local OS-exposed ports (`src/midi_io.rs`).
- Runtime input normalization supports Note On/Off + CC, then maps to `AppAction` (`src/app.rs`, `src/mapping.rs`).
- Mapping identity currently uses `source_kind` + `source_device_label` + `source_label` (`src/mapping.rs`).
- Direct mapping already has conflict/replacement behavior and touch/desktop aware targeting (`src/app.rs`).

Implication: Wi-Fi protocol support must preserve the existing action boundary and mapping semantics.

## Goals

1. Add explicit **supported Wi-Fi MIDI protocol path** with AKAI MPC workflow first.
2. Keep keyboard/MIDI/touch/pointer convergence on `AppAction` unchanged.
3. Preserve existing mapping scopes (`Global`, `Active Track`, `Track N`, etc.).
4. Make protocol/device identity clear enough to avoid accidental cross-device conflicts.
5. Keep touch and desktop UX parity for connection and mapping flows.

## Non-Goals (this spec)

- No MIDI 2.0/UMP transport in first slice.
- No BLE-MIDI in first slice.
- No OSC transport implementation in this slice (existing OSC rows remain representational).
- No replacement of Ableton Link transport sync model.

## Supported Protocol Model

Introduce transport-level classification, starting with:

- `MidiTransportProtocol::SystemMidi` (existing `midir` ports, includes OS-created network ports)
- `MidiTransportProtocol::RtpMidiNative` (planned native RTP-MIDI client/session path)

AKAI MPC support policy:

- **Phase 1:** Supported via `SystemMidi` using Akai Network MIDI Driver-created ports (host OS handles protocol).
- **Phase 2 (optional):** Validate direct/native RTP-MIDI session interoperability where practical.

## UX Flow

### Desktop

1. User opens `MIDI I/O` page.
2. Network-capable ports are visually tagged (`NET`, `AKAI`, or `RTP`).
3. User selects default input/output as usual.
4. User routes active track input/output on `Routing` page.
5. User maps MPC controls in `Mappings` (row edit or Direct Map).

### Touch

Same flow, but tap-first:

- Tap list row to focus/select.
- Tap value cells/chips for connect/set actions.
- Direct Mapping remains tap-to-target then perform controller gesture.

### Shared behavior

- Wi-Fi vs USB must not change action semantics.
- Status/footer should show protocol-qualified source labels for troubleshooting.

## Action Model Reuse (Required)

No protocol-specific action path is allowed.

Required flow remains:

`Wi-Fi MIDI packet -> MidiInputEvent -> mapping resolution -> AppAction`

This keeps behavior aligned with architecture constraints in `docs/dev/architecture.md` and current input model in `src/app.rs`.

## Scope Behavior (Required)

Wi-Fi protocol does not alter mapping scope rules:

- Global controls remain `Global`.
- Track controls remain `Active Track` or `Track N` based on mapping row scope.
- Relative/absolute scope semantics remain unchanged.

## Conflict and Replacement Rules

Reuse existing direct-mapping intent, with protocol-aware source identity.

### Source identity normalization (new requirement)

Treat mapping source identity as:

- `source_kind`
- `transport_protocol` (new)
- `source_device_label`
- `source_label`

### Replacement rules

- Unique target+scope row: replace/update in place.
- No matching target row: create row and enable.
- Existing same source bound elsewhere: prompt/resolve via existing move-or-keep-both style behavior.

### Wildcard rule

`Any MIDI` remains wildcard at runtime, but conflict UI should show explicit protocol/device match details to avoid hidden Wi-Fi/USB ambiguity.

## Protocol-Aware Labeling

Recommended display pattern:

- Input list: `Akai Force MPC (NET)` / `Network Session 1 (RTP)`
- Mapping source badge: `MIDI[NET] CC21 Ch1 @ Akai Force MPC`

Persistence compatibility:

- Keep old fields valid.
- Add protocol metadata with backward-compatible defaults (`SystemMidi`).

## Data Model / Persistence Changes

### `MappingEntry` evolution (backward compatible)

Add optional field:

- `source_protocol: Option<String>` (or enum with serde default)

Default behavior for old states:

- missing field => `SystemMidi`

### Endpoint metadata

Add runtime descriptor for MIDI ports:

- protocol kind
- inferred network flag
- optional vendor profile (`AKAI`, `Generic RTP`, `Unknown`)

## Likely Code Touch Points

- `src/midi_io.rs`
  - introduce protocol-aware endpoint descriptors
  - (later) add native RTP-MIDI runtime path behind same event/output interface
- `src/app.rs`
  - MIDI I/O rendering tags for protocol/network
  - mapping learn/direct-map conflict detail updates
  - protocol-aware source labeling in footer/discoverability
- `src/mapping.rs`
  - protocol-aware source matching helpers
  - backward-compatible serde defaults
- `src/pages.rs`
  - optional page state for protocol filters/focus chips (if added)
- `src/state.rs`
  - persisted mapping protocol field support
- `README.md` + `docs/dev/current-mappings.md`
  - document Wi-Fi MIDI support and operator flow once implemented

## Acceptance Criteria

1. AKAI MPC wireless remote MIDI ports are usable in `trekr` via supported setup on macOS/Windows host.
2. Network-origin MIDI Note/CC events can trigger mapped `AppAction`s identically to USB MIDI.
3. Track routing and passthrough work with selected network MIDI ports.
4. Direct Mapping on desktop and touch can learn from network MIDI input.
5. Conflict/replacement behavior remains explicit when a Wi-Fi source overlaps existing bindings.
6. Existing persisted states without protocol metadata continue loading correctly.
7. Discoverability/footer labeling can distinguish network-vs-local mappings.
8. Link sync behavior remains unchanged; Wi-Fi MIDI and Link can be active together.

## Delivery Plan

### Phase 1 (recommended first)

- Support AKAI via OS-level network MIDI ports (no native RTP stack yet).
- Add protocol/network tagging + protocol-aware mapping identity fields.
- Add tests for matching/conflict with protocol metadata.

### Phase 2 (optional)

- Add native RTP-MIDI session backend.
- Add discovery/session UX for direct network endpoints when OS driver is unavailable.

### Phase 3 (future)

- Evaluate BLE-MIDI and OSC input transport unification under same protocol abstraction.

## Open Questions

1. Should native RTP-MIDI be mandatory for Linux in MVP, or is OS-level virtual port compatibility sufficient first?
2. Do we enforce one active mapping per exact protocol/device/source by default, or keep permissive multi-bind with explicit warnings only?
3. Should protocol filtering be added to Mappings page immediately, or deferred until native RTP lands?

## Notes on Inference

- AKAI public docs confirm the driver-based remote workflow and remote port selection.
- They do not explicitly publish low-level packet details in the referenced support article.
- Treat “AKAI over RTP-MIDI-compatible session model” as a practical interoperability hypothesis to validate during Phase 1 device testing, not as a guaranteed protocol claim.
