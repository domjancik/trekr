Findings
1. severity: high; screenshot/page name: `timeline.png`; issue: 6 track columns are extremely dense, with tiny labels (`ARM/REC/MUT/SOL`, lane headers, FX rows) and minimal whitespace, making rapid scanning difficult; brief suggested fix: reduce visible tracks at once (or add compact/expanded modes), increase row/header height by a few pixels, and prioritize key labels with stronger size/contrast.

2. severity: high; screenshot/page name: `timeline-focused.png`; issue: left panel and main note grid compete visually with similar contrast and weight, so focus context is not immediately clear despite “TRACK T1”; brief suggested fix: add stronger focus separation (dim non-primary panel, brighten focused grid/header, and increase visual prominence of focus chip).

3. severity: medium; screenshot/page name: `mappings.png`; issue: bottom legend/help row is overloaded and cryptic (`TAP ROW`, `W WRITE`, `F8 DIRECT`, etc.), which hurts quick understanding of available actions; brief suggested fix: group actions by function with separators/icons and replace abbreviations with short plain-language labels where space allows.

4. severity: medium; screenshot/page name: `mappings-overlay.png`; issue: overlay table has weak column hierarchy (`TRIGGER`, `ACTION`, `SCOPE` look similar), so eye tracking across long rows is slower than necessary; brief suggested fix: strengthen header contrast/weight and add subtle alternating row backgrounds or clearer vertical separators.

5. severity: medium; screenshot/page name: `midi-io.png`; issue: selected state signaling is inconsistent (`DEF SEL` appears only on some cards; card tint + border + badge are all used), making it unclear what is selected vs default; brief suggested fix: standardize state model with one primary selected indicator and one secondary default marker applied consistently.

6. severity: medium; screenshot/page name: `routing.png`; issue: many controls labeled `SET`/`TGL` lack immediate affordance about what value will change, especially in dense FX blocks; brief suggested fix: rename to contextual actions (e.g., `SET KIND`, `TOGGLE MON FX`) or place action labels adjacent to field names.

7. severity: low; screenshot/page name: `routing.png`; issue: minor spacing inconsistency between stacked control rows in `INPUT FX` vs `OUTPUT FX` sections creates visual jitter; brief suggested fix: normalize vertical rhythm (uniform row heights and inter-row gaps across both sections).

8. severity: low; screenshot/page name: `mappings.png` and `mappings-overlay.png`; issue: row count presentation differs (`ROWS 1 / 30` vs `ROWS 1-19 / 30`), which can feel inconsistent between views; brief suggested fix: unify pagination format or clearly label one as “visible range” and the other as “cursor row.”