Findings

1. severity: high; screenshot/page name: `timeline.png`; issue: the track columns are too narrow for the amount of information shown, so labels like `TRACK 1`/`LOOP` sit against borders and the dense header chips make the page hard to parse quickly; brief suggested fix: increase column/header breathing room or reduce simultaneous metadata shown per column.

2. severity: high; screenshot/page name: `mappings.png`; issue: the table is visually overloaded and low-contrast in the data area, so rows, scopes, and enabled states blur together and the current selection is not obvious at a glance; brief suggested fix: strengthen row hierarchy with clearer selected-row treatment, more spacing between columns, and stronger differentiation for status cells.

3. severity: medium; screenshot/page name: `mappings.png`; issue: the footer key hints are cramped and read like a continuous strip, making actions such as `DEL REMOVE` and `W WRITE` easy to miss; brief suggested fix: group shortcuts into clearer clusters with more spacing and a stronger visual separation from the table.

4. severity: medium; screenshot/page name: `mappings-overlay.png`; issue: the overlay header actions (`F5 CLOSE`, `W WRITE`) are understated and easy to confuse with ordinary labels rather than available commands; brief suggested fix: style actionable shortcuts as buttons or distinct badges and separate them from static headings.

5. severity: medium; screenshot/page name: `midi-io.png`; issue: selected/default state communication is ambiguous because `DEF` and `SEL` are tiny tags attached to one card while the rest of the card layout looks mostly the same; brief suggested fix: make selected/default states primary card treatments with stronger color fills, icons, or dedicated status labels.

6. severity: medium; screenshot/page name: `routing.png`; issue: the right-edge action cells (`SELECT`, `TOGGLE`, `TAP +/-`) look bolted onto the fields and are not clearly differentiated from value display, which makes interaction intent unclear; brief suggested fix: restyle them as explicit action buttons with consistent width and clearer affordance.

7. severity: low; screenshot/page name: `routing.png`; issue: spacing and alignment are inconsistent between the mode tabs (`ACTIVE IN`, `THRU OFF`), the track text block, and the `TAP VALUE` panel, so the top section feels visually unbalanced; brief suggested fix: align these elements to a common vertical rhythm and normalize internal padding.

8. severity: low; screenshot/page name: `mappings-overlay.png`; issue: the `ROWS 1-19 / 26` and `SCOPE` labels are visually detached from the table and easy to overlook, weakening scanability of context/state; brief suggested fix: place table meta information in a dedicated header row aligned with the columns.

9. severity: low; screenshot/page name: `timeline.png`; issue: state chips such as `PLAY OFF`, `REC OFF`, `SYNC OFF`, `PEERS 0`, and `SONGLOOP ON` use very similar sizing and emphasis despite different importance, so priority is unclear; brief suggested fix: create a stronger hierarchy by emphasizing transport-critical states and de-emphasizing secondary telemetry.

10. severity: low; screenshot/page name: all pages; issue: many labels sit very close to borders and divider lines, creating a slightly clipped/tight feel even where text is technically visible; brief suggested fix: add a small, consistent padding increase around headings, tabs, and table cells.