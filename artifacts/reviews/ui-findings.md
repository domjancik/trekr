Findings

1. **severity: medium** — **mappings.png** — The mapping table uses very small, low-contrast text and dense rows, making source, target, scope, and enabled state difficult to scan quickly. **Suggested fix:** increase row height and text contrast, or provide stronger column separation and clearer headers.

2. **severity: low** — **mappings.png** — The `DEVICE` column is mostly empty (`--`), consuming space without adding useful information. **Suggested fix:** collapse empty device values or reduce the column width.

3. **severity: medium** — **mappings-overlay.png** — The overlay is visually dominant but provides little indication of how to close it beyond `F5 CLOSE`; the keyboard-only affordance is easy to miss. **Suggested fix:** add a visible close button and a clearer modal title/state treatment.

4. **severity: medium** — **midi-io.png** — Large empty device preview panels overpower the actual device names and selected-state indicators. **Suggested fix:** reduce empty panel height and make device identity, selection, and default status the primary visual content.

5. **severity: medium** — **routing.png** — The routing page contains many similarly styled colored controls, making the hierarchy between device selection, channel selection, FX, and toggles difficult to understand. **Suggested fix:** strengthen section headings, use consistent control patterns, and visually distinguish editable fields from status indicators.

6. **severity: low** — **routing.png** — Several labels are cramped or truncated, particularly the output device name (`MICROSOFT GS WAVETAB...`). **Suggested fix:** provide tooltips, allow wrapping, or widen the field when the value is selected.

7. **severity: medium** — **timeline.png** — Six track columns are compressed into narrow strips, resulting in tiny labels and densely packed controls that are difficult to operate or interpret. **Suggested fix:** support horizontal scrolling, resizable track widths, or a focused-track mode by default.

8. **severity: high** — **timeline-clip-align.png** — The clip-align dialog obscures much of the timeline while appearing embedded in the upper-left area, making it unclear what content is modal and what remains interactive. **Suggested fix:** use a centered modal with a backdrop, clear dialog boundaries, and explicit Cancel/Apply actions.

9. **severity: medium** — **timeline-clip-align.png** — `FIT 4200 -> 3840 TICKS @ 110 BPM` is presented as dense technical text without explaining the practical result. **Suggested fix:** rewrite it as a plain-language preview, such as “Shorten clip by 360 ticks; tempo remains 110 BPM.”

10. **severity: low** — **timeline-focused.png** — The focused-track state is subtle; the selected track is mainly communicated through a blue header while surrounding controls remain visually similar. **Suggested fix:** add a stronger focus outline or explicit “Focused: Track 1” status indicator.