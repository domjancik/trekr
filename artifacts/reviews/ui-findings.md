Findings

1. **severity: medium** | **screenshot/page: `timeline.png` (Timeline)** | **issue:** Top toolbar is very dense (`Q`, `PEERS`, `LINK`, `LINK SYNC`, BPM, tap) with weak visual grouping, so scanning state quickly is hard. | **brief suggested fix:** Add clearer grouped containers/subheaders (transport vs sync vs tempo), increase horizontal padding between groups, and emphasize active values with stronger contrast.

2. **severity: high** | **screenshot/page: `timeline-clip-align.png` (Timeline Clip Align modal)** | **issue:** Modal overlaps core track content without dimming or separation, so it competes with background data and is easy to misread as inline panel content. | **brief suggested fix:** Add stronger modal layering (backdrop dim, thicker border/shadow) and mute background track contrast while modal is open.

3. **severity: medium** | **screenshot/page: `timeline-clip-align.png` (Timeline Clip Align modal)** | **issue:** Field labels/values in the modal are compact and visually similar (`START`, `END`, `LENGTH`, `DEST`, `MODE`, `LOOP`), reducing hierarchy between editable value and row label. | **brief suggested fix:** Differentiate label/value typography or color and increase row spacing by a few pixels for faster parsing.

4. **severity: medium** | **screenshot/page: `mappings.png` (Mappings)** | **issue:** Row content is tightly packed; `TYPE`, `DEVICE`, `SOURCE`, `TARGET`, `SCOPE`, and `ON` columns have minimal breathing room, making long-list scanning fatiguing. | **brief suggested fix:** Increase row height and column padding slightly; consider stronger alternating row backgrounds for readability.

5. **severity: low** | **screenshot/page: `mappings-overlay.png` (Mappings Overlay)** | **issue:** Header action hints (`F5 CLOSE`, `W WRITE`) read like plain text and don’t stand out as interactive mode controls. | **brief suggested fix:** Style key hints as compact badges/buttons with stronger contrast and consistent spacing from the title.

6. **severity: medium** | **screenshot/page: `midi-io.png` (MIDI I/O)** | **issue:** Large empty device card interiors dominate visual weight, while actionable state (`DEF/SEL`) is small and easy to miss. | **brief suggested fix:** Reduce empty fill prominence and enlarge/relocate status chips so selected/default state is immediately visible.

7. **severity: low** | **screenshot/page: `routing.png` (Routing)** | **issue:** Some control labels are terse/ambiguous (`P2`, `MORE`, `KIND`, `SET`) and repeated across sections without contextual differentiation. | **brief suggested fix:** Use clearer labels/tooltips (e.g., `Param 2`, `More Params`) and section-specific prefixes to reduce ambiguity.

8. **severity: medium** | **screenshot/page: `timeline-focused.png` (Focused Track + Loop Detail)** | **issue:** Active track state is present but easy to miss against similarly saturated neighboring panels; focus hierarchy is weaker than expected for “focused” mode. | **brief suggested fix:** Increase focused panel prominence (stronger border/glow/contrast) and de-emphasize non-focused areas slightly.