# Current Mappings

This file is the current human-readable mapping reference for the runnable prototype.

Canonical sources of truth:
- keyboard/action bindings: `src/actions.rs`
- demo mapping overview entries: `src/mapping.rs`

Keyboard mappings currently implemented:

- `Space`: play/stop
- `R`: toggle recording
- `Shift+R`: cycle record mode (`Overdub` / `Replace`)
- `C`: clear current track
- `Shift+C`: clear all tracks
- `Home`: reset song loop to full song range
- `G`: toggle global loop
- `L`: toggle current track loop
- `[` / `]`: set current track loop start/end at playhead
- `Shift+[` / `Shift+]`: set global loop start/end at playhead
- `,` / `.`: nudge current track loop backward/forward
- `Shift+,` / `Shift+.`: nudge global loop backward/forward
- `-` / `=`: shorten/extend current track loop
- `Shift+-` / `Shift+=`: shorten/extend global loop
- `/` / `\`: half/double current track loop
- `Shift+/` / `Shift+\`: half/double global loop
- `A`: toggle arm on active track
- `M`: toggle mute on active track
- `S`: toggle solo on active track
- `I`: toggle passthrough on active track
- `Left` / `Right`: select previous/next track
- `1`-`9`: select track by absolute index
- `Tab` / `Shift+Tab`: next/previous page
- `F1` / `F2` / `F3` / `F4`: timeline / mappings / MIDI I/O / routing page
- `F5`: toggle mappings overlay
- `W`: toggle mappings page mode
- `Up` / `Down`: select current page item
- `Q` / `E`: adjust current page item
- `Enter`: activate/toggle current page item
- `Escape`: quit

Prototype demo MIDI/OSC mappings shown on the mappings page:

- `CC20` -> `Track Arm` (`Active Track`)
- `Note C2` -> `Record Hold` (`Global`)
- `CC21` -> `Track Loop` (`Active Track`)
- `CC22` -> `Track Mute` (`Active Track`)
- `/transport/play` -> `Play/Stop` (`Global`)
- `/track/active/arm` -> `Track Arm` (`Active Track`)

Notes:
- the mappings page is still a prototype overview, not the final editor
- MIDI/OSC input learn is not implemented yet
- the page shows demo MIDI/OSC entries plus the current keyboard surface
