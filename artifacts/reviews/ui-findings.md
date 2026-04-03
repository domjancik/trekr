Findings

1. **severity: high** | **screenshot/page:** `timeline.png` | **issue:** Primary content is very dense (6 track columns + many micro-controls), and key labels/actions are too small to scan quickly, weakening hierarchy. | **brief suggested fix:** Increase font size/line height for control bars and reduce simultaneous controls per row (progressive disclosure or collapsible sections).

2. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Several labels appear truncated or ambiguous (`S...`, `U...`, compact FX labels), making state and control intent unclear. | **brief suggested fix:** Reserve minimum label width and use explicit abbreviated tokens with tooltips/help legend.

3. **severity: medium** | **screenshot/page:** `timeline-focused.png` | **issue:** Focused mode still has multiple thin control rows with low visual separation from the main canvas, so focus state is not strongly communicated. | **brief suggested fix:** Strengthen focused-state treatment (clear section header, stronger contrast band, and deemphasize non-essential controls).

4. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Repeated `SET` and `TGL` buttons are visually identical across different parameter groups, which is easy to misread and potentially misleading. | **brief suggested fix:** Add contextual button labels/icons (`Set Input`, `Toggle Mon`) or group-local heading badges.

5. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Spacing is inconsistent between left signal block and right FX/monitor blocks; some rows feel tighter than others, reducing rhythm/readability. | **brief suggested fix:** Standardize vertical row height and inter-group padding using a fixed spacing scale.

6. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Bottom shortcut hint bar is overloaded and cryptic (`W WRITE`, `F8 DIRECT`, `N NEW`, etc.), with weak distinction between actionable vs informational text. | **brief suggested fix:** Split into “Actions” and “Hints” zones and increase contrast/spacing for active actions.

7. **severity: low** | **screenshot/page:** `mappings-overlay.png` | **issue:** Right-side metadata (`ROWS 1-19 / 30`, `SCOPE`) is detached from the table headers and easy to miss. | **brief suggested fix:** Align metadata with header row or place in a dedicated overlay status strip.

8. **severity: low** | **screenshot/page:** `midi-io.png` | **issue:** Device badges like `DEF SEL` are very small and crowded against long device names, reducing clarity of selected/default state. | **brief suggested fix:** Increase badge padding/size and move status chips into a consistent right-aligned status column.