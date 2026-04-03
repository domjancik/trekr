Findings

1. severity: high | screenshot/page: `timeline.png` | issue: Dense multi-row control bars at the top (play/record/mode/link/launch/quant/peers) compete visually with track content, so primary actions vs status are hard to parse quickly. | brief suggested fix: Group top controls into clearly separated sections (transport, loop, sync) with stronger spacing and one dominant active-state style.

2. severity: high | screenshot/page: `timeline-focused.png` | issue: Header/control alignment is inconsistent (`ARM/REC/MUT/SOL` row, FX row, and track headers), which makes it unclear which controls apply globally vs per-track. | brief suggested fix: Align all control rows to a consistent column grid and add explicit section labels (`Global`, `Track 1`) to clarify scope.

3. severity: medium | screenshot/page: `routing.png` | issue: The page has too many similarly styled small controls (`SET`, `TGL`, `+/-`) with weak affordance, making intent and interaction outcome unclear. | brief suggested fix: Differentiate control types visually (button, toggle, stepper) and add short inline helper labels/tooltips for each row.

4. severity: medium | screenshot/page: `midi-io.png` | issue: Very large empty list areas with minimal row detail make the screen feel blank and reduce scanability; selected/default badges are tiny and easy to miss. | brief suggested fix: Increase row information density (status, channel, role), enlarge/bold selected/default markers, and reduce unused panel space.

5. severity: medium | screenshot/page: `mappings.png` | issue: Column semantics are unclear at a glance (`TYPE/DEVICE/SOURCE/TARGET/SCOPE/ON`) because values are tightly packed and separators are visually weak. | brief suggested fix: Strengthen column boundaries, increase padding between columns, and emphasize header contrast to improve table readability.

6. severity: medium | screenshot/page: `mappings-overlay.png` | issue: Overlay hierarchy is weak: list content, title, and status metadata (`ROWS 1-19 / 30`, `SCOPE`) are visually similar, so context/state is not immediately obvious. | brief suggested fix: Create a clearer overlay header band with stronger typography and dedicated state chips for row range and scope.

7. severity: low | screenshot/page: `mappings.png` and `mappings-overlay.png` | issue: Some trigger/action labels are visually cramped (e.g., combo key strings and long action names), increasing risk of perceived clipping even when technically visible. | brief suggested fix: Use truncation with ellipsis plus full-value reveal on focus/hover, or widen trigger/action columns slightly.

8. severity: low | screenshot/page: all pages | issue: Active/inactive states rely heavily on subtle color changes and small text tokens, which slows recognition in dense screens. | brief suggested fix: Add a second state signal (icon, fill pattern, or stronger border weight) for active, selected, and armed states.