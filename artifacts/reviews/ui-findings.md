Findings

1. **severity: high** | **screenshot/page: `routing.png`** | **issue:** `OUTPUT CHA...` is visibly clipped, so a core control label is truncated and ambiguous. | **brief suggested fix:** Widen the label column or shorten with a deliberate abbreviation (`OUTPUT CH`) used consistently across the app.

2. **severity: high** | **screenshot/page: `routing.png`** | **issue:** Multiple headings read like placeholders (`PORTS? CHANNELS?`, `PRE-OUTPUT CHAIN?`, etc.), which makes the UI feel uncertain/misleading. | **brief suggested fix:** Replace `?` copy with finalized labels and reserve punctuation only for help/tooltips.

3. **severity: medium** | **screenshot/page: `timeline.png`** | **issue:** Track header strips are overcrowded (`1 2 3 4 5 6 7 8 + TRACK + LOOP`), weakening quick scan and state recognition. | **brief suggested fix:** Split header info into two rows (track identity vs step/loop state) or increase track card width/padding.

4. **severity: medium** | **screenshot/page: `timeline-focused.png`** | **issue:** Focused mode still carries dense micro-labels with low legibility, so “focused” state does not communicate enough simplification. | **brief suggested fix:** In focused mode, enlarge key labels and hide secondary metadata until selected/hovered.

5. **severity: medium** | **screenshot/page: `mappings.png`** | **issue:** Bottom hotkey/action legend is tightly packed and visually noisy, making control affordances hard to parse quickly. | **brief suggested fix:** Group actions into 2-3 labeled clusters (edit, learn, navigation) with stronger spacing and separators.

6. **severity: medium** | **screenshot/page: `mappings-overlay.png`** | **issue:** Large unused lower area and minimal framing around row-range (`ROWS 1-19 / 30`) make pagination/state feel unclear. | **brief suggested fix:** Add explicit pagination/status block near the table and rebalance panel height to match content.

7. **severity: medium** | **screenshot/page: `midi-io.png`** | **issue:** Device cards are dominated by large empty gray fields, so information hierarchy is weak (name/state controls don’t stand out). | **brief suggested fix:** Reduce empty fill height, promote device metadata/status, and surface actionable controls near the title area.

8. **severity: low** | **screenshot/page: `midi-io.png`** | **issue:** `DEF`/`SEL` chips are tiny and cramped at card edges, which weakens selected/default state communication. | **brief suggested fix:** Increase chip size/contrast and place state badges in a consistent, padded position.

9. **severity: low** | **screenshot/page: `mappings.png` and `mappings-overlay.png`** | **issue:** Inconsistent row density and spacing between main table vs overlay creates a visual jump between related views. | **brief suggested fix:** Normalize row height, header spacing, and column rhythm across both mappings views.

10. **severity: low** | **screenshot/page: `timeline.png` and `routing.png`** | **issue:** Several top-bar secondary labels are very low-contrast/tiny compared to primary tabs, making global mode/state hard to understand at a glance. | **brief suggested fix:** Increase contrast and font size for key secondary state labels, or reduce non-critical copy in the top region.