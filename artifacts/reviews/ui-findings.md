Findings

1. **severity: high** | **screenshot/page:** `timeline.png` | **issue:** Top-right status chips (`LINK OFF`, `START/STOP OFF`, `F6 / SHIFT+F6`, etc.) are crowded and visually merge into one dense block, making state hard to scan quickly. | **brief suggested fix:** Split into labeled groups (transport, launch, quantize), add consistent horizontal gaps, and reserve one line per group on narrower widths.

2. **severity: high** | **screenshot/page:** `timeline.png` | **issue:** Per-track headers and controls are extremely compressed across 6 columns; labels like `THRU TRACK`, `SONG`, `DWRP` become hard to parse and compete with note content. | **brief suggested fix:** Increase minimum track-column width or reduce visible tracks per row with horizontal paging/scroll.

3. **severity: medium** | **screenshot/page:** `timeline-focused.png` | **issue:** The `TRACK 1` selector and adjacent controls feel detached from the page title/description, so active scope (“focused track mode”) is not strongly communicated. | **brief suggested fix:** Add an explicit mode badge near the main title (e.g., `FOCUSED VIEW`) and align selector grouping with that heading.

4. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Header controls (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) have inconsistent widths/padding and don’t clearly indicate which are toggles vs status fields. | **brief suggested fix:** Standardize control sizing and apply distinct styling for `status`, `toggle`, and `action` elements.

5. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Right-side metadata (`ROWS 1 / 30`, `SCOPE ON`) is weakly aligned and visually disconnected from the table it describes. | **brief suggested fix:** Place row/scope metadata in a dedicated table header row aligned to column boundaries.

6. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay command hints (`F5 CLOSE`, `W WRITE`) are easy to miss and look like regular labels rather than actionable shortcuts/state. | **brief suggested fix:** Move shortcuts to a high-contrast shortcut bar and prefix with a clear label like `SHORTCUTS:`.

7. **severity: low** | **screenshot/page:** `midi-io.png` | **issue:** Large empty device panels dominate the screen, while key actions (`DEF`, `SEL`) are small and low-emphasis; this weakens action hierarchy. | **brief suggested fix:** Increase prominence of primary actions and reduce empty-panel visual weight with subtle placeholders or compact mode.

8. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Left and right control stacks use different internal spacing rhythms, making the two-column layout feel uneven and harder to scan. | **brief suggested fix:** Normalize row heights, label offsets, and vertical spacing across both columns.

9. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Some value fields resemble disabled controls (muted contrast, similar fills), which can mislead users about editability (`PASSTHROUGH OFF`, `INFX ON`, etc.). | **brief suggested fix:** Differentiate editable/selectable/toggle states with stronger contrast and distinct border treatments.

10. **severity: low** | **screenshot/page:** `all pages` | **issue:** Bottom shortcut strip (`F5 MAPPINGS`, `F7 DISCOVER`, `F8 DIRECT`) is visually subtle and competes with dense main content, reducing discoverability of global navigation. | **brief suggested fix:** Increase contrast and spacing of the global shortcut bar, and keep active destination more visibly highlighted.