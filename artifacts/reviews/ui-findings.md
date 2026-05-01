Findings

1. severity: **medium**; screenshot/page: **timeline.png**; issue: the top-right transport block (`120`, `TAP`, `*`) is cramped and reads like one fused control, so button boundaries and click targets are unclear; brief suggested fix: add horizontal spacing/padding between controls and use stronger per-control borders/background contrast.

2. severity: **medium**; screenshot/page: **timeline.png**; issue: per-track headers (`ARM REC MUT SOL`) are visually detached from the actual lane content, making it slower to map controls to a specific track; brief suggested fix: increase vertical association (shared background strip or tighter grouping) and reduce repeated header noise.

3. severity: **low**; screenshot/page: **timeline-focused.png**; issue: the `↑↓✕` micro-controls at far right of FX rows are too small/cryptic to parse quickly; brief suggested fix: enlarge hit areas and add short labels or tooltips for each icon action.

4. severity: **medium**; screenshot/page: **mappings.png**; issue: very dense row packing with low row-height separation makes scanning trigger/action/scope columns fatiguing; brief suggested fix: increase row height by a few pixels and add subtle zebra striping or stronger horizontal separators.

5. severity: **low**; screenshot/page: **mappings.png**; issue: bottom shortcut legend uses many equal-weight pills, so primary actions (`WRITE`, `REMOVE`) don’t stand out; brief suggested fix: visually prioritize destructive/primary actions with stronger color semantics and leave secondary hints lower contrast.

6. severity: **medium**; screenshot/page: **mappings-overlay.png**; issue: overlay title/action hints (`F5 CLOSE`, `W WRITE`) are not clearly separated from table headers, weakening hierarchy; brief suggested fix: create a distinct overlay header band with clearer spacing before the data grid.

7. severity: **high**; screenshot/page: **midi-io.png**; issue: large empty light-gray panes in device cards look like missing content/disabled state, making users unsure whether ports are loaded correctly; brief suggested fix: replace blank panes with explicit status text (e.g., “No channels listed” / “Connected, idle”) and clearer section labels.

8. severity: **medium**; screenshot/page: **midi-io.png**; issue: `DEF SEL` tags are tiny and crowded into card corners, so selection/default state is easy to miss; brief suggested fix: move state badges into a consistent, larger badge area with stronger contrast.

9. severity: **medium**; screenshot/page: **routing.png**; issue: multiple `SET`/`TGL` buttons repeat in tight grids without enough contextual distinction, increasing misclick risk; brief suggested fix: align each action button closer to its specific field label and vary button styling by action type (`set` vs `toggle`).

10. severity: **low**; screenshot/page: **routing.png**; issue: section hierarchy is busy (many colored panels of similar weight), so “Signal”, “Rec/Mon”, “Input FX”, and “Output FX” compete equally for attention; brief suggested fix: reduce accent intensity on secondary sections and reserve strongest emphasis for the active edit area only.

11. severity: **low**; screenshot/page: **global (all screenshots)**; issue: top nav active-state contrast is subtle and similar across tabs, which slows orientation when switching pages quickly; brief suggested fix: increase active-tab contrast/brightness and optionally add a stronger active indicator bar.

12. severity: **low**; screenshot/page: **global (all screenshots)**; issue: date/hash block near logo is visually dense and small relative to other chrome, adding noise without clear priority; brief suggested fix: reduce prominence (smaller contrast) or move it to a dedicated status area.