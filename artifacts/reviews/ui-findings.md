Findings

1. severity: high; screenshot/page name: `timeline`; issue: the top control bar is overcrowded, with many similarly styled pills (`MODE OVERDUB`, `RECOMPR EXTEND`, `SONGLOOP ON`, `NOTENDD OFF`, `F6 LINK`) competing at the same visual level, which makes the primary state hard to read quickly; brief suggested fix: group controls into labeled clusters, increase spacing between groups, and promote only the most important active states with stronger emphasis.

2. severity: high; screenshot/page name: `timeline`; issue: several labels look abbreviated to the point of being unclear or potentially clipped (`RECOMPR EXTEND`, `NOTENDD OFF`, dense track header labels), so the user has to decode terminology instead of scanning; brief suggested fix: use clearer full labels where possible, or add more horizontal room and reduce the number of visible tokens per row.

3. severity: medium; screenshot/page name: `mappings`; issue: the table is very dense and rows have nearly identical contrast/weight, so selected row, editable state, and column hierarchy are not immediately obvious; brief suggested fix: strengthen the selected-row treatment, add more separation between header and body, and reduce visual noise in non-active rows.

4. severity: medium; screenshot/page name: `mappings`; issue: the top mapping controls (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) read like the same kind of control even though they appear to represent different modes/actions, which is misleading; brief suggested fix: separate status fields from action buttons visually and use distinct styles for toggles versus one-shot actions.

5. severity: low; screenshot/page name: `mappings`; issue: the footer shortcut legend is cramped and low-hierarchy relative to the main table, making key actions harder to discover; brief suggested fix: give the shortcut bar more padding and clearer grouping so related actions read as intentional clusters.

6. severity: medium; screenshot/page name: `mappings-overlay`; issue: the overlay uses a large amount of empty lower space while the important content is compressed into the upper half, which weakens balance and makes the modal feel unfinished; brief suggested fix: reduce overlay height or vertically center the content block so the table occupies more of the available space.

7. severity: medium; screenshot/page name: `mappings-overlay`; issue: the tiny helper text under the title (`F5 CLOSE`, `W WRITE`) has weak emphasis and can be mistaken for metadata instead of primary actions; brief suggested fix: style these as explicit action hints or place them in a dedicated header/action row with clearer separation.

8. severity: high; screenshot/page name: `midi-io`; issue: the large empty gray areas inside each device card look like missing content or disabled panels, making it unclear what each card is supposed to communicate beyond the device name; brief suggested fix: either remove that empty interior space or populate it with concise status/details so the card shape matches user expectations.

9. severity: medium; screenshot/page name: `midi-io`; issue: badges like `DEF SEL` are cryptic and too small to communicate selection/default state reliably at a glance; brief suggested fix: spell out `Default` and `Selected`, or use clearer dedicated state chips with stronger contrast.

10. severity: medium; screenshot/page name: `routing`; issue: each routing row mixes value display and row actions (`+`, `SELECT`, `TAP +/-`, `TOGGLE`) in a way that makes the interactive target unclear, especially on the right edge; brief suggested fix: reserve a consistent action column and visually separate editable value areas from controls.

11. severity: medium; screenshot/page name: `routing`; issue: state communication relies heavily on color fills across large bars, but the semantic meaning of each color is not obvious, so users must infer whether a row is selected, editable, or just categorized; brief suggested fix: pair color with explicit labels/icons and use one consistent highlight style for selection versus category.

12. severity: low; screenshot/page name: `routing`; issue: the header row (`ACTIVE 1`, `THRU OFF`, `TRACK 1`, `TAP VALUE`) has uneven spacing and weak hierarchy, so mode, target, and action do not read as a coherent structure; brief suggested fix: align these into a clearer left-to-right grouping with more consistent spacing and one dominant current-context label.