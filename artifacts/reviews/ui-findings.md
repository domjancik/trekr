Findings

1. severity: **high**; screenshot/page: **mappings-overlay.png**; issue: The top-right header shows `SCOPE` with no visible value, which looks like missing/clipped state text and weakens state communication. suggested fix: Always render an explicit scope value (for example `GLOBAL`/`ACT TRACK`) or hide the label when empty.

2. severity: **medium**; screenshot/page: **mappings.png**; issue: The row density is very high and columns are visually compressed, making trigger/action/scope hard to scan quickly. suggested fix: Increase row height slightly and add stronger column separation or alternating row backgrounds.

3. severity: **medium**; screenshot/page: **mappings.png**; issue: Bottom keyboard-help strip is crowded (`SHIFT+LEFT/RIGHT FIELD 0/E ADJUST ENTER LEARN/TOGGLE`) and reads as a single run-on block. suggested fix: Group shortcuts into clearly separated chunks with consistent spacing and dividers.

4. severity: **medium**; screenshot/page: **midi-io.png**; issue: Device cards contain very large unlabeled empty regions, which makes users unsure whether content failed to load or if those are intentional meters/panels. suggested fix: Add explicit labels/placeholders (for example `No activity`, `No channels shown`) or reduce unused panel height.

5. severity: **low**; screenshot/page: **midi-io.png**; issue: `DEF SEL` badges are pushed tight to card edges and look close to clipping. suggested fix: Add right padding and a fixed badge container width for cleaner alignment.

6. severity: **medium**; screenshot/page: **timeline.png**; issue: Control hierarchy is weak at the top: many toggles share similar visual weight, so primary state (play/record/mode/quant) is hard to identify quickly. suggested fix: Promote critical transport/mode states with stronger contrast grouping and demote secondary settings.

7. severity: **low**; screenshot/page: **timeline.png / timeline-focused.png**; issue: Small numeric/label text in loop headers is at the edge of readability, especially in dense columns. suggested fix: Increase font size one step or reduce non-essential header tokens per column.

8. severity: **low**; screenshot/page: **routing.png**; issue: Routing screen was not visible in the provided attachments, so layout/state QA for that page is incomplete. suggested fix: Re-export and attach `routing.png` in a supported format so it can be reviewed consistently.