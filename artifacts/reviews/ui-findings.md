Findings

1. `severity: medium` `screenshot/page name: timeline.png` `issue: The top control bar is overly dense, with many equal-weight chips (`PLAY OFF`, `REC OFF`, `MODE OVERDUB`, `TEMPO 120`, `...`, `F6 LINK`) competing at the same visual level, so it is hard to parse status versus actions quickly.` `brief suggested fix: Group controls by function, separate persistent state from actions, and give the highest-priority transport/song states stronger emphasis.`

2. `severity: medium` `screenshot/page name: timeline.png` `issue: Several labels feel cramped against their containers, especially compact chips like `SONGLOOP ON`, `NOTENDD OFF`, and the small track header cells, which hurts scanability and reads close to clipping.` `brief suggested fix: Increase horizontal padding or shorten labels/abbreviations so text has more breathing room.`

3. `severity: low` `screenshot/page name: timeline.png` `issue: The `...` control is unclear and misleading because it looks like an overflow affordance but has no nearby context explaining what it expands or changes.` `brief suggested fix: Replace it with a labeled button or add a clearer icon/text hint for its purpose.`

4. `severity: medium` `screenshot/page name: mappings.png` `issue: The top status/action chips (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) look visually similar to tabs and passive labels, making it unclear which are editable controls versus readouts.` `brief suggested fix: Differentiate interactive controls from status badges with distinct styling and clearer active/inactive states.`

5. `severity: low` `screenshot/page name: mappings.png` `issue: The table header area on the right (`ROWS 1 / 30`, `SCOPE`, `ON`) feels loosely aligned with the grid below, which weakens the table structure.` `brief suggested fix: Align meta info to the column system or move row-count/status info into a dedicated toolbar line.`

6. `severity: medium` `screenshot/page name: mappings.png` `issue: The footer shortcut strip is crowded and low-contrast, so the available actions are hard to understand at a glance.` `brief suggested fix: Reduce the number of visible shortcuts, group them by category, and increase contrast for the most important commands.`

7. `severity: medium` `screenshot/page name: mappings-overlay.png` `issue: The overlay has a large empty lower area while the actionable content is compressed into the top half, making the dialog feel unfinished and weakening hierarchy.` `brief suggested fix: Reduce overlay height or enlarge row spacing/content density so the modal feels intentionally composed.`

8. `severity: low` `screenshot/page name: mappings-overlay.png` `issue: The helper text line (`F5 CLOSE   W WRITE`) is terse and easy to miss, so the overlay’s primary actions are not obvious.` `brief suggested fix: Promote close/save actions into clearer labeled buttons or a more prominent action row.`

9. `severity: medium` `screenshot/page name: midi-io.png` `issue: The device cards are dominated by large empty gray fields, which read like missing content or disabled areas rather than selectable devices.` `brief suggested fix: Shrink the empty fill area and strengthen the actual selectable device label/state treatment.`

10. `severity: medium` `screenshot/page name: midi-io.png` `issue: The tiny `DEF` and `SEL` markers in the card corners are easy to miss and unclear when both appear together.` `brief suggested fix: Use fuller labels or clearer badges/toggles for `Default` and `Selected` states, with more spacing from the card edge.`

11. `severity: medium` `screenshot/page name: routing.png` `issue: Row-end controls (`+`, `SELECT`, `TOGGLE`, `TAP +/-`) are inconsistent in wording, size, and placement, so the interaction model is not obvious.` `brief suggested fix: Standardize action labels and button layouts across rows so similar operations look and behave consistently.`

12. `severity: low` `screenshot/page name: routing.png` `issue: The header strip mixes mode/state (`ACTIVE 1`, `THRU OFF`) with descriptive text and a detached `TAP VALUE` button, which weakens the page’s information hierarchy.` `brief suggested fix: Separate status chips, page description, and global actions into distinct regions with clearer spacing.`