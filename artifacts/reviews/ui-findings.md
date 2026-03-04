Findings

1. severity: `medium`; screenshot/page name: `routing.png`; issue: The large colored value bars read like progress meters rather than editable routing selections, so the primary controls are visually misleading at a glance. brief suggested fix: Reduce the fill dominance and make selection affordances clearer with stronger field labels, explicit dropdown/input styling, or visible current-value containers.

2. severity: `medium`; screenshot/page name: `timeline.png`; issue: The top control strip is overly dense and uses many similarly styled toggles (`REC OFF`, `MODE OVERDUB`, `SONGLOOP ON`, `LINK OFF`, `SYNC OFF`), which weakens hierarchy and makes it hard to parse the current transport/song state quickly. brief suggested fix: Group controls by function, add spacing between groups, and increase visual contrast between active, inactive, and secondary states.

3. severity: `medium`; screenshot/page name: `mappings.png`; issue: The mappings table is very information-dense, but column hierarchy is weak and the selected row is only subtly differentiated, which slows scanning and makes state hard to read. brief suggested fix: Strengthen header contrast, add clearer row selection treatment, and give key metadata like scope/state more visual separation from the action text.

4. severity: `medium`; screenshot/page name: `mappings-overlay.png`; issue: The overlay leaves a large empty lower area while the active list occupies only the upper portion, making the modal feel unfinished and reducing information density where focus should be highest. brief suggested fix: Resize the overlay to fit its content better or expand the list area to use the available space more intentionally.

5. severity: `low`; screenshot/page name: `midi-io.png`; issue: The large gray empty regions inside each device card are ambiguous and look like missing content or disabled panes rather than intentional device tiles. brief suggested fix: Either reduce the unused area or populate it with status/details so each card communicates purpose immediately.

6. severity: `low`; screenshot/page name: `midi-io.png`; issue: The `DEF` and `SEL` badges are small and low-emphasis relative to the card size, so default/selected state is easy to miss. brief suggested fix: Increase badge prominence and tie selected/default state to the full card styling, not just tiny corner labels.

7. severity: `low`; screenshot/page name: `timeline.png`; issue: Track headers and state chips are tightly packed with inconsistent spacing, and abbreviations like `ARM`, `REC`, `MUT`, `SOL` create a high learning burden for quick reading. brief suggested fix: Normalize spacing between header elements and surface fuller labels or tooltipped legends for less obvious abbreviations.

8. severity: `low`; screenshot/page name: `mappings.png`; issue: The footer shortcut legend is cramped and visually competes with the main table despite being secondary help text. brief suggested fix: De-emphasize the legend, add more spacing between shortcut groups, or move it into a dedicated help/status region.