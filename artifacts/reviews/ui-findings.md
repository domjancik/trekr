Findings

1. `severity: medium` | `timeline.png` | The top control area is overloaded with many similarly styled pills (`PLAY OFF`, `RECORD OFF`, `MODE OVERDUB`, quant/peers/link), so primary state is hard to scan quickly. | Increase hierarchy by grouping into labeled sections (transport, loop, sync), and use stronger visual contrast for active states only.

2. `severity: medium` | `routing.png` | Row-end actions (`+`, `SELECT`, `TAP +/-`, `TOGGLE`) are small and visually inconsistent, making interaction intent unclear. | Normalize action button sizing/placement and add clearer affordances (consistent button style, explicit labels like `Edit`, `Toggle`, `Tap +/-`).

3. `severity: low` | `midi-io.png` | The `DEF SEL` badges are tiny and crowded into card corners, which reduces readability and state clarity. | Enlarge badges and place them in a consistent metadata row under each device name.

4. `severity: low` | `mappings.png` | Dense row layout and minimal column separation make key data (`Type`, `Device`, `Source`, `Target`, `Scope`, `On`) harder to parse at a glance. | Add slightly more vertical row spacing and stronger column delineation (subtle separators or alternating row backgrounds).

5. `severity: low` | `mappings-overlay.png` | Overlay header actions (`F5 CLOSE`, `W WRITE`) are easy to miss because they look like body text rather than actionable controls. | Style overlay actions as explicit buttons/chips with clearer active/inactive states.

6. `severity: low` | `timeline-focused.png` | Focus mode improves readability, but the `REC`/`MUT`/`SOL` labels remain visually detached from the panes and can be misread as static headers. | Anchor these labels to pane headers with clearer grouping and stronger alignment cues.