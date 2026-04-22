Findings

1. **severity: high** | **screenshot/page:** `timeline.png` | **issue:** The top control row (`LINK OFF`, `START/STOP OFF`, `F6`, `SHIFT+F6`) is crowded and visually merges into one strip, making action groups hard to parse quickly. | **brief suggested fix:** Split transport/settings/help into distinct grouped containers with more horizontal padding and stronger separators.

2. **severity: high** | **screenshot/page:** `timeline-focused.png` | **issue:** The focused state is ambiguous: both the track header and many surrounding controls keep similar contrast, so it is not immediately obvious what has keyboard focus. | **brief suggested fix:** Add a stronger, unique focus treatment (thicker border + accent fill + optional “FOCUSED TRACK” badge near the focused panel).

3. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Dense rows with minimal vertical breathing room make scanning key/action/scope columns slow, especially with repeated `ACT TRACK` labels. | **brief suggested fix:** Increase row height slightly and add subtle zebra striping or stronger column separators for faster row tracking.

4. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay context is weak: users may not immediately realize this is a modal layer vs a page, because the background dimming and modal boundary are too subtle. | **brief suggested fix:** Darken the backdrop more and strengthen modal chrome (shadow/border/title bar) to clearly signal modal state.

5. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Mixed color semantics are unclear (`green`, `pink`, `orange`, `blue` all used as structural and state colors), which weakens meaning of ON/OFF and mode states. | **brief suggested fix:** Reserve a strict semantic palette (e.g., green=enabled, red/orange=warning/off, neutral=structure) and keep section colors separate from state colors.

6. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Device cards have large empty gray interiors with little explanatory labeling, which can read as missing content or disabled areas. | **brief suggested fix:** Add concise sublabels (status, channels, activity) or reduce empty fill area so cards feel intentionally informative.

7. **severity: low** | **screenshot/page:** `mappings.png` | **issue:** Top metadata (`ROWS 1 / 30`, `SCOPE`, `ON`) is right-aligned but visually detached from the table header, making relationship to columns unclear. | **brief suggested fix:** Align metadata directly within the header row or place it in a dedicated header band with clear labels.

8. **severity: low** | **screenshot/page:** `timeline.png` | **issue:** Some labels appear cramped or visually clipped by tight containers (e.g., small top-right controls and compact button captions), reducing legibility at a glance. | **brief suggested fix:** Increase min-width/padding for compact buttons and prevent label text from touching borders.

9. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Repeated tiny controls (`+`, `SET`, `TGL`) create high cognitive load; intent is not always obvious without prior knowledge. | **brief suggested fix:** Replace shorthand with explicit verbs on hover/focus or compact icon+tooltip patterns, and standardize placement per row type.

10. **severity: low** | **screenshot/page:** `timeline-focused.png` | **issue:** Hierarchy between song column and loop detail is weak; both panes compete visually with similar weights and dense marks. | **brief suggested fix:** Emphasize primary pane with stronger title/contrast and de-emphasize secondary pane grid/marks until selected.