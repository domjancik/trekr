Findings

1. **severity: medium** — **screenshot/page: `routing.png`** — **issue:** Multiple `INPUT FX`/`OUTPUT FX` row labels appear cramped and visually noisy (`INPUTSLOT1`, `INPUTSEMI +12`, `OUTVEL+20%`), making row purpose hard to scan quickly. **brief suggested fix:** Increase inner padding and split each row into fixed columns (name, value, action) with clearer separation.

2. **severity: medium** — **screenshot/page: `timeline.png`** — **issue:** Track columns are extremely dense; labels like track headers, mode chips, and small status text compete in the same horizontal band, weakening hierarchy. **brief suggested fix:** Add stronger vertical grouping (header band vs content), reduce simultaneous inline metadata, and prioritize one primary label per row/section.

3. **severity: medium** — **screenshot/page: `timeline-focused.png`** — **issue:** Top control/status strip remains crowded even in focused mode, so “focused” state does not communicate enough simplification. **brief suggested fix:** In focused mode, hide secondary controls and enlarge key active-state labels (track, loop, mode) to reinforce context.

4. **severity: low** — **screenshot/page: `midi-io.png`** — **issue:** `DEF SEL` badges sit tight against row edges and read as cramped, which can look clipped at a glance. **brief suggested fix:** Add right padding and a clearer badge container with consistent spacing from row borders.

5. **severity: low** — **screenshot/page: `mappings.png`** — **issue:** Bottom shortcut legend is visually compressed and uses many abbreviated tokens, reducing immediate clarity. **brief suggested fix:** Increase spacing between legend items and group by action type (navigation/edit/learn) with subtle separators.

6. **severity: low** — **screenshot/page: `mappings-overlay.png`** — **issue:** Overlay has a large unused lower area while key controls sit in a tight top cluster, creating uneven visual balance and weak information hierarchy. **brief suggested fix:** Re-balance vertical layout (compact panel height to content or add structured footer/help block).

7. **severity: low** — **screenshot/page: `routing.png`** — **issue:** State communication between `REC FX`, `MON FX`, and `PASSTHROUGH` is ambiguous because active/inactive styles are similar in weight. **brief suggested fix:** Use a stronger on/off contrast pattern (distinct fill + icon/text state) and consistent toggle placement across sections.