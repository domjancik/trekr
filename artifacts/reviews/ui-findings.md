Findings

1. **severity: medium** | **screenshot/page: `timeline.png`** | **issue:** Top-right transport/meta controls (`LINK OFF`, `START/STOP OFF`, `F6 / SHIFT+F6`, etc.) are tightly packed and visually merge into one strip, making state and shortcut hints hard to parse quickly. | **brief suggested fix:** Split into two grouped rows (state vs shortcuts) with more horizontal padding and clearer separators.

2. **severity: medium** | **screenshot/page: `timeline.png`** | **issue:** Per-track footer labels (e.g., FX labels like `AR RT 1/16 ORD UP ...`) are cramped and appear close to clipping at tile edges, reducing legibility. | **brief suggested fix:** Increase footer height or truncate with ellipsis + tooltip/expanded detail on focus.

3. **severity: medium** | **screenshot/page: `timeline-focused.png`** | **issue:** The focused layout still uses very small dense text in key state rows (`REC`, `MUT`, `SOL`), so focus mode does not significantly improve scan speed. | **brief suggested fix:** In focused mode, enlarge key labels and reduce secondary metadata to strengthen hierarchy.

4. **severity: low** | **screenshot/page: `mappings.png`** | **issue:** Bottom command legend is crowded (`TAP ROW`, `TAP FIELD`, `W WRITE`, etc.) and uses similar visual weight for primary and secondary actions. | **brief suggested fix:** Group by priority (primary actions first), add spacing, and de-emphasize secondary shortcuts.

5. **severity: low** | **screenshot/page: `mappings-overlay.png`** | **issue:** Overlay shows large unused space below rows while also indicating partial pagination (`ROWS 1-19 / 30`), which can feel like missing content or layout inefficiency. | **brief suggested fix:** Either expand visible rows to fill space or reduce overlay height to match content density.

6. **severity: medium** | **screenshot/page: `midi-io.png`** | **issue:** `DEF`/`SEL` badges are tiny and low-context; it is not obvious whether they are status tags, toggles, or actions. | **brief suggested fix:** Add explicit labels (`Default`, `Selected`) and distinct styling for status vs actionable controls.

7. **severity: medium** | **screenshot/page: `routing.png`** | **issue:** Heavy abbreviation usage (`TGL`, `P2`, `VEL`, `SEM`, `MORE`) and repeated compact control patterns make intent unclear without prior knowledge. | **brief suggested fix:** Expand core labels or provide inline helper text on first row of each section; keep abbreviations for expert mode only.

8. **severity: low** | **screenshot/page: `routing.png`** | **issue:** Section spacing and border density are very uniform, so important state blocks (`REC FX`, `MON FX`, `INPUT FX`, `OUTPUT FX`) compete visually rather than forming a clear hierarchy. | **brief suggested fix:** Increase contrast and spacing between major groups, and use stronger heading treatment for active/critical sections.