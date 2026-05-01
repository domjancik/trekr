Findings

1. **severity: high | screenshot/page: `timeline.png` | issue:** The track columns are visually dense and repetitive, so it’s hard to quickly parse which controls are global vs per-track (top transport row, per-track headers, and FX rows compete equally). **brief suggested fix:** Increase hierarchy contrast: make global controls a stronger band, reduce per-track header noise, and group per-track controls with clearer section separators.

2. **severity: high | screenshot/page: `timeline-focused.png` | issue:** The left/right pane split uses nearly identical visual weight, but the right pane is the active note area; users may not immediately understand primary focus vs supporting context. **brief suggested fix:** Emphasize the active edit pane (stronger border/background or title treatment) and slightly de-emphasize the companion pane.

3. **severity: medium | screenshot/page: `mappings.png` | issue:** Bottom helper hints (`TAP ROW`, `W WRITE`, `F8 DIRECT`, etc.) are cramped and read like a single noisy strip, weakening discoverability of key actions. **brief suggested fix:** Chunk hints into labeled groups (navigation/edit/mode) with spacing between groups and stronger active-key styling.

4. **severity: medium | screenshot/page: `mappings-overlay.png` | issue:** Overlay table has large unused vertical space while row text is compact, making scanning slower and reducing information density where it matters. **brief suggested fix:** Slightly increase row height/text size or show more rows per view; tighten top/bottom padding to balance content area.

5. **severity: medium | screenshot/page: `routing.png` | issue:** Repeated micro-labels (`SET`, `TGL`, `+`) appear in many contexts with little semantic distinction, which can feel misleading (same affordance, different effects). **brief suggested fix:** Use contextual button labels/icons (e.g., `Apply`, `Toggle`, `Increment`) or add tiny section-specific legends.

6. **severity: medium | screenshot/page: `midi-io.png` | issue:** Card bodies are mostly empty gray blocks with minimal per-device state, so users can’t quickly tell what each input/output is doing beyond selection. **brief suggested fix:** Add concise status metadata per card (channel/activity/route/latency) and reduce empty fill dominance.

7. **severity: low | screenshot/page: `routing.png` | issue:** Some long labels (e.g., section subtitles and device names) sit close to borders and neighboring controls, creating near-clipping tension and visual crowding. **brief suggested fix:** Add a bit more horizontal padding and enforce truncation with ellipsis for long dynamic text.

8. **severity: low | screenshot/page: `mappings.png` and `mappings-overlay.png` | issue:** Scope/state communication is inconsistent (`SCOPE ON`, per-row `ON`, and `GLOBAL/ACT TRACK` all compete), making state meaning ambiguous at a glance. **brief suggested fix:** Standardize state model wording and placement (mode at top, row scope in one column, enable status in dedicated badge style).