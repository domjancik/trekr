Findings

1. `severity: medium` - `timeline.png` - The top control bar is overcrowded with many small, same-weight pills (`PLAY OFF`, `REC OFF`, `MODE OVERDUB`, `SONGLOOP ON`, `TEMPQ 120`, etc.), so state and action controls blur together and are hard to scan quickly. Suggested fix: group controls by function, separate status from actions visually, and increase contrast/spacing for active states.

2. `severity: medium` - `timeline.png` - The track headers are cramped and partially ambiguous; `LOOP` labels sit tightly inside the yellow header blocks and feel close to clipping, while the paired track/loop columns are not explained clearly at a glance. Suggested fix: give header labels more padding and add clearer structural separation or labels for track vs loop columns.

3. `severity: low` - `timeline.png` - `F6 LINK` appears detached on the far right and reads more like stray text than a usable control or shortcut hint. Suggested fix: align it with the related link control and style shortcut hints consistently.

4. `severity: medium` - `mappings.png` - The page is very dense and table rows are tightly packed, which makes scanning mappings difficult and weakens row-to-row separation. Suggested fix: add a bit more row height/padding and strengthen the selected-row treatment.

5. `severity: medium` - `mappings.png` - Header/status text like `ROWS 1 / 26`, `SCOPE`, and `ON` in the upper right is spatially separated and unclear as a grouped status, so it is hard to tell what `ON` applies to. Suggested fix: cluster related status labels into a single compact component with clearer labels.

6. `severity: low` - `mappings.png` - The footer shortcut legend is too compressed and cryptic (`W WRITE`, `N NEW`, `DEL REMOVE`, etc.), making the page harder to learn quickly. Suggested fix: reduce the number of inline shortcuts shown at once or rewrite them with clearer wording and spacing.

7. `severity: medium` - `mappings-overlay.png` - The overlay header prioritizes shortcut text (`F5 CLOSE`, `W WRITE`) almost as much as the title, and `W WRITE` is not immediately understandable. Suggested fix: make the title dominant and restyle shortcuts as secondary helper text with clearer action names.

8. `severity: low` - `mappings-overlay.png` - There is a large empty area below the visible rows, which makes the overlay feel unfinished and weakens information density. Suggested fix: tighten the modal height to content or use the space for pagination/help/context.

9. `severity: high` - `midi-io.png` - The large light-gray empty rectangles dominate both panels and look like missing content, disabled areas, or rendering placeholders rather than intentional device lists. Suggested fix: add clear list rows, empty-state messaging, or stronger framing so the content area reads as interactive and complete.

10. `severity: medium` - `midi-io.png` - Device state tags like `DEF` and `SEL` are tiny and cramped into the card corners, making default/selected state easy to miss. Suggested fix: enlarge and separate status chips, or move them into a consistent metadata row.

11. `severity: low` - `midi-io.png` - The left/right panel balance is uneven: the single large input card and stacked output cards create inconsistent spacing and visual rhythm. Suggested fix: normalize card sizing or add clearer section structure so the asymmetry feels intentional.