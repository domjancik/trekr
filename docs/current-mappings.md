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
- `T`: select notes at the active-track playhead
- `Shift+T`: additive select notes at the active-track playhead
- `V`: deselect notes on the active track
- `Shift+V`: toggle the active track recording view (`Overlay` / `Stacked`)
- `J` / `K`: select previous/next note
- `Shift+J` / `Shift+K`: select previous/next committed recording clip
- `U` / `O`: focus first/last selected note
- `H` / `P`: extend note selection backward/forward
- `Y`: extend note selection on both sides
- `B`: contract note selection from the focused edge
- `Z` / `X`: nudge selected notes earlier/later
- `D` / `F`: nudge selected notes down/up
- `Shift+M`: toggle mute on the selected committed recording clip
- `Shift+Delete`: delete the selected committed recording clip
- `Left` / `Right`: select previous/next track
- `1`-`9`: select track by absolute index
- `Tab` / `Shift+Tab`: next/previous page
- `F1` / `F2` / `F3` / `F4`: timeline / mappings / MIDI I/O / routing page
- `F5`: toggle mappings overlay
- `F7`: toggle inline mapping discoverability overlay
- `F8`: toggle direct UI mapping mode
- `Shift+F8`: toggle focused-track timeline view
- `F6`: toggle Ableton Link participation
- `Shift+F6`: toggle Ableton Link start/stop sync participation
- `W`: toggle mappings page mode
- `N`: add a mapping row in mappings write mode
- `Delete`: remove the selected mapping row in mappings write mode
- `Up` / `Down`: select current page item
- `Shift+Left` / `Shift+Right`: select current mappings-editor field in write mode
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
- the mappings page now has a basic write mode with field selection and MIDI learn
- the mappings page can now add/remove rows in write mode
- MIDI learn currently captures MIDI note and CC sources
- direct UI mapping can now target supported timeline and routing controls from discoverability-backed hit targets
- direct UI mapping now captures either the next MIDI note/CC or the next keyboard keypress, including modifier combinations
- direct mapping entered from the mappings page returns there after commit, while in-place direct mapping stays on the current page so multiple controls can be mapped in sequence
- after each direct mapping commit, the mode stays armed in target-selection state until canceled so full-surface mapping is faster
- selecting a different supported control while direct mapping is awaiting input retargets the pending capture instead of requiring a cancel
- `Escape` and `F8` remain reserved to cancel direct mapping instead of being captured as mapping sources
- MIDI mappings can now execute app actions from live MIDI input
- key mappings can now execute app actions from live keyboard input before the built-in fallback bindings
- MIDI mappings can be filtered to `Any MIDI` or a specific input device name
- track-scoped mappings can now target absolute scopes like `Track 3` directly from the UI
- note-edit mapping targets now include playhead select/add, deselect, previous/next, first/last focus, extend forward/backward/both, contract, and note pitch/time nudges
- hold-capable MIDI note/CC mappings now support press/release behavior for `Record Hold` and `Select Notes At Playhead Add`
- recording-stack mapping targets now include `Recording View`, `Select Next Recording Clip`, `Select Previous Recording Clip`, `Recording Clip Mute`, and `Delete Recording Clip`
- timeline view mapping targets now also include `Focused Track View`
- stacked recording clip actions now have default keyboard bindings in addition to timeline pointer controls and the mappings system
- stacked-view note-selection actions are scoped to the currently selected recording clip
- OSC input learn is not implemented yet
- the page shows demo MIDI/OSC entries plus the current keyboard surface
