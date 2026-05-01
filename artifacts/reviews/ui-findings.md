Findings

1. severity: high; screenshot/page: `routing.png`; issue: the `INPUT FX` and `OUTPUT FX` cards use many tiny labels (`P2`, `MORE`, `KIND`, `SET`, `TGL`) with equal visual weight, so the primary action path is unclear and reads as dense control noise; brief suggested fix: increase hierarchy by enlarging section titles and primary fields, de-emphasize secondary toggles/buttons, and group each row into clearer “value + action” clusters.

2. severity: medium; screenshot/page: `timeline.png`; issue: top toolbar state is hard to parse quickly because many chips (`PLAY`, `REC`, `REC MODE`, `REC WRAP`, `SONG LOOP`, etc.) have similar size/contrast and mixed semantics (mode vs status vs action); brief suggested fix: separate status indicators from mode/action controls with spacing and distinct styling (for example, status badges vs actionable buttons).

3. severity: medium; screenshot/page: `timeline-focused.png`; issue: the focused-track state is subtle (`TRACK T1` + layout change) and can be missed, which risks orientation loss; brief suggested fix: add a stronger page-level focused-state banner/highlight and a clearer “focused view” label near the main header.

4. severity: medium; screenshot/page: `mappings.png`; issue: row density is very high and columns (`TYPE`, `DEVICE`, `SOURCE`, `TARGET`, `SCOPE`, `ON`) are cramped, making scan and edit intent slower; brief suggested fix: increase row height/padding slightly and strengthen column separators/headers so trigger→action→scope is easier to follow.

5. severity: medium; screenshot/page: `mappings-overlay.png`; issue: overlay table and page-level footer hints are both visible with similar prominence, creating competing instruction layers; brief suggested fix: in overlay mode, dim or simplify background/global hints and prioritize only overlay-relevant commands.

6. severity: low; screenshot/page: `midi-io.png`; issue: large empty gray interiors inside device cards look like inactive placeholders, which can be misleading when cards are actually selectable/configurable; brief suggested fix: add explicit empty-state text or light metadata rows (ports/channels/status) so the area communicates purpose.

7. severity: low; screenshot/page: `mappings.png`; issue: right-top counter text (`ROWS 1 / 30`) and nearby labels are visually detached from the active table context; brief suggested fix: align count/scope indicators directly with the table header row and tighten spacing so context is immediate.

8. severity: low; screenshot/page: `timeline.png` and `timeline-focused.png`; issue: some tiny legend/utility text (for example right-side tempo/tap cluster and small footer hints) is near readability limits at this density; brief suggested fix: raise font size one step or increase contrast/letter spacing for secondary utility text.