Findings

1. severity: **medium**; screenshot/page name: **timeline.png**; issue: The top-row state chips (`PLAY`, `REC`, `REC MODE`, etc.) are visually similar in weight and color despite different importance, so active transport/record state is hard to scan quickly. brief suggested fix: Increase contrast/priority for live-critical states (transport/record) and de-emphasize secondary mode chips.

2. severity: **medium**; screenshot/page name: **timeline.png**; issue: Per-track headers are dense and repetitive (`ARM/REC/MUT/SOL`, `ADD INPUT FX`, `THRU TRACK`) with minimal spacing separation, making column boundaries cognitively heavy. brief suggested fix: Add slightly more vertical spacing between header rows and content, and use stronger grouping separators per track column.

3. severity: **low**; screenshot/page name: **timeline-focused.png**; issue: The focused view still shows many small controls at top with near-equal emphasis, so “focused track” mode does not feel meaningfully simplified. brief suggested fix: Collapse or dim non-essential global controls in focused mode and elevate the focused track context label.

4. severity: **high**; screenshot/page name: **routing.png**; issue: Label clipping/ambiguity in right-side FX panels (`KIND`, `SEM`, `VEL`, `MORE`) plus tiny `+`/`SET` affordances makes parameter editing intent unclear at a glance. brief suggested fix: Widen label/value cells or shorten tokens consistently, and add clearer distinction between value field vs action button.

5. severity: **medium**; screenshot/page name: **routing.png**; issue: The page has several similarly styled boxes (`SIGNAL`, `REC/MON`, `INPUT FX`, `OUTPUT FX`) with weak hierarchy, so primary workflow order is unclear. brief suggested fix: Introduce stronger section hierarchy (size/contrast/order cues) and a clearer left-to-right signal-flow emphasis.

6. severity: **medium**; screenshot/page name: **midi-io.png**; issue: Large empty device cards dominate space while key status/action text (`DEF`, `SEL`) is very small and low-salience, making state communication weak. brief suggested fix: Promote selected/default badges with stronger contrast and scale down empty interior area or add concise device meta info.

7. severity: **low**; screenshot/page name: **mappings.png**; issue: Bottom hint bar mixes shortcuts and actions in one continuous strip with little grouping, reducing quick discoverability. brief suggested fix: Split hints into labeled groups (navigation/edit/mode) with spacing or dividers.

8. severity: **medium**; screenshot/page name: **mappings-overlay.png**; issue: Overlay table column hierarchy is weak (`TRIGGER`, `ACTION`, `SCOPE` close in tone/weight), and row selection highlight is subtle against dense rows. brief suggested fix: Increase header contrast/weight and strengthen selected-row background or border for immediate focus tracking.