Findings

1. severity: **high** | screenshot/page: **timeline.png** | issue: The page is visually overloaded (6 track columns + loop lanes + FX rows), making scan order and primary state hard to parse quickly. | brief suggested fix: Reduce simultaneous detail in the default view (collapse secondary metadata/FX rows, increase section contrast, and emphasize one primary focus region).

2. severity: **high** | screenshot/page: **timeline.png** | issue: Multiple labels appear clipped/truncated (`SM...`, `RT...`, `TM...`, compact control chips on track headers/footers), which hides meaning. | brief suggested fix: Enforce min column widths or responsive truncation with explicit ellipsis + tooltip/expanded label on focus.

3. severity: **medium** | screenshot/page: **timeline-focused.png** | issue: Header/control text density is still very tight; state chips (`ARM/REC/MUT/SOL`, transport toggles) blend together and are hard to distinguish at a glance. | brief suggested fix: Add spacing between control groups, stronger active/inactive contrast, and clearer group separators.

4. severity: **medium** | screenshot/page: **routing.png** | issue: The panel uses many tiny controls (`+`, `SET`, `TGL`) with similar visual weight, so control intent is unclear and action hierarchy is weak. | brief suggested fix: Differentiate control types (value adjust vs commit vs toggle) with distinct button styles and labels (for example `Adjust`, `Apply`, `Toggle`).

5. severity: **medium** | screenshot/page: **routing.png** | issue: Two large sections (`INPUT FX` and `OUTPUT FX`) are information-dense and visually similar, increasing cognitive load and misread risk. | brief suggested fix: Increase section differentiation (clearer titles, more spacing, and stronger color/contrast boundaries for each block).

6. severity: **medium** | screenshot/page: **mappings.png** | issue: Table rows are very compact and column boundaries are subtle; long action labels dominate while metadata columns (`TYPE/DEVICE/SCOPE/ON`) feel cramped. | brief suggested fix: Rebalance column widths and strengthen column separators so trigger, action, and scope are easier to scan.

7. severity: **low** | screenshot/page: **mappings-overlay.png** | issue: Overlay command hints (`F5 CLOSE`, `W WRITE`) are terse and easy to miss; state communication for what mode the user is in is weak. | brief suggested fix: Add a clearer mode banner/subtitle (for example `Overlay Mode: Read-Only`) and group key hints in a dedicated help strip.

8. severity: **low** | screenshot/page: **midi-io.png** | issue: Default-selection badges (`DEF`, `SEL`) are small and visually crowded at row ends, which reduces immediate state readability. | brief suggested fix: Increase badge spacing/size and add a single explicit row-level state label (for example `Selected Default Output`).