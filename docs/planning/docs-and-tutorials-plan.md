# Docs and Tutorials Plan (Diátaxis)

_Last updated: 2026-03-13_
_Context source: `docs/planning/handoff-summary.md`, `README.md`, current `src/*` app surface, and in-progress branch `vk/156b-implement-midi-c`._

## 1) Fundamental feature inventory

### Core app and pages
- Native SDL3 app with four main pages: `Timeline`, `Mappings`, `MIDI I/O`, `Routing`.
- Overlay system:
  - Quick mappings overlay (`F5`)
  - Inline mapping discoverability overlay (`F7`)
- Focused-track timeline mode (`Shift+F8`).

### Transport, looping, and timing
- Play/stop transport.
- Record toggle with recording target priority (armed tracks first, else active track).
- Record modes: `Overdub` / `Replace`.
- Song loop controls and reset.
- Track loop controls (set start/end, nudge, resize, half/double, enable/disable).
- Quantize-aware behavior for editing and launch timing.
- Ableton Link integration:
  - Link enable/disable
  - Start/stop sync participation
  - Peer/status display in transport strip

### Recording and timeline content
- Live MIDI note-on/off capture.
- In-progress take preview on timeline.
- Loop-aware recording with wrap behavior:
  - `RecWrap Clamp`
  - `RecWrap Extend`
- Committed recording clips with ownership over notes/regions.
- Recording views per track:
  - `Overlay`
  - `Stacked`
- Stacked clip operations:
  - select clip
  - scroll clip window
  - mute selected clip
  - delete selected clip

### MIDI mapping system and discoverability
- Unified action model across built-in keyboard and user mappings.
- Mappings page modes:
  - `Read Only`
  - `Write`
- Mapping row add/remove and field editing.
- MIDI learn on selected row/source field.
- Direct UI mapping mode (`F8`) with target-first workflow.
- Direct mapping accepts MIDI note/CC and keyboard capture.
- Discoverability surfaces:
  - footer hover summaries
  - inline badges for built-in vs user-defined mappings
  - disabled mappings hidden from discoverability output

### MIDI I/O and routing
- MIDI device enumeration.
- Default input/output selection.
- Per-track routing fields (ports/channels) and passthrough toggle.
- Routed MIDI note playback and live passthrough behavior.

### Input surfaces and interaction
- Keyboard-first action surface.
- Mouse/touch support for non-timeline editing chrome:
  - tabs
  - transport chips
  - mapping fields/rows
  - MIDI I/O rows
  - routing rows and passthrough
  - recording clip controls in stacked view
- Timeline note/region pointer editing is not yet implemented.

### CLI, state, and automation
- CLI subcommands:
  - `run`
  - `capture-ui`
  - `commands`
  - `help`
- Launch state selection: `demo`, `empty`, persisted, and custom state file.
- Video mode selection: `windowed`, `fullscreen`, `kmsdrm-console`.
- Deterministic screenshot generation via `capture-ui` and scripts.
- Xtask bootstrap/check/run flow including `vendor/ableton-link` submodule initialization.

## 2) In-progress feature: `vk/156b-implement-midi-c`

### Current status (from branch history)
- Branch: `vk/156b-implement-midi-c`
- Feature theme: **MIDI controller banks workflow**
- Notable commits:
  - `feat: add controller bank mapping model and runtime wiring`
  - `feat: implement mappings-page midi controller banks workflow`
  - `fix: make ctrl+n create visible controller bank group`

### Expected product/doc impact
- Mappings docs will need a dedicated section for controller-bank concepts and workflow.
- How-to guidance should include:
  - creating a bank group
  - assigning mappings into banks
  - switching/using banks in performance
- Reference docs will need exact field definitions and constraints for bank ownership/assignment.
- Tutorials should add one independent short flow for “build a first controller bank”.
- Screenshot set likely needs at least one additional mappings-focused shot showing bank group visibility and selected bank state.

## 3) Documentation IA and page plan (Diátaxis)

Every page below is classified in exactly one Diátaxis mode.

