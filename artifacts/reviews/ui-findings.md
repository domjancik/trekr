Findings

1. severity: high | screenshot/page: `timeline.png` | issue: Track header labels are visibly clipped (`TRA...`), so key state/info is hidden at first glance. | brief suggested fix: Increase header width or shorten label tokens with meaningful abbreviations (e.g., `TRACK ARM`) and expose full text on focus/hover.

2. severity: medium | screenshot/page: `timeline.png` | issue: Top control rows are overly dense with similar visual weight, making primary vs secondary states hard to scan quickly. | brief suggested fix: Group controls into clearer sections (transport, loop, quantize), add spacing between groups, and emphasize active states with stronger contrast/background treatment.

3. severity: medium | screenshot/page: `mappings.png` | issue: Bottom shortcut/help strip is cramped and low-legibility; commands run together and are hard to parse quickly. | brief suggested fix: Add separators/padding, reduce command count per row (or wrap to two rows), and increase contrast for key actions.

4. severity: medium | screenshot/page: `mappings.png` | issue: Table hierarchy is weak because row contents, headers, and status fields (`ON`, scope) are visually similar, slowing scan speed. | brief suggested fix: Strengthen header styling and de-emphasize repeated status text; use alignment and subtle column tinting to separate identity/action/status columns.

5. severity: low | screenshot/page: `mappings-overlay.png` | issue: Overlay top-right metadata (`ROWS 1-19 / 30`, `SCOPE`) feels detached from the table and easy to miss. | brief suggested fix: Move metadata into a single aligned header row directly above column labels.

6. severity: medium | screenshot/page: `midi-io.png` | issue: Large empty gray device panels read like placeholders rather than actionable lists, making interaction intent unclear. | brief suggested fix: Add explicit list affordances (rows, selection highlight style, empty-state copy/instructions) and reduce unused panel area.

7. severity: low | screenshot/page: `midi-io.png` | issue: Small status chips (`DEF`, `SEL`) are visually cramped near card edges and easy to overlook. | brief suggested fix: Increase chip padding/margins and keep status badges in a consistent reserved location across cards.

8. severity: medium | screenshot/page: `routing.png` | issue: Row action controls are inconsistent (`TAP +/-` vs `SELECT` vs `TOGGLE`) without strong contextual cues, which can feel misleading. | brief suggested fix: Standardize control patterns per field type and add micro-labels/tooltips clarifying when a control is stepped, selected, or toggled.