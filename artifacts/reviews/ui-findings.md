Findings

1. `severity: high` `mappings.png` `mappings-overlay.png` issue: The app has two visually different mappings views for the same data, but the relationship between them is unclear; “F5 MAPPINGS” in the footer and “F5 CLOSE” in the overlay do not make it obvious whether this is a modal, alternate mode, or filtered view. brief suggested fix: Make the overlay state explicit with a stronger title/subtitle like “Mappings Overlay (Quick View)” and dim or disable the underlying navigation/state cues.

2. `severity: medium` `routing.png` issue: The right-edge action labels (`TAP +/-`, `SELECT`, `TOGGLE`) read like field values because they share the same row structure and sit inside the same control band. brief suggested fix: Separate actions into distinct button styling or a dedicated action column with clearer affordance.

3. `severity: medium` `timeline.png` issue: The top control bar is densely packed with similarly styled pills (`PLAY OFF`, `REC OFF`, `MODE OVERDUB`, `RECOMPR EXTEND`, `SONGLOOP ON`, etc.), which weakens hierarchy and makes it hard to identify the primary state quickly. brief suggested fix: Group controls by function and use stronger visual emphasis for transport/state-critical items versus secondary settings.

4. `severity: medium` `timeline.png` issue: Track headers are crowded and repetitive (`THRU`, `TRACK`, `MUTE`, `SOLO`, `ARM`, `REC`) with little spacing, so scanning across six columns is slow and error-prone. brief suggested fix: Increase spacing between header controls and simplify repeated labels or move secondary toggles into a cleaner sub-row.

5. `severity: medium` `midi-io.png` issue: The device cards contain large empty gray areas that look like disabled inputs or missing content, which makes the state of each device ambiguous. brief suggested fix: Replace empty fill blocks with clearer status content, meter placeholders, or concise metadata so the cards communicate purpose immediately.

6. `severity: medium` `mappings.png` issue: The dense table rows have minimal vertical padding and weak separation, so fields like source, target, scope, and enabled state blur together during quick scanning. brief suggested fix: Add slightly more row height or stronger column separation, and visually emphasize the primary columns.

7. `severity: low` `mappings.png` issue: The header controls (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) are close in weight and treatment to non-interactive labels like `ROWS 1 / 30`, which reduces state clarity. brief suggested fix: Differentiate active controls from status text with clearer button framing and quieter status styling.

8. `severity: low` `mappings-overlay.png` issue: The top-right metadata (`ROWS 1-19 / 30`, `SCOPE`) feels detached from the table and can be mistaken for column content rather than view-level status. brief suggested fix: Align it into a dedicated header/status bar above the table with clearer separation.

9. `severity: low` `midi-io.png` issue: The `DEF` and `SEL` badges are small and packed tightly into card corners, making default vs selected state easy to miss. brief suggested fix: Increase contrast/spacing and use more explicit labels or a single combined state treatment.

10. `severity: low` `routing.png` issue: Color coding is strong, but the meaning of each row color is not self-evident, so first-time users may not understand whether color indicates device type, selection, or editability. brief suggested fix: Add a small legend or more explicit labels so color is supporting meaning rather than carrying it alone.