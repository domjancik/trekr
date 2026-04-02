Findings

1. **severity: high** | **screenshot/page:** `mappings-overlay.png` | **issue:** The range label reads `ROWS 1-9 / 30` while the overlay visibly shows more than 9 rows, which makes paging/state unclear. | **brief suggested fix:** Bind the range text directly to rendered row indices and add a simple page indicator (`Page 1/2`) to avoid ambiguity.

2. **severity: high** | **screenshot/page:** `timeline-focused.png` | **issue:** Top-right transport/meta labels (`F6 / SHIFT+F6` area) are visually cramped against adjacent controls, reducing readability and looking partially crowded/clipped. | **brief suggested fix:** Increase horizontal padding and enforce a minimum gap between status chips and shortcut hints; wrap to a second line when space is tight.

3. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** The track columns are very dense; row labels, lane numbers, and note blocks compete visually, making scanning across 6 tracks slow. | **brief suggested fix:** Increase inter-column spacing slightly and reduce non-critical micro-label density (or fade secondary labels) to improve primary pattern readability.

4. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** State chips in the top control rows (`WRAP EXTEND`, `SONG LOOP ON`, `TEMPO 120`, etc.) read like one continuous string due to weak separation. | **brief suggested fix:** Add stronger chip boundaries/spacing and consistent active/inactive styling so each control state is parseable at a glance.

5. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Heavy abbreviation use (`REC FX`, `MON FX`, `TGL`, `VAL`, `CH`) makes controls hard to understand quickly, especially for first-time users. | **brief suggested fix:** Expand key labels where possible (or add inline helper text/tooltips) and reserve abbreviations for repeated, well-learned controls only.

6. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Large empty gray regions dominate each device card, implying missing content or disabled areas without clear meaning. | **brief suggested fix:** Replace blank fills with explicit placeholders (`No channels shown`, `Device details`) or compact card heights when no extra content exists.

7. **severity: low** | **screenshot/page:** `mappings.png` | **issue:** Bottom shortcut legend is crowded and uses similarly weighted styles, so destructive/primary actions are not clearly prioritized. | **brief suggested fix:** Group legend actions by category (edit/navigation/destructive) and increase contrast differences for critical actions.

8. **severity: low** | **screenshot/page:** `mappings.png` and `routing.png` | **issue:** Inconsistent internal spacing/padding between section headers and first control rows creates subtle alignment jitter across pages. | **brief suggested fix:** Standardize vertical rhythm (header-to-content gap, row height, section padding) with shared layout tokens.