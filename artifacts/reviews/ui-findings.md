Findings

1. severity: medium; screenshot/page name: `mappings.png`; issue: top action labels (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) read like buttons but state vs action is unclear, and equal visual weight makes priority ambiguous; brief suggested fix: separate status chips from action buttons (different fill/outline style) and add explicit selected/idle styling.

2. severity: medium; screenshot/page name: `mappings.png`; issue: mapping table rows are very dense with minimal vertical padding, which slows scanning and increases misread risk; brief suggested fix: add 2-4px row height/padding and stronger zebra or divider contrast.

3. severity: low; screenshot/page name: `mappings.png`; issue: `ROWS 1 / 30` is far right and visually detached from the table header, so pagination context is easy to miss; brief suggested fix: place row count adjacent to column headers or near table title.

4. severity: medium; screenshot/page name: `mappings-overlay.png`; issue: overlay has no dim/scrim behind it, so modal state is weak and could be confused with a normal page; brief suggested fix: add a darkened background layer and stronger modal border/title treatment.

5. severity: medium; screenshot/page name: `mappings-overlay.png`; issue: top-left shortcuts (`F5 CLOSE`, `W WRITE`) are easy to overlook and do not communicate primary action hierarchy; brief suggested fix: group them as explicit primary/secondary actions with clearer emphasis.

6. severity: medium; screenshot/page name: `midi-io.png`; issue: large empty list panels look like inactive/placeholder blocks, making current selection state unclear at a glance; brief suggested fix: show list rows or explicit empty-state text and highlight current input/output with clearer badges.

7. severity: low; screenshot/page name: `midi-io.png`; issue: `DEF/SEL` badges are tiny and low-prominence for critical default/selected state; brief suggested fix: increase badge size/contrast and add a legend or header key.

8. severity: high; screenshot/page name: `routing.png`; issue: state communication is inconsistent (`ACTIVE 1`, `THRU OFF`, plus right-side action chips), making it hard to tell what is current state vs what is actionable; brief suggested fix: split into “Current State” and “Actions” groups with distinct styling and labels.

9. severity: medium; screenshot/page name: `routing.png`; issue: right-edge controls (`SELECT`, `TOGGLE`, `TAP +/-`) are cramped and visually detached from their rows; brief suggested fix: increase control width/padding and align them in a consistent action column.

10. severity: medium; screenshot/page name: `timeline.png`; issue: dense top control bars create weak hierarchy; key transport/mode states compete with secondary controls and are hard to parse quickly; brief suggested fix: create a primary status strip (transport/mode/quantize) and move secondary actions to a lower-priority row.

11. severity: low; screenshot/page name: `timeline.png`; issue: repeated abbreviated labels (`ARM`, `REC`, `MUT`, `SOL`, `TRA...`) reduce clarity and some truncation-like text hurts readability; brief suggested fix: expand critical labels where possible or provide consistent tooltip/legend support.

12. severity: low; screenshot/page name: `all pages`; issue: bottom-right function-key hints (`F5/F7/F8`) are subtle and easy to miss despite global importance; brief suggested fix: raise contrast and reserve a persistent, clearer command strip treatment.