| Proposed page | Mode | Expected content | Screenshot request |
|---|---|---|---|
| `docs/tutorials/first-launch-and-first-playback.md` | Tutorial | Launch, page navigation, transport play/stop, active track basics | Use `timeline.png` in demo state; transport chips, playhead, active track must be clearly readable |
| `docs/tutorials/record-your-first-midi-loop.md` | Tutorial | Arm/record/stop/replay one loop and understand commit result | New shot: timeline with visible committed clip region + notes + record mode state |
| `docs/tutorials/store-and-recall-loops.md` | Tutorial | Save loop slot, toggle quantized recall, trigger recall timing | New shot: stored slot indicators and launch quantize chip clearly visible |
| `docs/tutorials/create-your-first-mapping.md` | Tutorial | Write mode, MIDI learn, direct mapping target flow | Use `mappings.png` + `mappings-overlay.png`; selected row/field and discoverability badges clear |
| `docs/tutorials/controller-banks-quickstart.md` | Tutorial | (vk/156b) Create a controller bank group and map first controls | New shot: mappings page with visible bank group, selected bank, and mapped rows |
| `docs/how-to/configure-midi-input-output.md` | How-to | Select default MIDI devices and verify signal path | Use `midi-io.png`; selected defaults and list focus must be obvious |
| `docs/how-to/route-a-track-and-enable-passthrough.md` | How-to | Set input/output/channel and passthrough on a track | Use `routing.png`; value field and passthrough state must be obvious |
| `docs/how-to/use-stacked-recording-clips-live.md` | How-to | Switch to stacked view, select/scroll/mute/delete clip safely | New shot: stacked lanes + selected clip + MUT/DEL controls |
| `docs/how-to/build-a-performance-mapping-layer.md` | How-to | Practical workflow for rapid direct mapping across timeline/routing | New shot: direct-mapping armed target highlight and post-commit state |
| `docs/how-to/launch-in-demo-empty-or-persisted-state.md` | How-to | Command recipes for predictable startup states | No screenshot needed |
| `docs/how-to/capture-and-review-ui-screens.md` | How-to | Run capture/review scripts and validate expected outputs | Optional shot: folder output summary table |
| `docs/reference/cli.md` | Reference | Full CLI command/flag reference including `capture-ui` options | No screenshot needed |
| `docs/reference/controls-and-default-keymap.md` | Reference | Canonical keyboard actions grouped by page/domain | Optional compact cheat-sheet graphic |
| `docs/reference/page-timeline.md` | Reference | Timeline widgets, chips, indicators, state labels | Use `timeline.png`; annotate control names/areas |
| `docs/reference/page-mappings.md` | Reference | Fields, modes, learn states, direct mapping rules, bank fields (vk/156b) | Use `mappings.png` and new bank-state shot |
| `docs/reference/page-midi-io.md` | Reference | MIDI I/O list behavior, defaults, and refresh expectations | Use `midi-io.png` |
| `docs/reference/page-routing.md` | Reference | Routing field semantics and passthrough behavior | Use `routing.png` |
| `docs/reference/state-model.md` | Reference | Terms and data entities: track, loop, clip, slot, mapping, bank | No screenshot needed |
| `docs/explanation/timeline-loop-and-clip-mental-model.md` | Explanation | Conceptual model of song loop vs track loop vs recording clip ownership | Diagram request: boundary/ownership figure |
| `docs/explanation/why-mappings-and-discoverability-are-action-based.md` | Explanation | Design rationale for unified actions and overlays | Use `mappings-overlay.png` |
| `docs/explanation/record-wrap-and-quantize-behavior.md` | Explanation | Why clamp/extend exists and how quantize affects outcomes | New shot: transport chips with RecWrap + quantize status |
| `docs/explanation/link-transport-synchronization.md` | Explanation | Conceptual Link behavior and expected user mental model | New shot: Link + Sync + peer count status |

## 4) Short onboarding videos (30s–1m, independent, logical order)

1. **First sound in 45s**  
   Goal: launch app, pick track context, play/stop.  
   Visual requirements: timeline transport and active track highlight readable.

2. **Record and commit in 60s**  
   Goal: arm, record, stop, replay committed take.  
   Visual requirements: recording state transition and committed clip visibility.

3. **Stored loops in 45s**  
   Goal: store and recall a slot, with quantized launch on/off behavior.  
   Visual requirements: slot markers + launch quantize state.

4. **Map one control in 60s**  
   Goal: create one mapping via MIDI learn and one via direct UI mapping.  
   Visual requirements: selected mapping row/field and direct-map target highlight.

5. **Route MIDI in 45s**  
   Goal: select I/O defaults and set per-track routing + passthrough.  
   Visual requirements: clear MIDI I/O selection and routing value changes.

6. **Controller banks quickstart in 60s** *(vk/156b)*  
   Goal: create a bank group and assign mappings into a bank.  
   Visual requirements: visible bank group creation and selected bank context.

## 5) Screenshot capability spec additions (for auto-generated doc shots)

Current built-in capture is strong for full-page deterministic screenshots. To fulfill planned docs/tutorial visuals, add these optional capabilities:

### A. Deterministic interaction script before capture
- Need: capture specific UI states (e.g., selected clip, direct mapping armed, specific field focused).
- Proposal:
  - `cargo run -- capture-ui --script artifacts/capture-scripts/<name>.json --capture-dir ...`
- Script primitives:
  - `show_page`
  - `send_action`
  - `click(x,y|named_target)`
  - `wait_frames`
  - `set_state_override`

### B. Named region capture (crop from full frame)
- Need: docs often require focused images (transport strip, mapping row, bank panel).
- Proposal:
  - `--capture-region <region-id>` for predefined UI regions
  - `--capture-rect x,y,w,h` for ad hoc crop
- Region ids should include: `timeline_transport`, `timeline_track_header_active`, `mappings_selected_row`, `mappings_bank_panel`, `routing_active_row`, `midi_io_selected_list`.

### C. Annotated capture mode
- Need: onboarding pages and reference docs benefit from callouts.
- Proposal:
  - `--annotate overlays.json`
- Overlay support:
  - box + label
  - arrow + label
  - highlight tint by region id

### D. Multi-shot sequence output (for tutorials/video storyboards)
- Need: consistent “step 1/2/3” stills per flow.
- Proposal:
  - `--sequence <script>` outputting numbered frames and metadata.
- Metadata should include step label, command args, and state hash.

### E. Stable target IDs exported from UI layer
- Need: avoid brittle pixel-based interaction in scripts.
- Proposal:
  - expose hit-target IDs for major controls (transport chips, mapping fields, bank controls, clip controls).
  - scripting can reference `target_id` rather than coordinates.

### F. Capture manifest for docs pipeline
- Need: CI-friendly proof that required docs images were produced.
- Proposal:
  - emit `artifacts/screenshots/manifest.json` with file list, dimensions, state mode, optional region id, app commit hash.

## 6) Suggested immediate implementation order

1. Create docs skeleton folders/pages by Diátaxis mode.
2. Ship `controller-banks-quickstart` tutorial once `vk/156b` merges.
3. Add capture scripting + named regions first (highest leverage for docs automation).
4. Add annotation overlays and sequence output.
5. Add capture manifest and CI assertion for required doc images.


