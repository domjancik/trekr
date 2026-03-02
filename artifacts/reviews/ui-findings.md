Findings

1. severity: high; screenshot/page name: `timeline.png`; issue: the top control bar is too dense and visually uniform, so primary transport/state controls (`PLAY OFF`, `REC OFF`, `MODE OVERDUB`, `SONGLOOP ON`, tempo, meter, link, sync) read like one long strip of equal-weight chips and are hard to parse quickly; brief suggested fix: group related controls with stronger spacing and panel separation, and give the most important transport/mode states a clearer visual priority.

2. severity: high; screenshot/page name: `mappings.png`; issue: the table is overloaded and several columns are too compressed, making row content hard to scan quickly, especially with repeated `--`, narrow type cells, and cramped right-side scope/on columns; brief suggested fix: widen or rebalance columns, reduce placeholder noise, and add stronger distinction between primary mapping content and secondary status columns.

3. severity: high; screenshot/page name: `routing.png`; issue: the large colored value bars dominate the page, but the meaning of the right-edge action/status labels (`ADJUST`, `READY`, `TOGGLE`) is unclear and disconnected from the selected state, which makes the interaction model hard to understand at a glance; brief suggested fix: separate editable values from action buttons/status badges and add clearer selected/focused-state treatment for the active field.

4. severity: medium; screenshot/page name: `midi-io.png`; issue: the input/output cards have inconsistent internal spacing and weak state communication, with `DEF` and `SEL` rendered as tiny corner tags that are easy to miss and visually cramped against the device labels; brief suggested fix: promote default/selected states into clearer badges or a dedicated status area with consistent padding.

5. severity: medium; screenshot/page name: `mappings-overlay.png`; issue: the overlay title, instructions, and row counter compete at the top with minimal separation, and the table lacks enough vertical rhythm, so the modal feels dense despite being a simplified view; brief suggested fix: add more spacing between header/instructions/table and slightly increase row height or column padding.

6. severity: medium; screenshot/page name: `timeline.png`; issue: track headers and loop/detail mini-columns are visually crowded, and the repeated tiny labels/markers at the top of each lane make it difficult to identify the active track and understand what each narrow column represents; brief suggested fix: strengthen header hierarchy, label the subcolumns more explicitly, and reduce decorative micro-markers that do not communicate clear meaning.

7. severity: medium; screenshot/page name: `mappings.png`; issue: the footer help text is very small and tightly packed, which makes important keyboard guidance easy to overlook; brief suggested fix: increase spacing between shortcuts and separate commands into clearer groups or segments.

8. severity: low; screenshot/page name: `routing.png`; issue: `ACTIVE 1` and `THRU OFF` look like similar toggle buttons even though they appear to represent different concepts, which weakens hierarchy and state clarity; brief suggested fix: differentiate mode tabs from status toggles through color, size, or grouping.

9. severity: low; screenshot/page name: `midi-io.png`; issue: long device names sit close to container edges and status tags, creating a near-clipped feel even where text is not technically cut off; brief suggested fix: add a bit more horizontal padding and reserve fixed space for status badges.

10. severity: low; screenshot/page name: `mappings-overlay.png`; issue: the highlighted first row relies mostly on a thin outline, which is subtle relative to the density of the grid and easy to miss; brief suggested fix: use a stronger fill/contrast change for the active row.
