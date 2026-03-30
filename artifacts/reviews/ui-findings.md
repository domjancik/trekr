Findings

1. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Top control bars are overloaded with dense, all-caps microcopy (`WRAP EXTEND`, `NOTEADD OFF`, `F6 / SHIFT+F6`, `QUANT 1/16`, etc.), making state hard to parse quickly. | **brief suggested fix:** Group controls into labeled clusters (Playback, Launch, Quantize), increase vertical spacing, and reduce simultaneous always-visible tokens.

2. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Per-track columns are visually crowded; track headers and loop lane labels are very small and compete with note data, reducing scanability. | **brief suggested fix:** Increase header height/font for track labels and reduce non-essential inline labels inside each column.

3. **severity: low** | **screenshot/page:** `timeline-focused.png` | **issue:** The focused view improves readability, but top status/control strips still use compact shorthand and weak hierarchy, so mode/state is not immediately obvious. | **brief suggested fix:** Add stronger active-state styling (filled badges/toggles) and replace shorthand with slightly clearer labels where space allows.

4. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Table density is high and column semantics are not immediately clear (especially `TYPE/DEVICE/SOURCE/TARGET/SCOPE/ON` with minimal separation). | **brief suggested fix:** Increase column separation and use stronger header contrast or subtle column backgrounds to improve column parsing.

5. **severity: low** | **screenshot/page:** `mappings.png` | **issue:** Footer command hints are cryptic (`W WRITE`, `N NEW`, `DEL REMOVE`, `0/E ADJUST`) and visually blend together. | **brief suggested fix:** Convert hints into grouped keycaps with short verb labels and consistent separators.

6. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay communicates row range (`ROWS 1-19 / 30`) but leaves large unused space below list content, weakening information density and flow. | **brief suggested fix:** Either show more rows per page or reduce overlay height to keep list and controls visually tighter.

7. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Input/output device cards contain large blank gray regions with unclear meaning (looks like missing data or inactive widgets). | **brief suggested fix:** Add explicit empty-state labels or meters/placeholders so users understand what those areas represent.

8. **severity: low** | **screenshot/page:** `midi-io.png` | **issue:** `DEF`/`SEL` badges are very small and tucked into corners, making selection/default state easy to miss. | **brief suggested fix:** Increase badge size/contrast and place state indicators consistently near device names.

9. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Heavy abbreviation use (`TGL`, `SET`, `VAL`, `REC/MON`, `INP FX`) makes controls feel technical but ambiguous for quick operation. | **brief suggested fix:** Expand highest-impact labels (or add inline tooltips/help row) for clearer action intent.

10. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Visual hierarchy between left signal path and right FX/monitor sections is close in weight, so primary workflow order is not immediately obvious. | **brief suggested fix:** Emphasize step order with numbered section headers or stronger progressive grouping from input -> monitoring -> output.