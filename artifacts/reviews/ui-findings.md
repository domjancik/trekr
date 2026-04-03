Findings

1. severity: medium; screenshot/page: `timeline.png`; issue: The multi-track view is visually dense with very small labels (`ARM/REC/MUT/SOL`, loop digits, FX rows), making scan-time high and state recognition slow. brief suggested fix: Increase label size/weight one step and reduce per-track chrome (fewer always-visible micro-labels) so track state stands out first.

2. severity: high; screenshot/page: `timeline.png`; issue: Several controls in track footers look clipped/compressed (for example `+ ADD OUTPUT FX ...` and right-edge symbols), which reads like truncation rather than intentional abbreviation. brief suggested fix: Give footer controls more horizontal space or enforce consistent truncation with tooltips/expanded state.

3. severity: medium; screenshot/page: `timeline-focused.png`; issue: The focused layout still keeps many tiny secondary controls in the top bars, so “focused mode” does not strongly communicate reduced complexity. brief suggested fix: Collapse non-essential toggles in focused mode and emphasize the active track/loop with stronger contrast and clearer header grouping.

4. severity: low; screenshot/page: `mappings.png`; issue: Bottom shortcut strip is crowded and low-clarity, with commands running together visually, which makes action discovery hard. brief suggested fix: Split shortcuts into grouped chunks (navigation/edit/learn) with clearer spacing and stronger separators.

5. severity: medium; screenshot/page: `mappings.png`; issue: Scope/state terminology is inconsistent or ambiguous (`GLOBAL`, `ACT TRACK`, `ARMED/ACT`) and may be hard to interpret quickly. brief suggested fix: Normalize scope labels to a single pattern (for example `Active Track`, `Global`, `Armed Track`) and avoid mixed abbreviations.

6. severity: low; screenshot/page: `mappings-overlay.png`; issue: Overlay uses a very large empty area below the table, weakening hierarchy and making the actionable region feel small. brief suggested fix: Either expand list content/metadata into that space or reduce overlay height to keep attention on the table.

7. severity: medium; screenshot/page: `midi-io.png`; issue: `DEF`/`SEL` badges are tiny and visually attached to device rows without clear legend, so default vs selected state is easy to miss. brief suggested fix: Increase badge prominence and add explicit labels (for example `Default`, `Selected`) or a short legend in-panel.

8. severity: medium; screenshot/page: `routing.png`; issue: Control semantics are unclear in multiple places (`SET`, `TGL`, `+/-` clusters) because button meaning depends on context but looks identical everywhere. brief suggested fix: Differentiate action types visually (toggle vs commit vs increment) and add concise inline labels/icons per control type.