Findings

1. severity: high | screenshot/page: `timeline.png` | issue: Track columns are too dense; header text, mode chips, and loop controls compete in the same narrow space, making labels hard to scan quickly. | brief suggested fix: Increase minimum column width or reduce visible tracks at once; collapse secondary controls behind a per-track expander/menu.

2. severity: high | screenshot/page: `timeline.png` | issue: Several labels appear visually clipped or cramped in track header/footer rows (for example `+ ADD OUTPUT FX` and small parameter strings), reducing legibility. | brief suggested fix: Add truncation with ellipsis + tooltip, and enforce consistent horizontal padding so text does not touch borders.

3. severity: medium | screenshot/page: `timeline-focused.png` | issue: The focused view still has crowded control density in the top action rows (play/record/mode/launch/quant), so hierarchy between global transport vs track-specific state is weak. | brief suggested fix: Group global controls into one clearly separated band and track-local controls into another with stronger spacing and section labels.

4. severity: medium | screenshot/page: `routing.png` | issue: Abbreviations (`TGL`, `P1`, `P2`, `REC FX`, `MON FX`) are not self-explanatory, especially for first-time users. | brief suggested fix: Expand key labels or add inline helper text/tooltips for abbreviated controls.

5. severity: medium | screenshot/page: `routing.png` | issue: Visual hierarchy is noisy: many boxes share similar border weight and contrast, so active state and interaction priority are hard to identify quickly. | brief suggested fix: Use stronger contrast only for active/editable sections and lighter treatment for secondary containers.

6. severity: medium | screenshot/page: `mappings.png` | issue: The bottom hotkey legend is dense and cryptic (`W WRITE`, `F8 DIRECT`, etc.) with limited separation between actions and states. | brief suggested fix: Split legend into “actions” and “status” groups with clearer separators and consistent chip styling.

7. severity: low | screenshot/page: `mappings.png` | issue: Some trigger strings are awkwardly formatted (`SHIFT+, .`, symbol-heavy bindings), which can be misread as typos. | brief suggested fix: Normalize keybinding display format (for example `Shift + ,`), and keep one canonical style across all rows.

8. severity: medium | screenshot/page: `mappings-overlay.png` | issue: Overlay leaves a large empty area after visible rows without clear paging/scroll affordance, which can imply missing data or render issues. | brief suggested fix: Add explicit pagination/scroll hint (`Page 1 of N`, `More below`) or auto-fit visible rows.

9. severity: low | screenshot/page: `midi-io.png` | issue: Input/output device cards have heavy repeated framing with minimal differentiation between selected, default, and inactive states beyond small corner tags. | brief suggested fix: Strengthen selected/default visual states (clear icon, stronger fill/border difference) and reduce non-selected visual weight.

10. severity: low | screenshot/page: `all pages` | issue: Global state communication is weak (`LAST ACTION: READY` always present but low-information), while high-impact mode states are easy to miss. | brief suggested fix: Replace passive footer text with contextual status messages and emphasize current mode/state near the relevant controls.