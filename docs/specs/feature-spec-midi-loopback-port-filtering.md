# Feature Spec: MIDI Loopback Port Filtering

## Summary

Trekr-created MIDI clients such as `trekr-midi-input`, `trekr-midi-output`, `trekr-midi-inputs`, and `trekr-midi-outputs` can appear in ALSA/midir device listings while the app scans or holds live MIDI connections.

Those ports are useful as loopback plumbing for advanced workflows, especially routing between multiple Trekr instances. They should not be presented as normal hardware choices in the default MIDI I/O, Routing, or Mapping workflows.

## Problem

On Linux ALSA sequencer systems, application clients can appear alongside physical MIDI devices. Trekr currently uses stable client names for scanning and live connections, and those names may show up in the same device list as USB controllers and synths.

This creates operator-facing confusion:

- users can mistake Trekr's own ports for physical MIDI devices
- selecting a self port can create no-op routes or accidental feedback paths
- short-lived scan clients can appear/disappear during refresh, making the list look unstable
- the useful advanced loopback path is mixed into the default hardware-selection path

## Goals

- hide Trekr-owned MIDI plumbing from default user-facing device lists
- preserve an explicit advanced path for loopback and multi-instance routing
- avoid breaking existing physical MIDI device selection and refresh behavior
- make filtered/self ports explainable when diagnostics are needed
- keep the implementation based on visible port names until a stronger backend identity model exists

## Non-Goals

- removing Trekr's ALSA/midir clients
- preventing advanced users from routing between Trekr instances
- implementing a full MIDI patchbay
- solving duplicate physical device names
- changing the current `MidiPortRef` name-based persistence model

## Port Classification

Trekr should classify discovered ports into at least two visibility groups:

- `normal`: physical devices and external virtual MIDI ports that should appear in normal selection lists
- `trekr_plumbing`: Trekr-owned scan, input, output, and loopback clients that should be hidden by default

Initial name-based Trekr plumbing patterns:

- exact or prefix match for `trekr-midi-`
- case-insensitive match for `Trekr-midi-` variants reported by ALSA

The match should be narrow. A user-created external port that merely contains `trekr` somewhere in the middle should not be filtered unless it follows the Trekr-owned prefix convention.

## Proposed UX

### Default MIDI I/O

The MIDI I/O page should hide `trekr_plumbing` ports by default.

Physical controllers, synths, and user-created external virtual ports remain visible.

If every discovered port is filtered out and no normal devices exist, the page may show an empty normal list plus a compact status note such as:

```text
Trekr loopback ports hidden
```

### Advanced Loopback Visibility

Trekr should expose an explicit advanced option to show loopback/plumbing ports.

Candidate controls:

- a MIDI I/O page toggle: `Show loopback`
- a settings flag once a settings page exists
- an environment variable or CLI flag for headless/diagnostic builds, for example `TREKR_SHOW_MIDI_LOOPBACK=1`

V1 can implement only one explicit path. A page toggle is preferred for usability; an environment variable is acceptable as the smallest safe first pass.

When visible, loopback ports should be labeled or grouped so they are not confused with hardware:

```text
Loopback / Trekr plumbing
trekr-midi-input
trekr-midi-output
```

### Routing And Mapping

Routing and Mapping device selectors should follow the same visibility policy as the MIDI I/O page.

If a saved project already references a Trekr-owned port:

- keep the stored name intact
- show it as offline/hidden rather than silently replacing it
- allow it to resolve when loopback visibility is enabled and the port is present

## Implementation Notes

The current repo shape suggests a small shared helper in `src/midi_io.rs`:

- classify each scanned `MidiPortRef`
- expose filtered normal lists for default UI use
- keep raw scanned lists available for diagnostics or advanced loopback mode

Avoid duplicating filtering rules in page code. The filter should sit near `MidiDeviceCatalog::scan_internal()` or in a small helper that all catalog consumers use.

The existing automatic refresh behavior should continue to work after filtering. Refresh should not cause Trekr-owned scan clients to flicker in the normal device list.

## Acceptance Criteria

- Running one Trekr instance no longer shows its own `trekr-midi-*` ports in the default MIDI I/O lists.
- Physical USB MIDI input/output devices still appear and can be selected normally.
- Saved routes/mappings that reference hidden Trekr-owned ports are preserved by name and are not rewritten to another device.
- An explicit advanced/diagnostic path can show Trekr loopback ports when needed.
- Loopback-visible ports are labeled or grouped as Trekr plumbing.
- Device refresh does not make short-lived Trekr scan clients flicker in the default lists.
