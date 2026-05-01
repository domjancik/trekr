Findings

1. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Bottom hint bar is visually dense and hard to parse quickly (`TAP ROW`, `TAP FIELD`, `W WRITE`, `F8 DIRECT`, etc.), with weak grouping between actions vs key hints. | **brief suggested fix:** Split into 2 grouped zones (actions vs keybindings), add clearer separators, and reduce token count shown at once.

2. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Rightmost `ON` status cells are very narrow and low-contrast, so enabled state is easy to miss while scanning rows. | **brief suggested fix:** Widen the status column and increase contrast or use a stronger on/off badge treatment.

3. **severity: low** | **screenshot/page:** `mappings-overlay.png` | **issue:** Header context is minimal (`ROWS 1-19 / 30`, `SCOPE`) and doesn’t clearly indicate sort/filter mode, making overlay state ambiguous vs full page state. | **brief suggested fix:** Add one explicit mode line (e.g., “Overlay: quick browse, read-only/write-capable”) near the title.

4. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Device cards look like large empty blocks; primary actions/states (`DEF`, `SEL`) are tiny and detached from card hierarchy. | **brief suggested fix:** Promote selected/default state into prominent card headers and reduce empty fill area or add summary metadata in the body.

5. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Many adjacent control groups (`INPUT FX`, `OUTPUT FX`, `REC FX`, `MON FX`) use similar visual weight, so users must read every label to orient. | **brief suggested fix:** Strengthen section hierarchy with clearer header bands, spacing between logical clusters, and distinct accent rules per major block.

6. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Mixed button labeling (`SET`, `TGL`, value chips) is not self-explanatory at a glance, especially where value and action are separated. | **brief suggested fix:** Standardize control grammar (e.g., always `value + verb`) and add short inline affordance labels/tooltips in the footer legend.

7. **severity: high** | **screenshot/page:** `timeline.png` | **issue:** Severe information density across top controls and six track columns causes weak scanability; track state, loop data, and FX labels compete equally. | **brief suggested fix:** Introduce stronger vertical rhythm and progressive disclosure (collapse secondary per-track metadata until focused/selected).

8. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Small text in track headers and FX footers is close to clipping and hard to read (`ORD UP..`, tiny symbols at right), increasing interpretation cost. | **brief suggested fix:** Increase min text size or reserve larger footer/header height; truncate with deliberate ellipsis rules and hover/full-view affordance.

9. **severity: medium** | **screenshot/page:** `timeline-focused.png` | **issue:** Focused mode improves space, but left pane (song lanes) and right pane (loop detail) have similar emphasis, so the “focused track” intent is still diluted. | **brief suggested fix:** Visually prioritize the active editing pane (stronger highlight/background contrast) and de-emphasize the non-active pane.

10. **severity: low** | **screenshot/page:** `timeline-focused.png` | **issue:** Top-right timing controls (`120`, `TAP`, `*`) are compact and somewhat cryptic without immediate semantic cues. | **brief suggested fix:** Add tiny labels/icons for tempo/tap/sync state and increase spacing between interactive units.