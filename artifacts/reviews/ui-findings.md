Findings

1. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Major controls are visually dense and similarly weighted (top mode controls, per-track toggles, per-column headers), so scan priority is weak and it takes time to find the current actionable area. | **brief suggested fix:** Increase hierarchy contrast: stronger section headers, lighter treatment for secondary controls, and one clear “active context” band.

2. **severity: medium** | **screenshot/page:** `timeline-focused.png` | **issue:** The focused state (`TRACK T1`) is easy to miss because its styling is close to neighboring controls; state change from the multi-track view is not strongly communicated. | **brief suggested fix:** Add a clearer focus indicator (stronger color fill, badge, or dedicated “Focused Track” label near the title).

3. **severity: low** | **screenshot/page:** `mappings.png` | **issue:** Column spacing/alignment feels inconsistent across dense rows (`SOURCE`, `TARGET`, `SCOPE`, `ON`), and the narrow rightmost status area is cramped, making row parsing slower. | **brief suggested fix:** Normalize column widths/padding and give the status column slightly more width or consolidate status into one clearer token.

4. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay command hints (`F5 CLOSE`, `W WRITE`) are understated and blend with surrounding table text, so interaction affordances are easy to miss. | **brief suggested fix:** Promote overlay actions into a distinct, high-contrast action strip at the top of the modal.

5. **severity: low** | **screenshot/page:** `midi-io.png` | **issue:** “DEF/SEL” labels in selected cards are small and visually detached from the card title, reducing clarity of selection/default state at a glance. | **brief suggested fix:** Move state chips closer to device names and increase contrast/size slightly.

6. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Mixed control semantics (`SET`, `TGL`, plus/minus steppers) are not immediately self-explanatory and look similar despite different behavior. | **brief suggested fix:** Differentiate control types with clearer visual grammar (button style by action type) and add short inline labels/tooltips like “toggle,” “apply,” “increment.”

7. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Some labels are tightly packed with values (`REC FX`, `MON FX`, `KIND`, `SEMI`, `VEL`) and spacing is uneven between grouped blocks, creating visual noise. | **brief suggested fix:** Standardize vertical rhythm between field rows and increase label-to-value spacing for readability.

8. **severity: high** | **screenshot/page:** `timeline.png` and `timeline-focused.png` | **issue:** Text/legend density in track headers and mini-panels risks quick comprehension failure (many abbreviations and compact tokens like `ARM/REC/MUT/SOL`, `IN/OUT`, tiny numeric lanes). | **brief suggested fix:** Reduce simultaneous visible metadata (progressive disclosure), expand key labels, and reserve tiny typography for secondary info only.