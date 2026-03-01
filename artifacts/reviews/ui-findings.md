Findings

1. severity: `high` | screenshot/page name: `mappings.png` | issue: The bottom command hint bar is crowded and partially ambiguous (`TAP ROW`, `TAP FIELD`, `TAP AGAIN ACT`, `W WRITE`, `N NEW`, `DEL REMOVE`), with weak separation between hotkeys and actions, so it reads like a run-on string. | brief suggested fix: Group shortcuts into clearer chunks with stronger spacing/dividers and label them as `key -> action`.

2. severity: `high` | screenshot/page name: `mappings.png` | issue: The table is too dense for quick scanning; rows, columns, and state chips (`GLOBAL`, `ACT TRACK`, `ON`) have nearly equal visual weight, making it hard to identify the active mapping at a glance. | brief suggested fix: Increase contrast between selected row, body rows, and metadata columns; reduce emphasis on secondary columns and add more row padding.

3. severity: `medium` | screenshot/page name: `mappings-overlay.png` | issue: The overlay header is unclear: `F5 CLOSE  W WRITE` reads like content rather than actions, and the overlay lacks a strong title/action hierarchy. | brief suggested fix: Separate title from commands, e.g. title on the left and shortcut actions on the right with clearer spacing or boxed buttons.

4. severity: `medium` | screenshot/page name: `mappings-overlay.png` | issue: `ROWS 1-19 / 26` and `SCOPE` are visually detached from the table and easy to miss, so pagination/state context is weak. | brief suggested fix: Attach these indicators to the table header row or place them in a dedicated top-right status block with stronger contrast.

5. severity: `medium` | screenshot/page name: `midi-io.png` | issue: The large empty device panels are visually dominant and read like missing content or broken lists, especially because the actual selected device labels are tiny and tucked into the corners. | brief suggested fix: Reduce empty panel dominance and surface device names/counts more prominently near the section headers.

6. severity: `medium` | screenshot/page name: `routing.png` | issue: The control model is hard to parse quickly: `+`, `TAP +/-`, `SELECT`, and `TOGGLE` mix interaction styles without explaining whether values are stepped, opened, or directly editable. | brief suggested fix: Standardize control affordances by using consistent verbs/icons for the same interaction type and reserve `TAP` wording for only one interaction pattern.

7. severity: `low` | screenshot/page name: `routing.png` | issue: The active state in the top mode strip (`ACTIVE 1`) is much louder than neighboring context (`THRU OFF`, `TRACK 1`, explanatory text), which makes the strip feel unbalanced and harder to scan. | brief suggested fix: Rebalance emphasis so the primary mode, selected track, and routing context read as one structured header.

8. severity: `medium` | screenshot/page name: `timeline.png` | issue: The top toolbar is overloaded with similarly styled pills (`PLAY OFF`, `REC OFF`, `MODE OVERDUB`, `SONGLOOP ON`, `TEMPO 120`, etc.), so state importance is unclear and users must read each item individually. | brief suggested fix: Group transport, mode, sync, and meter into separate visual sections and reserve the strongest highlight for the most critical live states.

9. severity: `medium` | screenshot/page name: `timeline.png` | issue: Track headers and loop headers are cramped; labels like `TRACK 1` and `LOOP` sit very close to borders and neighboring UI, which hurts readability and risks clipping at smaller sizes. | brief suggested fix: Add more header padding and slightly widen or simplify header content.

10. severity: `low` | screenshot/page name: `all screenshots` | issue: Section headers, tab states, and footer actions all use similar line weight and small text, so the interface lacks a clear hierarchy for first-time scanning. | brief suggested fix: Increase hierarchy contrast by making page titles and active areas more distinct, while muting persistent chrome and secondary metadata.