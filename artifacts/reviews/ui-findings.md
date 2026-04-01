Findings

1. **severity: high** — **screenshot/page: `timeline.png`** — **issue:** Information density is extremely high (tiny labels, many control chips, per-track micro text), making it hard to parse track state quickly. **brief suggested fix:** Reduce simultaneous visible metadata per track, increase key label sizes, and group secondary controls behind a focused/expanded state.

2. **severity: medium** — **screenshot/page: `timeline.png`** — **issue:** Top-right transport/launch settings (`F6 / SHIFT+F6`, launch quantization, peers) are tightly packed with weak spacing/hierarchy, so related controls are not immediately scannable. **brief suggested fix:** Split into 2–3 grouped rows/cards with clearer spacing and stronger section labels.

3. **severity: medium** — **screenshot/page: `timeline-focused.png`** — **issue:** Focused mode still uses very small secondary labels and abbreviations (`TRN`, `SEM +12`, footer tokens), so “focused” state does not materially improve readability. **brief suggested fix:** In focused mode, enlarge typography and expand abbreviated labels into clearer terms.

4. **severity: medium** — **screenshot/page: `mappings.png`** — **issue:** Scope/status values appear truncated/abbreviated (`ARMED/ACT`, `ACT TRACK`) and can be misread. **brief suggested fix:** Widen scope/status columns or shorten with standardized chips + tooltip/full text on focus.

5. **severity: low** — **screenshot/page: `mappings.png`** — **issue:** Keybinding text formatting is inconsistent (e.g., punctuation combos like `SHIFT+, .`), which hurts quick recognition. **brief suggested fix:** Normalize shortcut notation format and spacing across all rows.

6. **severity: medium** — **screenshot/page: `mappings-overlay.png`** — **issue:** Header metadata (`ROWS 1-19 / 30`, `SCOPE`) is visually detached from table columns, weakening table structure comprehension. **brief suggested fix:** Align header labels directly over their columns and add stronger column separators.

7. **severity: medium** — **screenshot/page: `midi-io.png`** — **issue:** Large empty/light panels dominate space with minimal explanatory state; users may not understand what is selectable vs already assigned. **brief suggested fix:** Add explicit empty/assigned states, row-level affordances, and clearer selected/default indicators with a legend.

8. **severity: medium** — **screenshot/page: `routing.png`** — **issue:** Toggle/state communication is weak (`ON/OFF` text, `TGL`, `SET`, fill bars) and relies on subtle visual differences. **brief suggested fix:** Use explicit state badges (e.g., `Monitoring: ON`), stronger active/inactive contrast, and consistent control patterns for toggles vs actions.

9. **severity: low** — **screenshot/page: `routing.png`** — **issue:** Mixed spacing and alignment between left signal panel and right FX panels makes relationships (input chain vs output chain) less obvious. **brief suggested fix:** Use a stricter grid with matched vertical rhythm and aligned section headers.