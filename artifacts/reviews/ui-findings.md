Findings

1. severity: `high` | screenshot/page: `timeline` | issue: The top control bar is extremely dense and many labels (`MODE OVERDUB`, `RECOMPR EXTEND`, `NOTEADD OFF`, `LINK OFF`, `SYNC OFF`) read as a single uninterrupted strip, which makes state scanning slow and easy to misread. | brief suggested fix: Group related controls with more spacing or dividers, and promote the most important transport/song states with stronger contrast or larger blocks.

2. severity: `high` | screenshot/page: `timeline` | issue: Track headers are visually cramped and repetitive; labels like `THRU`, `TRK`, `MUTE`, `SOLO`, `ARM`, `REC` compete at the same weight, so the active track/state is not obvious at a glance. | brief suggested fix: Reduce header noise by demoting secondary toggles and give the selected track a stronger container/background treatment than individual cells.

3. severity: `medium` | screenshot/page: `mappings` | issue: The table is so horizontally compressed that column meaning is hard to parse quickly, especially with repeated `--`, long targets, and very narrow `ON` cells on the right. | brief suggested fix: Increase padding between columns, widen the state/scope area, and consider truncation rules or stronger column separators.

4. severity: `medium` | screenshot/page: `mappings-overlay` | issue: The overlay header actions (`F5 CLOSE`, `W WRITE`) are understated and easy to miss, so it is not immediately clear whether this is a modal, a filtered list, or an editable state. | brief suggested fix: Make the overlay title bar more explicit with clearer action grouping and a stronger modal treatment for close/edit affordances.

5. severity: `medium` | screenshot/page: `mappings` and `mappings-overlay` | issue: Status communication is weak: `TAP MODE: READ ONLY`, `TAP LEARN: IDLE`, and `TAP DIRECT MAP` look visually similar even though they imply different modes and levels of actionability. | brief suggested fix: Use distinct visual styles for passive status, current mode, and actionable controls.

6. severity: `medium` | screenshot/page: `midi-io` | issue: The input/output cards have large empty bodies with tiny labels, so the dominant visual impression is blank panels rather than a clear device selection screen. | brief suggested fix: Shrink unused body height or add clearer per-device metadata/actions so each card communicates purpose faster.

7. severity: `medium` | screenshot/page: `routing` | issue: The right-edge action areas (`TAP +/-`, `SELECT`, `TOGGLE`) are visually detached from the values they affect, which makes the interaction model feel ambiguous. | brief suggested fix: Bind actions more tightly to the editable value, either by integrating them into the field chrome or adding clearer affordance labels.

8. severity: `low` | screenshot/page: `routing` | issue: Color hierarchy is inconsistent across rows; bright fill colors imply semantic meaning, but the meaning is not obvious and the saturation competes with selection state. | brief suggested fix: Reserve strong colors for state changes or category coding with a visible legend/pattern, and keep default rows more neutral.

9. severity: `low` | screenshot/page: `midi-io` and `routing` | issue: Section subtitles (`SELECT DEFAULT INPUTS AND OUTPUTS`, `INPUT AND OUTPUT ROUTING FOR THE ACTIVE TRACK`) are small and low-emphasis relative to surrounding chrome, so page purpose is easy to miss. | brief suggested fix: Increase subtitle contrast/size or place it closer to the page title as a clearer explanatory subheader.

10. severity: `low` | screenshot/page: `all pages` | issue: Bottom-right function hints (`F5 MAPPINGS`, `F7 DISCOVER`, `F8 DIRECT`) are consistently de-emphasized, making important global actions look like footer noise. | brief suggested fix: Give active/global shortcuts a more deliberate hierarchy, especially when they open major views or modes.