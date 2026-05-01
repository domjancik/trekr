Findings

1. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** The top control strip is very dense; labels like `REC MODE OVRDUB`, `REC WRAP EXT`, `SONG LOOP ON`, `LAUNCH Ø OFF` read as similar-weight blocks, so state and action are hard to scan quickly. | **brief suggested fix:** Increase hierarchy by separating toggles vs status pills (different fill/weight), and emphasize active states with stronger contrast plus a consistent ON/OFF pattern.

2. **severity: high** | **screenshot/page:** `timeline.png` | **issue:** Track columns are packed tightly; repeated micro-labels (`ARM/REC/MUT/SOL`, `THRU`, `SONG`, `OUR`, small step numbers) create visual noise and make each lane hard to parse at a glance. | **brief suggested fix:** Reduce label repetition per lane (show shared headers once), add more vertical spacing between control rows, and simplify tiny badges.

3. **severity: medium** | **screenshot/page:** `timeline-focused.png` | **issue:** The focused view still has many small controls above the main piano-roll area, so “focused” mode does not strongly communicate reduced complexity. | **brief suggested fix:** Collapse nonessential top controls in focused mode and enlarge primary track/loop labels to reinforce focus context.

4. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Table rows are extremely compressed; separators and text baselines are close enough that rows visually blend, reducing scannability and increasing misread risk. | **brief suggested fix:** Add row height/padding and stronger alternating row contrast or subtler striping.

5. **severity: low** | **screenshot/page:** `mappings.png` | **issue:** `ROWS 1 / 30` in the top-right is easy to miss and weakly tied to current viewport state. | **brief suggested fix:** Move row count next to table header labels or add `showing x-y of n` directly above the list.

6. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay command hints (`F5 CLOSE`, `W WRITE`) are low prominence and can be mistaken for static labels rather than actionable shortcuts. | **brief suggested fix:** Group shortcuts into a distinct “Overlay Controls” bar with stronger contrast and consistent keycap styling.

7. **severity: low** | **screenshot/page:** `midi-io.png` | **issue:** Large empty card interiors dominate each device block, while actionable state (`DEF/SEL`) is tiny and top-right; this makes selection state easy to miss. | **brief suggested fix:** Reduce empty area or repurpose it for clear status lines; enlarge and anchor selection/default indicators near device names.

8. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Mixed abbreviations (`REC FX`, `MON FX`, `TGL`, `P2`, `VEL`) and varying label styles make controls feel inconsistent and harder to interpret quickly. | **brief suggested fix:** Standardize terminology and abbreviation rules; provide short explicit labels/tooltips for ambiguous fields.

9. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** `ACTIVE T1` and `THRU OFF` near the top read like adjacent tabs/buttons, but their interaction relationship is unclear. | **brief suggested fix:** Visually group mode vs track selectors into separate containers with headings or divider spacing.

10. **severity: low** | **screenshot/page:** `mappings.png`, `midi-io.png`, `routing.png`, `timeline*.png` | **issue:** Global footer shortcuts (`F5 MAPPINGS`, `F7 DISCOVER`, `F8 DIRECT`) are consistently present but visually de-emphasized, so discoverability is weak for first-time use. | **brief suggested fix:** Increase footer shortcut contrast and add a subtle “Global Shortcuts” label to improve affordance.