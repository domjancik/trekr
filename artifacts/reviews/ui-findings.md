Findings

1. severity: high; screenshot/page: `timeline.png`; issue: Very dense multi-track layout has weak visual hierarchy, so track state vs note content is hard to parse quickly (many equal-weight headers/toggles competing at once). brief suggested fix: Increase grouping contrast (section backgrounds), emphasize active track/state with stronger color/weight, and de-emphasize secondary metadata.

2. severity: medium; screenshot/page: `timeline.png`; issue: Top control rows (`PLAY/RECORD/MODE`, then `WRAP EXTEND / SONG LOOP ON / TEMPO 120 / NOTEADD OFF`) are tightly packed with inconsistent spacing and read like one long string. brief suggested fix: Add consistent horizontal padding and split into labeled control groups with clearer separators.

3. severity: medium; screenshot/page: `timeline.png`, `timeline-focused.png`; issue: Right-side status line (`LINK OFF  START/STOP OFF / SHIFT+F6`) is visually cramped and ambiguous (shortcut text and state text blend together). brief suggested fix: Separate shortcut help from live state text, and use a distinct style for keybind hints.

4. severity: medium; screenshot/page: `mappings.png`; issue: Footer command strip is cryptic (`TAP AGAIN ACT`, `W WRITE`, `N NEW`, etc.) and visually crowded, which hurts quick learnability. brief suggested fix: Use clearer verb phrases (e.g., `Write Mapping`) and add spacing/grouping between edit/navigation actions.

5. severity: medium; screenshot/page: `midi-io.png`; issue: Device rows include large unlabeled gray blocks that look like empty/disabled content, making row purpose unclear beyond the device name. brief suggested fix: Add explicit labels or remove/fill those regions with meaningful status/details (channel, activity, routing).

6. severity: low; screenshot/page: `routing.png`; issue: Action affordances on the right (`TAP +/-`, `SELECT`, `TOGGLE`) are inconsistent in wording and button treatment, so interaction model feels mixed. brief suggested fix: Standardize action labels and control style (same verb pattern and same button geometry).

7. severity: low; screenshot/page: `mappings-overlay.png`; issue: State communication is minimal for paging (`ROWS 1-19 / 30`) with no visible next/prev affordance, so users may miss additional mappings. brief suggested fix: Add explicit paging hints (e.g., `PgUp/PgDn`) or a small visible pager control near the row counter.