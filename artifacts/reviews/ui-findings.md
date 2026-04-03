Findings

1. severity: high | screenshot/page: `timeline.png`, `timeline-focused.png` | issue: The top-right transport/status strip is densely packed (`LINK OFF`, `START/STOP OFF`, `F6 / SHIFT+F6`, `LAUNCH...`, `QUANT`, `PEERS`) and reads as one run-on block, making state hard to parse quickly. | brief suggested fix: Split into grouped chips (Transport, Launch, Quantize, Peers) with stronger spacing and separators; keep shortcuts visually secondary.

2. severity: medium | screenshot/page: `timeline.png` | issue: Track header rows are cramped; labels like `THRU TRACK X`, step numbers, and suffixes (e.g., `+2`) compete in the same narrow line, with near-clipping at card edges. | brief suggested fix: Increase header height or move secondary indicators (step counts/modifiers) to a second line/right-aligned badge area.

3. severity: medium | screenshot/page: `mappings.png` | issue: Column content density is very high with minimal vertical rhythm; rows and action labels blend together, slowing scan speed. | brief suggested fix: Add subtle row striping or larger row padding and increase contrast between headers vs row values.

4. severity: medium | screenshot/page: `mappings-overlay.png` | issue: Overlay includes a large empty lower region while active data is compressed at top, weakening information hierarchy and implying missing content. | brief suggested fix: Auto-size table region to viewport or show clearer pagination/“more rows” affordance; reduce dead space.

5. severity: medium | screenshot/page: `midi-io.png` | issue: Large blank gray device panels look like disabled or uninitialized areas; it is unclear whether these are lists, meters, or placeholders. | brief suggested fix: Add explicit empty/list states and row structure (e.g., “No additional devices”, column headers, or item slots) to communicate intent.

6. severity: low | screenshot/page: `midi-io.png` | issue: Small corner labels like `DEF SEL` are tight to borders and visually easy to miss, reducing state clarity for defaults/selection. | brief suggested fix: Increase label padding and convert to clearer badges/toggles with full words (`Default`, `Selected`).

7. severity: medium | screenshot/page: `routing.png` | issue: Heavy abbreviation use (`REC FX`, `MON FX`, `TGL`, `VAL`, `TRN`) makes controls harder to understand without prior knowledge. | brief suggested fix: Expand key labels or provide inline legend/tooltips; keep abbreviations only where space is truly constrained.

8. severity: low | screenshot/page: `routing.png` | issue: Spacing/alignment is slightly inconsistent between left signal stack and right FX blocks (button gutters and label offsets vary), creating visual jitter. | brief suggested fix: Normalize component grid metrics (shared padding, control heights, label baselines) across panels.