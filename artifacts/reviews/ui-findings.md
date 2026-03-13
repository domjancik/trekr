Findings

1. **severity: medium** — **screenshot/page: `timeline.png`** — **issue:** Top control ribbons are overly dense (`PLAY/RECORD/MODE`, `WRAP/SONG LOOP/TEMPO`, link/start/quantize block), with minimal spacing and equal visual weight, making state scanning slow. **brief suggested fix:** Add stronger grouping (boxed sections), increase horizontal padding between controls, and emphasize active states with one primary accent while muting inactive chips.

2. **severity: medium** — **screenshot/page: `timeline.png`** — **issue:** Track columns are very narrow for the amount of metadata, and labels (e.g., track headers/mode tags) compete with note graphics, reducing readability. **brief suggested fix:** Reduce per-track chrome (header row height and duplicated tags), or show fewer tracks at once with clearer per-track labels.

3. **severity: low** — **screenshot/page: `timeline-focused.png`** — **issue:** Focused mode still carries dense global control rows that visually compete with the main two-pane editor, weakening hierarchy. **brief suggested fix:** De-emphasize global controls in focused mode (smaller/secondary styling) and elevate the active track/loop pane titles.

4. **severity: medium** — **screenshot/page: `mappings.png`** — **issue:** Table headers and top-right metadata (`ROWS 1 / 30`, `SCOPE`, `ON`) feel misaligned and cramped, making column meaning less immediate. **brief suggested fix:** Align header baselines to column starts and combine right-side metadata into a single compact header row.

5. **severity: medium** — **screenshot/page: `mappings-overlay.png`** — **issue:** Overlay key hints (`F5 CLOSE`, `W WRITE`) are easy to miss and ambiguous relative to footer shortcuts on the base screens, which can confuse open/close vs navigate behavior. **brief suggested fix:** Use a dedicated “Overlay controls” strip with clearer verbs (`Close overlay`, `Write mappings`) and consistent key labeling.

6. **severity: low** — **screenshot/page: `midi-io.png`** — **issue:** Selected-device badges (`DEF`, `SEL`) are tightly packed into card corners and read like clipped micro-labels at this size. **brief suggested fix:** Merge into one clearer badge (e.g., `DEFAULT SELECTED`) or move badges to a fixed metadata row in each card header.

7. **severity: medium** — **screenshot/page: `routing.png`** — **issue:** Right-edge action chips (`SELECT`, `TOGGLE`, `TAP +/-`) are visually inconsistent and unclear in meaning vs the `+/-` mini buttons; interaction model is not obvious at a glance. **brief suggested fix:** Standardize control patterns per row (single primary action + optional stepper), and add concise inline helper text for special actions like tap value editing.