Findings

1. severity: **high** | screenshot/page: **timeline.png** | issue: Track columns are too dense; labels, step markers, and note blocks compete at very small sizes, so scanning per-track state is slow. | brief suggested fix: Increase minimum column width (or reduce visible track count), and move secondary metadata into a compact toggle/tooltip layer.

2. severity: **medium** | screenshot/page: **timeline.png** | issue: Loop header text in each track (`1 2 3 4 5 6 +2 ..`) appears cramped/truncated, which reads like clipping rather than intentional shorthand. | brief suggested fix: Shorten the token set, add spacing, or wrap it into a dedicated “loop stats” row with clearer separators.

3. severity: **medium** | screenshot/page: **timeline-focused.png** | issue: Focused-vs-all-tracks state is not strongly communicated; the view change relies mostly on subtitle text and layout inference. | brief suggested fix: Add an explicit mode badge (for example `FOCUSED VIEW`) near the main title and a stronger active control style for the selected scope button.

4. severity: **medium** | screenshot/page: **mappings.png** | issue: The bottom shortcut strip is visually crowded; many equal-weight tokens make key actions hard to prioritize quickly. | brief suggested fix: Group shortcuts by function (navigation/edit/learn), add spacing between groups, and emphasize 2–3 primary actions.

5. severity: **low** | screenshot/page: **mappings-overlay.png** | issue: Scope labeling is weak; `SCOPE` is separated from row values and can be read as a floating note instead of a column header. | brief suggested fix: Align `SCOPE` directly as a third column header with the values and matching header styling.

6. severity: **medium** | screenshot/page: **midi-io.png** | issue: Large empty gray regions inside device cards dominate visual weight and look like missing content or disabled areas. | brief suggested fix: Replace with compact status summaries (channels/activity/latency) or reduce panel height when no detail content is present.

7. severity: **low** | screenshot/page: **midi-io.png** | issue: `DEF SEL` tags are tight to card edges and low-clarity at a glance, which weakens default vs selected state readability. | brief suggested fix: Convert to clearer chips (`DEFAULT`, `SELECTED`) with consistent padding and stronger contrast.

8. severity: **medium** | screenshot/page: **routing.png** | issue: Control semantics are mixed (`SET`, `TGL`, `+`, `ADJ`) without clear affordance hierarchy, so users must infer whether a field is value-edit, action, or toggle. | brief suggested fix: Normalize control patterns (one style per action type) and add concise inline legends/tooltips for each control family.