Findings

1. severity: medium | screenshot/page: `midi-io.png` | issue: Subtitle has a typo (`SELECT DEFULT INPUTS AND OUTPUTS`), which hurts trust and quick comprehension. | brief suggested fix: Change to `SELECT DEFAULT INPUTS AND OUTPUTS`.

2. severity: medium | screenshot/page: `timeline.png` | issue: Track columns are too dense in `TRACK ALL`; labels and note marks become hard to scan quickly. | brief suggested fix: Increase minimum column width or reduce visible tracks in this mode, with horizontal paging/scroll.

3. severity: medium | screenshot/page: `timeline.png` | issue: Header/control bars are crowded (`PLAY/REC/MODE`, `LINK/START/STOP`, `QUANT/PEERS`) with weak grouping, so state is hard to parse at a glance. | brief suggested fix: Split into clearly separated groups with stronger spacing and section labels.

4. severity: low | screenshot/page: `timeline-focused.png` | issue: Left/right pane meaning (`SONG` vs `LOOP`) is subtle; users can miss which side they are editing. | brief suggested fix: Add stronger pane headers/background differentiation and explicit active-pane highlight.

5. severity: medium | screenshot/page: `mappings.png` | issue: Bottom shortcut strip is overloaded and visually compressed, reducing readability and discoverability. | brief suggested fix: Break shortcuts into grouped clusters (navigation/edit/learn) with spacing or two-line layout.

6. severity: low | screenshot/page: `mappings-overlay.png` | issue: Overlay action hints (`F5 CLOSE`, `W WRITE`) read like plain text, not actionable controls. | brief suggested fix: Style them as button chips or a dedicated command bar with clearer affordance.

7. severity: medium | screenshot/page: `mappings.png` and `mappings-overlay.png` | issue: Scope/state values (`GLOBAL`, `ACT TRACK`, `ARMED/ACT`) are visually similar and low-emphasis, weakening state communication. | brief suggested fix: Add consistent color coding or badges per scope/state category.

8. severity: low | screenshot/page: `routing.png` | issue: Right-side per-row actions (`SELECT`, `TOGGLE`, `TAP +/-`) look detached from the value fields and can read as status labels. | brief suggested fix: Increase contrast and proximity, and use consistent button styling for all row actions.

9. severity: low | screenshot/page: `midi-io.png` | issue: Device cards contain large unlabeled empty regions, which can look like missing content rather than selectable areas. | brief suggested fix: Add subtle placeholder labels (for ports/channels/activity) or reduce empty fill area.