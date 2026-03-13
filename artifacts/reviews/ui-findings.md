Findings

1. **severity: medium** — **screenshot/page: `mappings.png`** — **issue:** Bottom command legend is dense and cryptic (`TAP AGAIN ACT`, `O/E ADJUST`, etc.), making primary actions hard to parse quickly. **brief suggested fix:** Group commands into labeled clusters (Navigation/Edit/Learn), expand ambiguous abbreviations, and increase spacing between groups.

2. **severity: medium** — **screenshot/page: `mappings.png`** — **issue:** Table rows are visually very tight; long target/action labels and scope/status columns compete with little breathing room, reducing scanability. **brief suggested fix:** Increase row height/padding slightly and add clearer column separation (subtle vertical dividers or alternating row tint).

3. **severity: low** — **screenshot/page: `mappings-overlay.png`** — **issue:** Overlay shows `ROWS 1-19 / 30` but provides weak scroll/state affordance, so users may not realize more mappings exist. **brief suggested fix:** Add an explicit scrollbar/progress marker or a “more rows below” indicator.

4. **severity: medium** — **screenshot/page: `mappings-overlay.png`** — **issue:** Large unused lower area in the overlay weakens hierarchy and makes content feel unfinished. **brief suggested fix:** Auto-size the overlay to content height (with max height), or use the extra space for filter/help/context controls.

5. **severity: high** — **screenshot/page: `midi-io.png`** — **issue:** Input/output device cards are mostly blank blocks with minimal labeling; control intent (select, monitor, route, default) is unclear. **brief suggested fix:** Replace empty areas with explicit per-device metadata/actions (status, channel/activity, select/default buttons) and reduce placeholder fill.

6. **severity: medium** — **screenshot/page: `midi-io.png`** — **issue:** `DEF SEL` badges are tiny and ambiguous (default vs selected state merged), weakening state communication. **brief suggested fix:** Separate badges (`DEFAULT`, `SELECTED`) with distinct color/position and add a legend or consistent iconography.

7. **severity: medium** — **screenshot/page: `routing.png`** — **issue:** Right-edge controls (`+`, `SELECT`, `TAP +/-`, `TOGGLE`) are inconsistent by row and low-clarity, making interaction model hard to learn. **brief suggested fix:** Standardize control structure/order per row and add concise row-level helper text or icons.

8. **severity: medium** — **screenshot/page: `timeline.png` and `timeline-focused.png`** — **issue:** Control chips and mode states at the top are very dense with similar visual weight, so critical transport/state info does not stand out. **brief suggested fix:** Elevate primary states (Play/Record/Mode/Quantize) with stronger contrast/priority and demote secondary toggles into a secondary bar or collapsible section.