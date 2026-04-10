Findings

1. **severity: high | screenshot/page: `timeline.png` | issue:** The screen is information-dense with very small text and many same-weight controls, so primary status vs editable controls is hard to parse quickly. **brief suggested fix:** Increase visual hierarchy (larger section titles, stronger grouping backgrounds, and clearer primary/secondary control styles) and reduce simultaneous on-screen control density.

2. **severity: high | screenshot/page: `timeline-focused.png` | issue:** Multiple labels are visibly truncated (`SM...`, `VL...`, `RT...`, etc.), which hides meaning in a view that should be “focused.” **brief suggested fix:** Prioritize full labels in focused mode (wider label columns, overflow tooltips, or abbreviated but standardized tokens).

3. **severity: medium | screenshot/page: `timeline.png` | issue:** Repeated micro-controls (`ARM/REC/MUT/SOL`, loop toggles, launch/quant/peers) are tightly packed with minimal spacing, making adjacent hit targets/read targets blend together. **brief suggested fix:** Add consistent vertical rhythm and slightly larger padding between control rows/groups.

4. **severity: medium | screenshot/page: `routing.png` | issue:** `SET`, `+`, and `TGL` controls appear frequently with similar visual weight but different behaviors, which is misleading and slows interpretation. **brief suggested fix:** Differentiate action types by style (e.g., toggle state pills vs action buttons) and add short inline labels/tooltips.

5. **severity: medium | screenshot/page: `routing.png` | issue:** State communication is weak in some sections (`REC FX`, `MON FX`, `THRU`) because ON/OFF emphasis is subtle relative to surrounding chrome. **brief suggested fix:** Use stronger state contrast (filled active backgrounds, clearer OFF treatment, and consistent color semantics).

6. **severity: medium | screenshot/page: `mappings.png` | issue:** The bottom shortcut/help strip is cramped and visually noisy; tokens blend together and are hard to scan. **brief suggested fix:** Increase spacing between key hints and group them into labeled clusters (navigation, edit, learn, remove).

7. **severity: low | screenshot/page: `mappings-overlay.png` | issue:** Overlay header metadata (`ROWS 1-19 / 30`, `SCOPE`) is detached from table headers and easy to miss. **brief suggested fix:** Align metadata with the table header row and increase contrast/size slightly.

8. **severity: low | screenshot/page: `midi-io.png` | issue:** Large empty interior areas in device cards dominate the view and can look like missing content/state ambiguity. **brief suggested fix:** Use compact card heights or add explicit empty-state/level/state placeholders so blank space reads intentional.

9. **severity: low | screenshot/page: `mappings.png` | issue:** Column hierarchy is weak; `TYPE/DEVICE/SOURCE/TARGET/SCOPE/ON` headers are small and low-emphasis relative to row content. **brief suggested fix:** Increase header contrast/weight and add clearer column separators or background tinting.