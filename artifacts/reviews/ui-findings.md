Findings

1. severity: medium; screenshot/page name: `timeline.png`; issue: the top control bars are very dense with many similarly styled toggles, so priority actions (transport vs loop vs launch) are hard to scan quickly; brief suggested fix: group controls into clearly separated clusters with stronger section labels and add extra horizontal spacing between groups.

2. severity: medium; screenshot/page name: `timeline.png`; issue: per-track lane labels and note index numbers are cramped against note glyphs, creating near-overlap and reducing readability in busy rows; brief suggested fix: increase left gutter width for row indices and add a few pixels of padding between labels and note content.

3. severity: low; screenshot/page name: `timeline-focused.png`; issue: hierarchy is weak between the selected track state and surrounding inactive controls because most text weight and contrast remain similar; brief suggested fix: dim inactive toolbar text and increase contrast/weight for the active track context label.

4. severity: medium; screenshot/page name: `mappings.png`; issue: table density is high and footer key-hints are hard to parse (many short tokens with minimal separation), so discoverability of edit actions is weak; brief suggested fix: split hints into grouped segments (navigation/edit/learn) with spacing or divider blocks and emphasize the primary action.

5. severity: low; screenshot/page name: `mappings-overlay.png`; issue: top-right metadata (`ROWS 1-19 / 30`, `SCOPE`) feels visually detached from table headers and can be misread as row content context; brief suggested fix: align metadata on the same baseline as column headers or move it into a dedicated overlay status bar.

6. severity: medium; screenshot/page name: `midi-io.png`; issue: device cards have large empty gray areas with little state explanation, making it unclear what is selectable, active, or just placeholder space; brief suggested fix: replace empty fill with concise status text/metrics and add explicit selected/active badges per device.

7. severity: medium; screenshot/page name: `routing.png`; issue: row controls are inconsistent and somewhat misleading (`TAP +/-` appears on one row while others use `SELECT`/`TOGGLE`, plus/minus affordances are not self-explanatory); brief suggested fix: standardize right-side action labels and add explicit verbs/tooltips (e.g., `Cycle`, `Choose`, `Enable`) for each row type.