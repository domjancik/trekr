Findings

1. severity: **high** | screenshot/page: **timeline.png** | issue: The per-track lane headers and control chips (`ARM/REC/MUT/SOL`, `ADD INPUT FX`, loop badges, tiny `+ -` controls) are densely packed with minimal separation, making it hard to parse track state at a glance. | brief suggested fix: Increase vertical spacing between header rows, group related controls with clearer containers, and hide/de-emphasize secondary controls until focus/selection.

2. severity: **high** | screenshot/page: **timeline-focused.png** | issue: Focused mode still shows many small, low-contrast micro-labels and symbols (`↑↓x`, loop badge, tiny numbers) that are easy to miss and feel cryptic. | brief suggested fix: Promote key state labels (active track, loop, armed) to larger/high-contrast text and replace symbolic-only controls with short text labels or tooltips.

3. severity: **medium** | screenshot/page: **mappings.png** | issue: The bottom shortcut legend is visually crowded and reads like one continuous strip, so action groups are unclear. | brief suggested fix: Split shortcuts into grouped clusters (navigation/edit/mapping modes) with spacing or separators and reduce token count shown by default.

4. severity: **medium** | screenshot/page: **mappings-overlay.png** | issue: The overlay lacks strong visual distinction from the base page (similar palette/borders), so modal state is not immediately obvious. | brief suggested fix: Add a darker scrim, thicker/bright modal frame, and a stronger title/state line to clearly communicate “overlay active.”

5. severity: **medium** | screenshot/page: **midi-io.png** | issue: Large empty card interiors dominate the page while key state indicators (`DEF/SEL`) are tiny, weakening hierarchy and making selected/default routing status easy to miss. | brief suggested fix: Reduce empty fill area and elevate status chips (size/contrast/position) near device names; add a compact summary row for selected defaults.

6. severity: **medium** | screenshot/page: **routing.png** | issue: Mixed control styles (`SET`, `TGL`, inline `+/-`, value fields) are visually similar but imply different behaviors, which can mislead users. | brief suggested fix: Standardize interaction patterns and visual affordances per control type (button vs toggle vs value stepper) with distinct shapes/colors.

7. severity: **low** | screenshot/page: **routing.png** | issue: Section density is high in the right panel (`REC/MON`, `INPUT FX`, `OUTPUT FX`) and spacing between subsection headers/rows is inconsistent. | brief suggested fix: Normalize row heights and header padding, and add slightly larger gaps between subsections.

8. severity: **low** | screenshot/page: **mappings.png / mappings-overlay.png** | issue: Some labels are terse or jargon-heavy (`ACT TRACK`, `ARMED/ACT`, `TAP AGAIN ACT`) and not quickly understandable for scanning. | brief suggested fix: Expand or clarify abbreviations in visible labels (or provide a compact legend) to improve immediate comprehension.

9. severity: **low** | screenshot/page: **all pages** | issue: Global status hierarchy is weak: “LAST ACTION: READY” and function-key hints are present but visually understated relative to dense content. | brief suggested fix: Give persistent status/action areas slightly stronger contrast and clearer grouping so users can orient quickly before scanning detail rows.