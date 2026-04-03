Findings

1. **severity: high** — **screenshot/page: timeline.png** — **issue:** The screen is extremely dense, with many similarly styled controls and labels at the top (`PLAY OFF`, `RECORD OFF`, `MODE OVERDUB`, wrap/song/tempo/noteread) so active state is hard to parse quickly. **suggested fix:** Increase visual hierarchy for active vs inactive states (stronger contrast, grouped sections, and clearer separators between transport, loop, and timing controls).

2. **severity: high** — **screenshot/page: timeline.png** — **issue:** Several per-track header areas look cramped, with tiny text packed next to step numbers and mode tags, making labels feel near-clipped and hard to scan. **suggested fix:** Add horizontal padding and/or reduce header content per row; move secondary metadata to a second line or tooltip/help line.

3. **severity: medium** — **screenshot/page: mappings.png** — **issue:** The mappings table rows are very tight, and the right-side `SCOPE` + `ON` columns are visually compressed, which slows row-level comprehension. **suggested fix:** Widen right-most columns slightly and add stronger column boundaries or zebra striping to improve row tracking.

4. **severity: medium** — **screenshot/page: mappings.png** — **issue:** Top status chips (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) read like equal-priority buttons/statuses, so interaction/state intent is ambiguous. **suggested fix:** Differentiate status vs action styling (e.g., status pill style for state, button style for actionable controls).

5. **severity: medium** — **screenshot/page: mappings-overlay.png** — **issue:** Large empty area below visible rows with `ROWS 1-19 / 30` provides weak pagination/scroll communication; users may miss that more rows exist. **suggested fix:** Add explicit pagination affordance (scrollbar, “more rows below” hint, or page controls).

6. **severity: low** — **screenshot/page: mappings-overlay.png** — **issue:** Header actions (`F5 CLOSE`, `W WRITE`) are subtle and easy to overlook as key actions. **suggested fix:** Increase prominence/contrast and group them as a small action bar.

7. **severity: medium** — **screenshot/page: midi-io.png** — **issue:** Inputs and outputs use different panel structures (single large card vs stacked cards), creating inconsistent spacing and unclear parity between sections. **suggested fix:** Normalize list/card layout and spacing across both sides, or clearly label why structures differ (e.g., “active output” vs “available outputs”).

8. **severity: medium** — **screenshot/page: routing.png** — **issue:** Repeated micro-controls (`-`, `+`, `SET`, `TGL`) are visually similar across many rows, making control purpose and current mode easy to misread. **suggested fix:** Add clearer grouping and inline state descriptors (e.g., “toggle monitoring: ON”) and reserve stronger color for current state.

9. **severity: low** — **screenshot/page: timeline-focused.png** — **issue:** Focused track view still inherits dense top-control rows with minimal spacing, weakening the “focused” mode clarity. **suggested fix:** In focused mode, simplify the top bar and hide non-essential global controls to reinforce the active context.