Findings

1. **severity: medium** | **screenshot/page: mappings.png** | **issue:** The command hint row at the bottom is very dense and low-contrast; labels like `SHIFT+LEFT/RIGHT FIELD` and `Q/E ADJUST` read as one long strip and are hard to parse quickly. | **brief suggested fix:** Split hints into grouped clusters with more horizontal padding and stronger contrast between key and action text.

2. **severity: medium** | **screenshot/page: mappings.png** | **issue:** Rows are visually crowded with minimal vertical breathing room, making scan speed slower and increasing misread risk between adjacent mappings. | **brief suggested fix:** Increase row height slightly or add subtle zebra/background differentiation per row block.

3. **severity: low** | **screenshot/page: mappings-overlay.png** | **issue:** Header metadata (`ROWS 1-19 / 30`, `SCOPE`) sits far right with weak visual connection to table columns, so scope context feels detached. | **brief suggested fix:** Align metadata directly above the corresponding column or move it into the table header row.

4. **severity: medium** | **screenshot/page: midi-io.png** | **issue:** Device cards have large empty interiors with no immediate affordance cues, so it is unclear whether they are selectable rows, meters, or placeholders. | **brief suggested fix:** Add explicit inline status/role labels (e.g., `Selected`, `Input Port`, `Inactive`) and reduce empty fill area.

5. **severity: medium** | **screenshot/page: midi-io.png** | **issue:** `DEF` and `SEL` tags are tiny and cramped in the top-right of cards, reducing state clarity. | **brief suggested fix:** Increase tag size/spacing and use clearer chip styling with distinct colors or icons.

6. **severity: high** | **screenshot/page: routing.png** | **issue:** Very high information density across `SIGNAL`, `REC/MON`, `INPUT FX`, and `OUTPUT FX` sections creates weak hierarchy; users must work hard to find current actionable control. | **brief suggested fix:** Introduce stronger section hierarchy (larger section headers, clearer grouping gaps, and a primary “active edit” highlight).

7. **severity: medium** | **screenshot/page: routing.png** | **issue:** Repeated small buttons (`SET`, `TGL`, `+`, `-`) look similar but imply different behaviors, which is potentially misleading. | **brief suggested fix:** Differentiate control types by shape/color and add micro-labels/tooltips for toggle vs commit actions.

8. **severity: medium** | **screenshot/page: timeline.png** | **issue:** Top control bar mixes transport, mode, and track filters in one visual band with similar weight, weakening priority of playback-critical controls. | **brief suggested fix:** Separate transport and global playback controls from per-track/view controls using grouping and contrast tiers.

9. **severity: medium** | **screenshot/page: timeline.png** | **issue:** Track columns are dense and repetitive; headers and lane labels are small relative to content complexity, slowing orientation across 6 tracks. | **brief suggested fix:** Increase header prominence and add clearer per-track separators or alternating column backgrounds.

10. **severity: low** | **screenshot/page: timeline-focused.png** | **issue:** Focused view improves readability, but left/right pane relationship (`SONG` vs `LOOP`) is not immediately self-explanatory for new users. | **brief suggested fix:** Add short pane subtitles (e.g., `Arrangement` and `Loop Detail`) or a one-line contextual helper near the pane headers.