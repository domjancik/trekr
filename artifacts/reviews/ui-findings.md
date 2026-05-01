Findings

1. **severity: high** — **screenshot/page: timeline.png** — **issue:** The six-track view is extremely dense; key controls (`ARM/REC/MUT/SOL`, FX rows, per-track badges) collapse into tiny text and become hard to scan quickly. **brief suggested fix:** Increase minimum row/header heights and reduce simultaneous detail per track (or add a compact/expanded track mode) so critical controls are legible at a glance.

2. **severity: medium** — **screenshot/page: timeline.png** — **issue:** In the top status/control strip, timing/meta fields (`120`, `*`, `Q 1/16`, `P 0`) are visually cryptic and weakly labeled, so state meaning is unclear without prior knowledge. **brief suggested fix:** Add short inline labels/tooltips (e.g., `Tempo`, `Quant`, `Pattern`) or use grouped labeled chips.

3. **severity: medium** — **screenshot/page: mappings.png** — **issue:** Bottom legend/help text is crowded and low-contrast in places, making keyboard guidance hard to parse quickly. **brief suggested fix:** Split help into grouped segments with clearer spacing and stronger contrast for key/action pairs.

4. **severity: medium** — **screenshot/page: mappings-overlay.png** — **issue:** Overlay context is ambiguous: title says `MAPPINGS OVERLAY`, but no explicit indicator of what changed vs base page (beyond `F5 CLOSE`) and reduced chrome can feel like a separate page. **brief suggested fix:** Add a stronger modal state cue (`Overlay`, dimmed backdrop label, or short subtitle like `Quick read-only list`).

5. **severity: low** — **screenshot/page: midi-io.png** — **issue:** Input/output card interiors are mostly empty gray blocks with little hierarchy, so actionable information density appears low and cards look unfinished. **brief suggested fix:** Add placeholder structure (port status, activity meter, or route summary lines) to clarify what each large panel communicates.

6. **severity: medium** — **screenshot/page: routing.png** — **issue:** Multiple adjacent pastel-tinted panels (`Signal`, `Rec/Mon`, `Input FX`, `Output FX`) compete equally for attention; primary workflow order is not obvious. **brief suggested fix:** Strengthen hierarchy with clearer section priority (size/contrast), and add a simple left-to-right flow cue (`Input -> Monitor/Record -> Output`).

7. **severity: low** — **screenshot/page: timeline-focused.png** — **issue:** Track-focus state (`TRACK T1`) is present but visually subtle relative to surrounding controls, so mode change from multi-track to focused view may be missed. **brief suggested fix:** Increase focus-state prominence (stronger badge contrast and/or page subtitle reflecting `Focused Track: T1`).

