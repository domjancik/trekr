Findings

1. **severity: high** | **screenshot/page:** `timeline.png` and `timeline-focused.png` | **issue:** Top control bands are overly dense (many tiny toggles/labels in one horizontal strip), so state is hard to scan quickly and key mode info gets lost. | **brief suggested fix:** Group controls into 2–3 labeled clusters with more vertical spacing and stronger emphasis on active states.

2. **severity: high** | **screenshot/page:** `timeline.png` | **issue:** Per-track column headers and control labels are very small and visually crowded, making track-level actions difficult to understand at a glance. | **brief suggested fix:** Increase header text size/line height and reduce simultaneous visible metadata in each track card.

3. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Bottom FX chips in track columns appear truncated/clipped (long labels compressed into narrow chips). | **brief suggested fix:** Add truncation with ellipsis + tooltip/full label region, or increase chip width/allow wrap in that row.

4. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Table columns are inconsistent in information density (very wide `TARGET`, very narrow `ON`), which weakens scan rhythm and makes status feel secondary. | **brief suggested fix:** Rebalance column widths and promote status columns (`SCOPE`, `ON`) with clearer alignment and stronger visual affordance.

5. **severity: medium** | **screenshot/page:** `mappings-overlay.png` vs `mappings.png` | **issue:** Overlay uses different column vocabulary/layout (`TRIGGER/ACTION`) than the main mappings page (`TYPE/DEVICE/SOURCE/TARGET`), causing context switching. | **brief suggested fix:** Keep naming and column structure consistent between main page and overlay, or add explicit mapping between terms.

6. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Device cards contain large empty gray blocks with minimal meaning, so users can’t quickly tell enabled/armed/default states beyond tiny tags. | **brief suggested fix:** Replace empty fill areas with concise status rows (connection, channel, activity) and stronger selected/default badges.

7. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Many repeated small `SET`/`+` controls are ambiguous about scope (field-level vs section-level action). | **brief suggested fix:** Add clearer grouping labels and inline hints (e.g., “Apply to field”), and differentiate primary vs secondary actions visually.

8. **severity: low** | **screenshot/page:** all pages | **issue:** Footer shortcut bar (`F5/F7/F8`) has weak hierarchy and low prominence relative to the amount of workflow-critical interaction it represents. | **brief suggested fix:** Increase contrast/weight for active or recommended shortcut and align shortcut styling with current page state.