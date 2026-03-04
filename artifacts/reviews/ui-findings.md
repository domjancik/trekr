Findings

1. severity: `high`; screenshot/page name: `timeline.png`; issue: the top control bar is overloaded and several labels are cryptic (`RECOMPR EXTEND`, `NOTEADD OFF`, `PEERS 0`, `F6 LINK`), making the page hard to parse quickly. Suggested fix: group controls into labeled clusters, expand ambiguous labels, and demote secondary toggles behind a submenu or secondary row.

2. severity: `high`; screenshot/page name: `routing.png`; issue: the selected value cards dominate the page visually while the actual editable affordances are small and inconsistent (`+/-`, `SELECT`, `TOGGLE`, `TAP +/-`), so it is unclear what is interactive versus informational. Suggested fix: standardize control affordances and make action buttons visually distinct from value fields.

3. severity: `medium`; screenshot/page name: `mappings.png`; issue: the table is extremely dense and the repeated horizontal rules plus full-width rows flatten hierarchy, making it difficult to scan columns or identify the active row quickly. Suggested fix: increase row spacing slightly, strengthen column alignment, and use clearer active-row emphasis with subtler non-active row treatment.

4. severity: `medium`; screenshot/page name: `mappings-overlay.png`; issue: the overlay lacks a strong modal header/state treatment, so it reads like another page rather than a focused temporary layer. Suggested fix: add a more prominent overlay title bar and dim or separate the background more clearly to reinforce modal state.

5. severity: `medium`; screenshot/page name: `midi-io.png`; issue: the large empty device cards create excessive dead space while labels and state chips (`DEF`, `SEL`) are tiny, so the important status is visually underweighted. Suggested fix: reduce empty panel height or add structured device metadata, and enlarge/default-highlight state indicators.

6. severity: `medium`; screenshot/page name: `timeline.png`; issue: track headers cram multiple short labels (`ARM`, `REC`, `MUT`, `SOL`) into very tight spaces, which hurts readability and makes states easy to miss. Suggested fix: add more padding in the header rows or collapse secondary states into icons/tooltips with clearer selected styling.

7. severity: `low`; screenshot/page name: `mappings.png`; issue: the footer shortcut strip mixes status-like chips and keyboard hints with similar styling, which weakens hierarchy and makes the legend slower to decode. Suggested fix: separate live status, actions, and key hints into distinct visual groups.

8. severity: `low`; screenshot/page name: `routing.png`; issue: spacing between section title, mode tabs, and the first routing row is uneven, which makes the header area feel crowded relative to the rest of the page. Suggested fix: normalize vertical spacing so section headers, explanatory text, and controls follow a consistent rhythm.

9. severity: `low`; screenshot/page name: `mappings-overlay.png`; issue: the right-side `ROWS 1-19 / 30` and `SCOPE` labels feel detached from the table and can be mistaken for column content. Suggested fix: anchor these summary labels into the overlay header or align them with the table header row.

10. severity: `low`; screenshot/page name: `all pages`; issue: tab colors communicate section identity, but active-state treatment is subtle enough that quick recognition still depends on reading the label. Suggested fix: strengthen active tab contrast or add a more obvious active marker shape/border treatment.