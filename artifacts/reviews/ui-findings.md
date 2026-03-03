Findings

1. severity: high; screenshot/page name: `timeline.png`; issue: the top control bar is too dense and several chips read like a continuous strip of equal-weight toggles, so critical transport/state information (`PLAY OFF`, `REC OFF`, `SONGLOOP ON`, tempo, division, link/sync) is hard to scan quickly; brief suggested fix: group controls into labeled clusters with more spacing and stronger emphasis for primary live states versus secondary settings.

2. severity: medium; screenshot/page name: `timeline.png`; issue: track headers and loop badges are cramped against the dense marker row above them, which weakens separation between timeline content and track metadata; brief suggested fix: add more vertical padding above track cards or simplify the marker row so headers have clearer breathing room.

3. severity: medium; screenshot/page name: `mappings.png`; issue: the footer hint bar packs too many commands into a single low-contrast line, making key actions hard to discover and parse under pressure; brief suggested fix: split primary actions from secondary hints and increase spacing/contrast for the most important commands.

4. severity: medium; screenshot/page name: `mappings.png`; issue: `TAP MODE: READ ONLY` and `TAP LEARN: IDLE` look visually similar even though one is a persistent mode and the other is transient state, which blurs status communication; brief suggested fix: differentiate mode and live state with distinct styling or placement.

5. severity: medium; screenshot/page name: `mappings-overlay.png`; issue: the overlay title, close hint, write action, row count, and column headers are loosely arranged and do not form a strong visual header, so the dialog takes extra effort to orient; brief suggested fix: consolidate these into a clearer header block with aligned metadata and stronger separation from the table.

6. severity: low; screenshot/page name: `mappings-overlay.png`; issue: the empty lower half of the overlay feels unfinished and makes the visible content appear top-heavy; brief suggested fix: reduce overlay height or use the extra space for help text, pagination context, or details for the selected mapping.

7. severity: medium; screenshot/page name: `midi-io.png`; issue: the selected/default tags (`DEF`, `SEL`) are tiny and tucked into the card edge, so device state is easy to miss; brief suggested fix: promote selection/default state into clearer badges or a stronger card header treatment.

8. severity: medium; screenshot/page name: `midi-io.png`; issue: the large empty device panels dominate the screen and make it unclear whether content is missing, loading, or simply absent by design; brief suggested fix: add explicit empty-state or list framing so users understand what the panel is expected to show.

9. severity: high; screenshot/page name: `routing.png`; issue: the repeated right-edge buttons mix symbols and words (`+`, `SELECT`, `TOGGLE`, `TAP +/-`) without clear consistency, which makes the interaction model harder to learn; brief suggested fix: standardize control affordances so similar actions use the same labeling pattern and placement.

10. severity: medium; screenshot/page name: `routing.png`; issue: `ACTIVE 1`, `THRU OFF`, `TRACK 1`, and the descriptive sentence compete in the same strip without clear hierarchy, so the current routing context is harder to parse than it should be; brief suggested fix: separate current state badges from descriptive text and make the active track context more prominent.

11. severity: low; screenshot/page name: `routing.png`; issue: several rows have oversized colored fields with minimal visible content, which makes the page feel visually heavy relative to the actual information shown; brief suggested fix: reduce field height or increase internal labeling so each row communicates its purpose more efficiently.

12. severity: low; screenshot/page name: `mappings.png`, `midi-io.png`, `routing.png`, `timeline.png`; issue: bottom-right shortcuts (`F5 MAPPINGS`, `F7 DISCOVER`) look like persistent status labels rather than actionable navigation, so they are easy to overlook; brief suggested fix: restyle them as clearer buttons or move them into a more obviously interactive navigation/help area.