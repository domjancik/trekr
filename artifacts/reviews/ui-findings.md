Findings

1. severity: **high** | screenshot/page: **timeline.png** | issue: The per-track timeline columns are extremely dense; key controls (`ARM/REC/MUT/SOL`, loop bars, FX strips, note lanes) compete visually, making primary actions hard to parse quickly. | brief suggested fix: Increase vertical grouping and contrast between control rows vs. content lanes; reduce simultaneous on-screen detail in “all tracks” view (progressive disclosure or collapsible rows).

2. severity: **high** | screenshot/page: **timeline-focused.png** | issue: Left and right panes (song vs. loop detail) use nearly identical visual weight and similar line density, so the focused state is not communicated strongly despite the “FOCUSED TRACK + LOOP DETAIL” label. | brief suggested fix: Emphasize the active pane with stronger background/outline contrast and de-emphasize secondary pane content.

3. severity: **medium** | screenshot/page: **mappings.png** | issue: Header controls (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) look like passive labels rather than interactive stateful controls. | brief suggested fix: Add explicit control affordances (button shape depth, active/inactive styling, or small state indicators/icons).

4. severity: **medium** | screenshot/page: **mappings.png** | issue: Table row density is high and row separators are subtle, which hurts scanability of trigger/action pairs. | brief suggested fix: Increase row height slightly or strengthen alternating row backgrounds/dividers.

5. severity: **medium** | screenshot/page: **mappings-overlay.png** | issue: Overlay lacks strong modality cues; it can be mistaken for a normal page rather than an interrupting layer. | brief suggested fix: Add clearer modal treatment (dimmer backdrop, stronger title bar, and a more prominent close/escape hint).

6. severity: **medium** | screenshot/page: **midi-io.png** | issue: Very large empty device panes create uncertainty about whether content failed to load or is intentionally sparse. | brief suggested fix: Add explicit empty/loading/help text inside panes (e.g., “No additional ports detected”).

7. severity: **medium** | screenshot/page: **routing.png** | issue: Repeated shorthand labels (`TGL`, `SET`, `P2`, `MORE`) are ambiguous without contextual hints, increasing cognitive load. | brief suggested fix: Expand labels or add concise inline helper text/tooltips for abbreviated controls.

8. severity: **low** | screenshot/page: **routing.png** | issue: Inconsistent internal padding/alignment across control blocks (left signal column vs. right FX panels) makes layout feel uneven. | brief suggested fix: Normalize horizontal/vertical spacing tokens across panels.

9. severity: **low** | screenshot/page: **mappings-overlay.png** | issue: Top-right status block (`ROWS 1-19 / 30`, `SCOPE`) is visually detached from table headers and easy to miss. | brief suggested fix: Align status metadata with the table header row or move it into a dedicated, clearly labeled header strip.

10. severity: **low** | screenshot/page: **all pages** | issue: State communication relies heavily on subtle color shifts; selected vs. enabled vs. focused states are not always distinguishable at a glance. | brief suggested fix: Add secondary state cues (icons, border styles, text badges) so state is legible beyond color alone.