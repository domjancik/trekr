Findings

1. severity: high; screenshot/page name: `timeline.png`; issue: the top control bar is overcrowded and several labels are cryptic (`MODE OVERDUB`, `RECOMPR EXTEND`, `SONGLOOP ON`, `F6 LINK`), which makes state hard to parse quickly. brief suggested fix: group controls into labeled sections, reduce the number of always-visible toggles, and replace shorthand with clearer labels or tooltipped abbreviations.

2. severity: high; screenshot/page name: `timeline.png`; issue: track headers are cramped and repetitive, with `ARM/REC/MUTE/SOLO` compressed into very small zones above each column, weakening hierarchy and making per-track state hard to scan. brief suggested fix: increase header height or simplify each track header to the highest-value states, with stronger selected-track emphasis.

3. severity: medium; screenshot/page name: `mappings.png`; issue: the table is extremely dense, and the active row highlight is subtle relative to the rest of the grid, so it is hard to tell where focus is. brief suggested fix: increase row height or spacing slightly and make the focused row/state more visually distinct with stronger contrast or a left-edge indicator.

4. severity: medium; screenshot/page name: `mappings.png`; issue: scope values such as `ACT TRACK`, `ARMED/ACT`, and `GLOBAL` are visually similar to editable cells, which makes the table harder to understand at a glance. brief suggested fix: style scope as a distinct badge/tag column or use stronger column separation and alignment.

5. severity: medium; screenshot/page name: `mappings-overlay.png`; issue: the overlay title and close/write hints do not clearly communicate whether this is a modal, filtered view, or editable mode; it reads like a second table layered on top without enough state explanation. brief suggested fix: add a clearer subtitle such as “Read-only overlay” or “Quick mapping browser” and visually separate command hints from content.

6. severity: medium; screenshot/page name: `mappings-overlay.png`; issue: there is a large amount of unused space below the rows while the table content stays packed at the top, making the overlay feel unfinished and unbalanced. brief suggested fix: either enlarge row spacing/font size or constrain the panel height to better fit the visible content.

7. severity: medium; screenshot/page name: `midi-io.png`; issue: the large empty device panels dominate the page, but selection state is communicated only by small `DEF`/`SEL` tags, which are easy to miss. brief suggested fix: make the selected/default device state part of the card framing or header treatment instead of relying on tiny corner labels.

8. severity: low; screenshot/page name: `midi-io.png`; issue: the page hierarchy is weak because `INPUTS` and `OUTPUTS` headers are clear, but the actual device cards beneath them have inconsistent visual weight and too much blank space. brief suggested fix: tighten panel sizing to content and strengthen section-to-card spacing consistency.

9. severity: medium; screenshot/page name: `routing.png`; issue: controls on the right edge (`TAP +/-`, `SELECT`, `TOGGLE`) look like status chips more than actions, so affordance is unclear. brief suggested fix: give action controls a more button-like treatment and separate them from current-value fields.

10. severity: medium; screenshot/page name: `routing.png`; issue: the color coding across rows is strong, but its meaning is not explained, which makes the page harder to understand quickly for new users. brief suggested fix: add a minimal legend or reserve color for state categories with explicit labels.

11. severity: low; screenshot/page name: `routing.png`; issue: the `ACTIVE 1` and `THRU OFF` pills at the top have stronger contrast than the explanatory text, which pulls attention away from the actual routing settings. brief suggested fix: reduce pill emphasis slightly or increase the prominence of the section description/current edit target.

12. severity: low; screenshot/page name: `mappings.png`, `mappings-overlay.png`, `routing.png`, `timeline.png`; issue: bottom-right footer shortcuts (`F5 MAPPINGS`, `F7 DISCOVER`, `F8 DIRECT`) are persistent but visually understated and easy to ignore despite being global navigation. brief suggested fix: either promote them into a clearer global nav/status bar or reduce their presence if they are secondary.