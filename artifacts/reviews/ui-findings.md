Findings

1. severity: high; screenshot/page name: `routing`; issue: the header band mixes status, mode, track context, helper text, and `TAP VALUE` into one row with weak grouping, so it is hard to tell what is editable versus just status; brief suggested fix: split this into clearly labeled sections or rows, and give editable controls a stronger visual treatment than passive labels.

2. severity: high; screenshot/page name: `midi-io`; issue: the large pale panels read like empty placeholders even though they represent device selections, and the tiny `DEF`/`SEL` badges are easy to miss; brief suggested fix: add clearer card titles/state labels inside each panel and make default/selected badges larger and visually anchored.

3. severity: medium; screenshot/page name: `timeline`; issue: the top control strip is overly dense and uses many short, cryptic labels (`RECOMPR EXTEND`, `...`, `0 1/16`, `PEERS 0`) with similar styling, which weakens hierarchy and makes state hard to scan quickly; brief suggested fix: group controls by function, use more explicit labels where possible, and reserve stronger contrast for active states.

4. severity: medium; screenshot/page name: `mappings`; issue: the mappings table is extremely compressed horizontally, with long target names, tiny scope values, and very small footer hints competing in the same visual plane; brief suggested fix: increase row height or column separation, and promote the primary columns while de-emphasizing secondary metadata and shortcuts.

5. severity: medium; screenshot/page name: `mappings-overlay`; issue: the overlay header combines title, commands, row count, and scope with weak spacing, so the action hints (`F5 CLOSE`, `W WRITE`) feel like part of the title instead of separate controls; brief suggested fix: separate title, commands, and table metadata into distinct aligned zones.

6. severity: low; screenshot/page name: `routing`; issue: action labels on the right edge of rows (`SELECT`, `TOGGLE`, `TAP +/-`) are inconsistent in wording and sizing, which makes similar controls feel unrelated; brief suggested fix: standardize action terminology, width, and placement across rows.

7. severity: low; screenshot/page name: `timeline`; issue: the track columns rely on very small markers and faint grid divisions, so active content and loop detail are harder to read at a glance; brief suggested fix: increase contrast for active note/step markers and strengthen the selected-column emphasis relative to inactive tracks.

8. severity: low; screenshot/page name: `mappings`, `mappings-overlay`; issue: footer/help text is undersized and low-contrast relative to the rest of the UI, making critical keyboard guidance easy to overlook; brief suggested fix: raise contrast and separate primary shortcuts from secondary hints with stronger spacing or grouping.