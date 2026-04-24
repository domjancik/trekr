Findings

1. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Top control strip is overly dense (`WRAP EXTEND`, `SONG LOOP ON`, `TEMPO 120`, `HARMONY C`, `NOTEADD OFF`, plus transport/launch controls), making scan order and grouping hard to parse quickly. | **brief suggested fix:** Split into 2 clearly labeled groups (song/loop settings vs transport/launch), with larger horizontal gaps and consistent group headers.

2. **severity: medium** | **screenshot/page:** `timeline-focused.png` | **issue:** Focused state is not immediately obvious beyond the `TRACK T1` pill; the rest of the layout still looks like the full timeline page, reducing state clarity. | **brief suggested fix:** Add a stronger focused-mode cue (page subtitle contrast, dim non-focused controls, or a persistent “Focused Track View” badge in the main canvas header).

3. **severity: low** | **screenshot/page:** `mappings.png` | **issue:** Table columns are visually crowded; `TYPE/DEVICE/SOURCE/TARGET/SCOPE/ON` headers are small relative to row density, so row parsing is slow. | **brief suggested fix:** Increase header contrast/size and add slightly more vertical row spacing or alternating row tint for easier scanning.

4. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay command hints (`F5 CLOSE`, `W WRITE`) are understated and easy to miss, so overlay state/actionability is weak. | **brief suggested fix:** Promote overlay actions into a distinct high-contrast action bar with clearer verb labels (e.g., `Close Overlay`, `Write Mapping`).

5. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Device cards contain very large empty body areas, which read like missing content rather than intentional structure; hierarchy between name/state/content is unclear. | **brief suggested fix:** Reduce empty panel height or populate with concise status/meta rows (ports, channel, activity), and tighten card padding.

6. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Heavy abbreviation usage (`REC/MON`, `TGL`, `P2`, `VEL`) makes controls harder to understand without prior knowledge. | **brief suggested fix:** Expand key labels or add short inline helper text/tooltips for first-use clarity.

7. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Multiple colored sections compete equally for attention, weakening primary hierarchy (active track route vs secondary FX settings). | **brief suggested fix:** Reserve strongest contrast for primary route path and mute secondary panels until selected/focused.