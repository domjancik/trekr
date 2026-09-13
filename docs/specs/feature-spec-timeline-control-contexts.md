# Timeline FX: fixed slots and parameter inspector

Status: implemented design, 13 September 2026. This supersedes the earlier horizontal tiles, per-effect rows, and paginated FX field cycle.

## Layout

Each track retains Input FX above its timeline and Output FX below it. Each band contains four static square effect slots in a 2×2 grid on the left and a stationary 2×2 parameter grid on the right. Both grids use reading order: top-left, top-right, bottom-left, bottom-right. Slot order is processing order; empty slots are skipped by the engine.

Slots never collapse or resize when effects are added or removed. Band heights remain 48 logical pixels at Default, 42 Compact, 36 Tiny, and 56 Touch. A distinct seven-pixel bitmap pictogram identifies each effect; bypass adds a slash. Empty slots show a plus sign. The right grid shows all inline parameters of the selected effect, with values only and unused cells blank. There is no parameter pagination. Routing remains a separate editor. Both FX bands have a four-logical-pixel gap to the timeline, below the input rack and above the output rack.

## Input model

Keyboard and mappable semantic actions are primary. Pointer and touch operate on the same selection and action model.

| Input | FX slots focused (left) | Parameters focused (right) |
|---|---|---|
| Shift+Enter / Shift+F10 | Open effect-type choices | Open selected parameter choices |
| Up / Down | Previous / next slot, including empty slots | Previous / next available parameter |
| Enter | Switch focus to parameters | Switch focus to FX slots |
| Q / E | Previous / next effect kind; populate the selected empty slot | Decrease / increase the selected value |
| Shift+M | Toggle selected effect bypass | Toggle selected effect bypass |
| Delete | Empty selected slot | No keyboard action |
| Ctrl+Up / Ctrl+Down | Swap effect with preceding / following slot | No keyboard action |
| Left / Right | Previous / next track | Previous / next track |
| Shift+Up / Shift+Down or Shift+Left / Shift+Right | Previous / next timeline context | Previous / next timeline context |
| Shift+1 / 2 / 3 / 4 (top row) | Select FX slot 1 / 2 / 3 / 4 | Select FX slot 1 / 2 / 3 / 4; keep parameter focus |

Navigation cycles in reading order. Reordering stops at the first/last slot; it does not wrap. Swapping with an empty slot moves the effect into that slot. Swapping with an occupied slot exchanges effects. Selection follows the moved effect, and remembered parameter selection moves with it. Removing an effect preserves its empty slot and selection. Changing an existing effect never cycles through None; Delete is the explicit removal operation.

Enter switches focus even when the slot is empty. Its blank parameter area has no editable values; Enter returns to FX. Q/E in the blank parameter area does nothing. The selected slot is remembered per track and input/output chain. Parameter selection is remembered per slot, and switching focus does not reset it. Replacing an effect starts parameter selection at the first parameter.

Shift+Up/Down and Shift+Left/Right both switch timeline contexts, from either focus area. Ctrl+Up/Down reorders when slots have focus. Shift+1–4 selects the corresponding physical slot in the current FX chain, including empty slots, preserving focus and each slot’s remembered parameter selection. These top-row shortcuts apply only inside an FX context. Unmodified digits still select tracks; Shift+numpad and Alt-based stored-loop shortcuts retain their existing behavior. User-defined key mappings take precedence over built-in shortcuts. Other pages retain their existing bindings.

## Feedback and pointer interaction

The selected effect remains outlined when parameters have focus. The actively edited icon or parameter has a contrasting fill and inset frame. The parameter area has a focus outline while active. The footer gives the chain, slot number, full effect name, bypass state, selected parameter label/value, and contextual keyboard hints. Selection feedback works without hover. Hovering a slot or value shows its actual track, chain, effect name, parameter label/value, and mapping badges in the footer without changing selection. Keyboard or mapped actions clear the hover description so selected-control feedback takes precedence.

The first click on an effect icon selects its slot and focuses FX without changing the kind. Clicking the already selected, focused icon again cycles to the next effect kind; further clicks continue cycling. For an empty slot, the first click selects and the next click inserts an effect into that exact slot. Each parameter click immediately focuses and edits that parameter: the left half decreases its value, and the right half increases it. The exact midpoint belongs to the right half. Unused parameter cells do nothing. Mouse and touch share these rules and use the same undoable actions as keyboard/mapped editing. Hover descriptions state Left - / Right +. Adjacent triangular buttons invoke previous/next-slot reorder actions. They are pointer affordances, not additional keyboard focus stops. Adding, replacing, bypassing, and deleting remain available through keyboard and mappings.

Right-click an FX icon (including an empty slot) to choose from all ten effect types, each with its icon, or Empty slot. Right-click a populated parameter cell to choose an exact available value. Opening a menu selects and focuses its target without editing it. Shift+Enter, Shift+F10, and the mappable Open Timeline FX Options action open the same menu for the focused control.

Menus mark the current choice with an asterisk. Click an item or use Up/Down and Enter to apply it as one undoable edit. Q/E and Left/Right also move through choices; Home/End jump to the endpoints. Long lists scroll with the wheel or navigation. Escape, another right-click, or clicking outside cancels without an edit or click-through. Other actions dismiss the menu before acting. User key mappings take precedence over Shift+Enter and Shift+F10; while a menu is open, its navigation keys operate the menu.

Parameter lists follow the existing inline editor's available values and step sizes, including numeric bounds, rhythmic choices, roots, and target tracks. Note Filter List offers All or the first N notes within Low–High, matching the existing count-based inline control. A custom current value is retained as an additional choice when needed. Choosing the current effect kind preserves its parameters; choosing another uses that kind's defaults and preserves bypass state.

## Mapping contract

Shared navigation uses Previous/Next Page Item, Activate Page Item, and Adjust Page Item Backward/Forward. The existing Previous/Next Mapping Field actions continue to switch timeline contexts when the Timeline page is active.

The mapping catalogue exposes these Active Track targets:

- Open Timeline FX Options
- Select Timeline FX Slot 1, 2, 3, and 4
- Toggle Timeline FX
- Cycle Timeline FX Kind
- Adjust Timeline FX Param 1, 2, 3, and 4
- Move Timeline FX Up / Down
- Add Timeline FX
- Delete Timeline FX

These target the selected slot in the currently selected Input FX or Output FX context. Parameter actions address absolute parameter indices, independent of the focused parameter. Generic adjustment actions provide bidirectional editing of the current selection. Explicit mapped delete/reorder actions remain direct operations on the selected effect; the keyboard shortcuts additionally follow the focus-area rules above.

The legacy Scroll Timeline FX Params target remains accepted and now advances parameter selection without hiding any values. Saved timeline parameter-window fields retain their serialized names and now store parameter selection. Legacy field enum values remain readable; the active two-area flow uses Kind and ParamPrimary.

Direct mapping exposes kind/add on each slot, all available parameter cells, and reorder triangles. Bypass and delete can be assigned in the mapping editor. All effect parameters use existing engine semantics; this UI adds no new musical parameters.

## Verification

Exercise empty, sparse, and full chains, all ten effect kinds, both chains, all four density presets, keyboard events, and mapped MIDI CC input. Verify stable geometry, square slots, disjoint hit targets, all parameter cells, unique icons, focus frames, bypass, deletion, reordering through empty slots, and unchanged track/context navigation. Run formatting, cargo check, focused tests, full tests, renderer captures, and visual review.
