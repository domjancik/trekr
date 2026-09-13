# UI review findings — fixed FX slots and inspector

Local visual review, 13 September 2026. Inspected the tracked timeline and focused timeline screenshots and populated FX fixtures at Default, Compact, Tiny, and Touch.

- FX slots remain square in a 2×2 grid. The right 2×2 inspector fits within each track band, with no effect-count-driven height growth.
- All three Arp parameters are visible without pagination. Selected Gate has a readable filled cell and contrasting inset frame; the selected effect remains outlined.
- Bypassed icons have a slash. Empty slots retain their position and show a plus.
- Effect shapes are distinguishable at the inspected densities. Tiny remains physically small; controller/keyboard operation is the primary interaction.
- Focused-track mode keeps the same slot size and expands the inspector across the available width. This is consistent with the fixed-slot layout, though value cells are spacious at that width.
- No FX overlap or clipping was observed in the inspected captures. Automated tests also cover disjoint controls, fixed geometry, and unique icon rendering.

Validation: 357 tests passed; two native MIDI hardware tests intentionally ignored. Keyboard events, mapped MIDI CC actions, and hover descriptions were exercised. Tracked screenshots were regenerated with capture-ui-screens.ps1.

The repository's external screenshot-review command was rejected by automatic approval review because screenshot sensitivity and destination authorization were not established. This report records local visual inspection, not an external review.

Shortcut follow-up: refreshed renderer captures and inspected the populated Default lane. The parameter footer visibly leads with Shift+Up/Down context switching and Shift+1–4 FX selection. Reordering uses Ctrl+Up/Down. No geometry changes were introduced.

Context-menu and spacing follow-up: locally inspected effect-type and Note Filter High value menus, refreshed Timeline and focused Timeline, and the populated Tiny fixture. Type choices include distinct icons and Empty slot; numeric lists scroll and mark the current value. No menu clipping was observed. Both FX-to-timeline gaps are four logical pixels, verified across all density presets. Menu cancellation, exact choices, undo, viewport bounds, and non-mutating right-click opening pass regression tests. Full suite: 357 passed; two live MIDI hardware tests ignored. Formatting, cargo check, build, and diff whitespace checks passed.

PR preparation: regenerated all five README screenshots with capture-ui-screens.ps1 -StateMode demo, plus focused timeline and clip-align captures. Locally inspected Timeline, Mappings, Mappings Overlay, MIDI I/O, and Routing; unchanged pages reproduce their tracked images. Included fx-type-menu.png and fx-value-menu.png generated with TREKR_FX_REVIEW_DIR=artifacts/screenshots and the fx_menu_renderer_captures_kind_and_long_numeric_options test. Shift+Enter opens the focused FX choices, while Enter switches sides. cargo fmt --check, cargo xtask check, and cargo test --workspace --all-targets passed (357 tests, two hardware tests ignored).
