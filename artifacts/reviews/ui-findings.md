Findings

1. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** The six-track view is visually overcrowded; per-track labels, note lanes, and loop chips are too dense to scan quickly. | **brief suggested fix:** Increase minimum column width or reduce simultaneous columns (pagination/zoom mode) and simplify per-column header content.

2. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Several header tokens look truncated/cryptic (for example the small chips like `+2..`, `FE / SHIFT+F6`), which weakens immediate comprehension. | **brief suggested fix:** Use explicit abbreviations with consistent width rules and tooltips/help text for shortened labels.

3. **severity: low** | **screenshot/page:** `timeline-focused.png` | **issue:** State communication is better than the full timeline, but REC/MUT/SOL headers are low-salience compared to large content panes, so track state is easy to miss. | **brief suggested fix:** Increase contrast/emphasis for active state headers and add clearer active/inactive markers.

4. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Bottom command hint strip is crowded and reads like a single dense sentence, making key actions hard to discover fast. | **brief suggested fix:** Group hints by function (edit/navigation/mapping) with spacing separators and stronger visual hierarchy.

5. **severity: low** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay top actions (`F5 CLOSE`, `W WRITE`) are small and visually similar to static labels, so actionability is unclear. | **brief suggested fix:** Style actionable commands as distinct buttons/chips and separate them from passive text.

6. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Large empty interior blocks dominate each device card without clear meaning, creating visual noise and weak information density. | **brief suggested fix:** Either remove/reduce those blocks or repurpose them with explicit status content (activity, channels, routing summary).

7. **severity: low** | **screenshot/page:** `midi-io.png` | **issue:** Selection/default indicators (`DEF`, `SEL`) are tiny and easy to miss in the top-right of cards. | **brief suggested fix:** Promote these to stronger badges/toggles with clearer contrast and consistent placement.

8. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Control types (`SELECT` vs `TOGGLE`) are repetitive and visually similar at a glance, which slows interaction parsing. | **brief suggested fix:** Differentiate control styles more strongly (shape/color/icon) and surface current state inline (for example `ON/OFF` as primary badge).