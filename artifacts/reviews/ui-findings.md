Findings

1. severity: medium; screenshot/page name: `timeline.png`; issue: the top control bar is too dense and uses many abbreviated labels (`REC QF`, `MODE OVERDUB`, `RECOMPR EXTEND`, `NOTEFWD`, `PEERS 0`) with nearly uniform visual weight, so it is hard to scan state quickly. brief suggested fix: group controls into labeled clusters, reduce abbreviation where possible, and reserve stronger contrast for active song/transport states.

2. severity: medium; screenshot/page name: `timeline.png`; issue: track header text is cramped and visually collides with neighboring controls and borders, especially around `THRU`, track names, and `MUTE` badges, which makes the per-track state harder to parse. brief suggested fix: add padding inside track headers, simplify header content, or split routing/state badges onto a second line.

3. severity: low; screenshot/page name: `timeline.png`; issue: the selected states are not immediately obvious because multiple highlight colors compete at once across tabs, loop cells, and toggle pills. brief suggested fix: reduce the number of accent colors used for selection and keep one consistent highlight treatment for “currently selected” versus “enabled”.

4. severity: high; screenshot/page name: `mappings.png`; issue: each row contains many compressed columns and the `SCOPE`/`ON` area at the far right feels cramped, making the table hard to read quickly and easy to mis-scan. brief suggested fix: widen or rebalance columns, shorten row count per page, and visually separate the status columns from the main mapping content.

5. severity: medium; screenshot/page name: `mappings.png`; issue: the top mode controls (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) read like equal-weight buttons even though they appear to represent different kinds of state. brief suggested fix: distinguish mode selectors from action buttons with different styling and add clearer active/inactive state treatment.

6. severity: medium; screenshot/page name: `mappings.png`; issue: the footer shortcut legend is dense and cryptic, with many low-contrast abbreviations packed into one line, which increases cognitive load. brief suggested fix: reduce the number of shortcuts shown inline, expand the most important labels, and group keys by action type.

7. severity: low; screenshot/page name: `mappings-overlay.png`; issue: the overlay has a large amount of empty space below the visible rows, so the content feels unfinished and the user may not immediately know whether more rows are hidden or missing. brief suggested fix: tighten the overlay height to content or add clearer pagination/scroll affordances.

8. severity: medium; screenshot/page name: `mappings-overlay.png`; issue: the overlay title, close hint, row count, and scope labels are weakly organized, so the header does not establish a strong reading order. brief suggested fix: turn the header into a clearer two-zone layout with title/actions on the left and row/status metadata on the right.

9. severity: medium; screenshot/page name: `midi-io.png`; issue: the device cards are visually large but mostly empty, which makes them read like inactive panels rather than selectable list items and weakens information density. brief suggested fix: reduce card height or add meaningful device metadata/status inside the body so the space communicates value.

10. severity: medium; screenshot/page name: `midi-io.png`; issue: `DEF` and `SEL` badges are small and easy to miss, so default versus selected state is not obvious at a glance. brief suggested fix: increase badge prominence and use a more explicit selected treatment on the entire card, not just the corner tags.

11. severity: medium; screenshot/page name: `routing.png`; issue: the right-edge controls mix `TAP +/-`, `SELECT`, and `TOGGLE` in different widths and styles, which makes the interaction model feel inconsistent and slightly misleading. brief suggested fix: standardize control affordances and label patterns so adjustment, selection, and toggle actions are visually distinct but structurally consistent.

12. severity: low; screenshot/page name: `routing.png`; issue: the segmented header (`ACTIVE T1`, `THRU OFF`, `TRACK 1`) has ambiguous hierarchy, so it is not immediately clear which value is the current context versus a toggle versus a title. brief suggested fix: separate page title, active target, and mode/toggle state into distinct labeled regions.

13. severity: low; screenshot/page name: `routing.png`; issue: the pastel fill colors for rows are strong, but the semantic meaning of each color is not obvious without prior knowledge. brief suggested fix: add a small legend or use more explicit labels/icons so color is supportive rather than required for comprehension.