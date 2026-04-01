Findings

1. severity: high | screenshot/page name: `timeline.png` | issue: Dense per-track headers and tiny labels (`ARM/REC/MUT/SOL`, mode chips, track footers) compete at the same visual weight, making scan order unclear and slowing comprehension. | brief suggested fix: Increase hierarchy by enlarging/brightening primary row labels, de-emphasizing secondary metadata, and grouping controls into clearer bands (global, per-track, footer).

2. severity: high | screenshot/page name: `timeline-focused.png` | issue: The focused state is not immediately obvious beyond size change; selected track/loop boundaries are subtle in a visually busy grid. | brief suggested fix: Add a stronger focus treatment (clear accent frame + background tint + explicit “Focused: Track 1” badge near header).

3. severity: medium | screenshot/page name: `mappings.png` | issue: Header columns (`TYPE / DEVICE / SOURCE / TARGET / SCOPE / ON`) are weakly separated from row content; rows read like continuous bars, reducing column clarity. | brief suggested fix: Add stronger vertical separators or alternating cell backgrounds per column, especially between `TARGET`, `SCOPE`, and toggle state.

4. severity: medium | screenshot/page name: `mappings.png` | issue: Top mode chips (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) look like editable fields/buttons but unclear which are status vs actions. | brief suggested fix: Split status pills from action buttons with distinct styles and labels (e.g., “Status: Read Only”, “Action: Direct Map”).

5. severity: medium | screenshot/page name: `mappings-overlay.png` | issue: Overlay title and control hints (`F5 CLOSE`, `W WRITE`) are cramped at top-left, while row count sits far right, creating awkward balance and weak orientation. | brief suggested fix: Use a structured overlay header row with left title/actions and right pagination/scope aligned on the same baseline.

6. severity: medium | screenshot/page name: `midi-io.png` | issue: Large empty gray panels inside device cards dominate visual weight and look like missing content rather than intentional list bodies. | brief suggested fix: Reduce empty panel prominence and add explicit empty/list affordances (row separators, “no additional ports”, or count badges).

7. severity: low | screenshot/page name: `midi-io.png` | issue: `DEF/SEL` chips are very small and close to card borders, making state hard to parse quickly. | brief suggested fix: Increase chip contrast and padding; align consistently with device names on a stable baseline.

8. severity: medium | screenshot/page name: `routing.png` | issue: Multiple similar control blocks (`SET`, `TGL`, plus/minus) repeat with minimal differentiation between editable values and toggles, which can mislead interaction expectations. | brief suggested fix: Standardize control semantics visually (buttons vs fields vs toggles) and add concise inline labels/tooltips for action meaning.

9. severity: low | screenshot/page name: `routing.png` | issue: Inconsistent spacing between stacked cards and section gutters (left signal panel vs right FX panels) makes alignment feel uneven. | brief suggested fix: Normalize vertical rhythm and gutter sizes using a single spacing scale across both columns.

10. severity: low | screenshot/page name: `mappings-overlay.png`, `timeline.png`, `routing.png` | issue: Bottom shortcut strip (`F5 MAPPINGS`, `F7 DISCOVER`, `F8 DIRECT`) appears in multiple contexts with equal emphasis, but current/active shortcut state is not obvious. | brief suggested fix: Highlight the active shortcut context and mute inactive ones to improve state communication.