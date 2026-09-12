# Feature Spec: Active-Track Automatic MIDI Thru

## Summary

Auto THRU is one global routing option. When enabled, live MIDI follows the input, output port, output channel, and FX route configured on the selected active track, without requiring its manual THRU switch to be on.

## Current Behavior

THRU is currently a per-track manual switch. Changing the active track does not make its configured MIDI route live unless that track's THRU switch was enabled separately.

## Desired Behavior

- Auto THRU sends input matching only the active track's configured input port/channel.
- It sends through the active track's configured output port/channel and FX chain.
- Switching the active track immediately changes the Auto THRU route.
- Manual THRU remains available and continues to monitor any track independently of the active track.
- A `None` output remains no route; Auto THRU does not substitute the app-default output.

## Implementation

Add an app-level `auto_thru_enabled` flag and a Routing-page `Auto THRU` toggle. Live input treats a matching track as monitored when either its manual THRU flag is enabled or Auto THRU is enabled and the track is active.

## Acceptance Criteria

- With Auto THRU on and manual THRU off, incoming MIDI for active Track B sends to Track B's configured output.
- After selecting Track A, incoming MIDI for Track A sends to Track A's configured output instead.
- A non-active track does not become live solely because Auto THRU is enabled.
- Manual THRU behavior and non-THRU playback remain unchanged.

## Validation

- Focused live-input tests cover the automatic fallback and explicit-route precedence.
- Existing MIDI runtime tests cover delayed live input-FX dispatch through the same resolver.
- Run `cargo fmt`, focused tests, `cargo check`, and the full test suite.

## Deployment Notes

This is application-level route resolution only. It changes no ALSA, midir, KMSDRM, packaging, or Orange Pi service configuration. The selected output must still be visible and available on the device.
