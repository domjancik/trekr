Findings

1. severity: medium; screenshot/page: `timeline.png`; issue: Column density is very high (6 tracks + many small badges/buttons), making track state hard to parse at a glance. suggested fix: Increase vertical rhythm and reduce simultaneous chrome (e.g., collapse less-used per-track controls or add a simplified/default track header mode).

2. severity: medium; screenshot/page: `timeline-focused.png`; issue: Left pane (input/through) and right pane (loop detail) have similar visual weight, so primary focus is ambiguous despite “focused track” mode. suggested fix: Strengthen focus hierarchy by dimming secondary pane and emphasizing the active editing pane with stronger contrast or header treatment.

3. severity: low; screenshot/page: `mappings.png`; issue: Bottom hint bar actions mix active-looking chips and passive text with inconsistent spacing, which weakens scanability of key commands. suggested fix: Normalize chip padding/spacing and separate “active shortcuts” vs “instructional text” into distinct visual groups.

4. severity: medium; screenshot/page: `mappings-overlay.png`; issue: Overlay header/action area is sparse and low-contrast relative to the table, so mode/context (“overlay” vs base page) is easy to miss. suggested fix: Add a clearer modal/overlay header band with stronger contrast and a more explicit mode label.

5. severity: low; screenshot/page: `midi-io.png`; issue: Device cards have large empty gray bodies with little affordance, making it unclear whether they are selectable lists, meters, or placeholders. suggested fix: Add lightweight empty-state labels or structural cues (rows/placeholders/icons) to clarify expected content and interaction.

6. severity: medium; screenshot/page: `routing.png`; issue: Many repeated `SET`/`TGL` controls create ambiguity about which value each button affects, especially in dense FX blocks. suggested fix: Tighten label-control binding (closer proximity or inline pairing) and differentiate control types visually (e.g., color/shape per action type).

7. severity: low; screenshot/page: `routing.png`; issue: Minor spacing inconsistency between section headers and first control rows (varies across SIGNAL, REC/MON, INPUT FX, OUTPUT FX), which makes layout feel uneven. suggested fix: Standardize top padding/margins for all panel sections using one spacing scale.

8. severity: low; screenshot/page: `mappings.png` and `mappings-overlay.png`; issue: Scope/state chips (`GLOBAL`, `ACT TRACK`, `ON`) are visually subtle and similar to surrounding cells, reducing state clarity. suggested fix: Increase state contrast and use consistent semantic color coding for scope and enabled/disabled state.