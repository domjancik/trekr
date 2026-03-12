Findings

1. **severity: medium**; **screenshot/page:** `timeline.png`; **issue:** Control text in the top tool rows is very dense and low-contrast (`WRAP EXTEND`, `TEMPO 120`, `NOTEADD OFF`, quant/peer/link group), which makes state scanning slow and feels visually cramped. **brief suggested fix:** Increase font size/contrast for state chips and add more horizontal padding/gap between chip groups.

2. **severity: medium**; **screenshot/page:** `timeline.png`; **issue:** Per-track headers are crowded (`ARM/REC/MUT/SOL`, track name, `SONG`, `DUB`, loop digits) and compete for attention, so it is hard to identify the active state quickly. **brief suggested fix:** Promote only active states with stronger emphasis and demote secondary labels; split header into two clearer rows or simplify controls shown per track.

3. **severity: low**; **screenshot/page:** `timeline-focused.png`; **issue:** `TRACK T1` label is less clear than `TRACK 1` and looks inconsistent with other naming on the page. **brief suggested fix:** Standardize wording to `TRACK 1` everywhere and keep one naming convention for track identity.

4. **severity: medium**; **screenshot/page:** `mappings.png`; **issue:** Bottom command hint strip is overloaded and tightly packed (`TAP ROW`, `TAP FIELD`, `TAP AGAIN ACT`, etc.), reducing quick discoverability. **brief suggested fix:** Group hints by action type (navigation/edit/commit), add spacing, and hide lower-priority hints until needed.

5. **severity: low**; **screenshot/page:** `mappings-overlay.png`; **issue:** Overlay status header (`ROWS 1-19 / 30`, `SCOPE`) is visually detached from table columns and easy to miss. **brief suggested fix:** Align status metadata directly with the table header row and increase contrast/weight slightly.

6. **severity: medium**; **screenshot/page:** `midi-io.png`; **issue:** Selected/default cues (`DEF SEL`) are tiny and embedded in card headers, so default I/O state is not obvious at a glance. **brief suggested fix:** Add a stronger selected/default badge style (larger chip or icon) and keep it in a consistent position across cards.

7. **severity: medium**; **screenshot/page:** `routing.png`; **issue:** Right-edge action affordances are inconsistent (`TAP +/-`, `SELECT`, `TOGGLE`) and look like static labels rather than interactive controls. **brief suggested fix:** Use one consistent button style/pattern for all row actions and add a clear active/focus state.

8. **severity: low**; **screenshot/page:** `routing.png`; **issue:** Top context bar mixes `ACTIVE T1`, `THRU OFF`, `TRACK 1`, and `TAP VALUE` with weak hierarchy, making current routing context ambiguous. **brief suggested fix:** Reformat as labeled key-value groups (e.g., `Active Track: 1`, `Thru: Off`) and separate status from actions visually.