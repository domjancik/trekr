Findings

1. severity: high; screenshot/page name: `routing`; issue: the top status strip is hard to parse quickly because `ACTIVE T1`, `THRU OFF`, `TRACK 1`, the description text, and `TAP VALUE` all compete at the same visual level, so the current mode and editable context are unclear; brief suggested fix: group this into labeled sections with stronger separation and promote the primary state (`Active Track Routing`, selected track, thru state) using larger contrast or spacing.

2. severity: high; screenshot/page name: `midi-io`; issue: the large light-gray device panes dominate the screen but appear empty and non-interactive, which makes it unclear whether ports are missing, selected, or simply placeholders; brief suggested fix: add explicit empty/loaded state text inside panes and visually distinguish selected devices from unused space.

3. severity: medium; screenshot/page name: `mappings`; issue: the table is very dense and row contents run close to cell boundaries, making long targets like `HALF/DOUBLE LOOP` and scope labels harder to scan; brief suggested fix: increase row padding and rebalance column widths so `Target` and `Scope` breathe more.

4. severity: medium; screenshot/page name: `mappings`; issue: `TAP MODE: READ ONLY` and `TAP LEARN: IDLE` look like interactive controls but read more like passive badges, so state vs action is ambiguous; brief suggested fix: style them clearly as status chips or convert them into obvious buttons/toggles with affordances.

5. severity: medium; screenshot/page name: `mappings-overlay`; issue: the overlay header is weakly structured, with `F5 CLOSE`, `W WRITE`, column headers, and row count all packed into the same band, which slows comprehension; brief suggested fix: separate commands from table metadata and strengthen the title/header hierarchy.

6. severity: medium; screenshot/page name: `timeline`; issue: the toolbar above the tracks is crowded with many equal-weight pills (`PLAY OFF`, `REC OFF`, `MODE OVERDUB`, `SONGLOOP ON`, `TEMPO 120`, etc.), so the most important live state is not obvious; brief suggested fix: prioritize transport and recording state visually, and demote secondary settings into a quieter group.

7. severity: medium; screenshot/page name: `timeline`; issue: the paired track columns repeat similar structures without strong labels for the difference between the left and right lane, making the layout harder to understand on first glance; brief suggested fix: add clearer per-lane labels or a stronger visual distinction between track and loop/detail columns.

8. severity: low; screenshot/page name: `routing`; issue: right-edge action labels like `SELECT`, `TOGGLE`, and `TAP +/-` are cramped against colored fields and can read like field values instead of actions; brief suggested fix: separate action buttons from value areas with more padding and a more button-like treatment.

9. severity: low; screenshot/page name: `mappings-overlay`; issue: `ROWS 1-19 / 26` and `SCOPE` sit far from the table title and feel disconnected from pagination/filter state; brief suggested fix: align row count and scope info with the main header or table controls so they read as related metadata.

10. severity: low; screenshot/page name: `timeline`; issue: small labels such as `F6 LINK`, `PEERS 0`, and the tiny top markers above tracks are legible only with effort, weakening quick scanning; brief suggested fix: increase font size or contrast slightly for secondary status text and markers.