Findings

1. **severity: high | screenshot/page: `timeline.png` | issue:** The page is information-dense to the point that primary state is hard to parse quickly (global transport/options, per-track controls, loop lanes, and FX rows all compete visually at similar contrast/weight). **brief suggested fix:** Increase hierarchy separation: stronger section headers, reduced contrast for secondary metadata, and more vertical spacing between global controls, track bodies, and FX rows.

2. **severity: medium | screenshot/page: `timeline.png` | issue:** Track header/control text is cramped (`ARM/REC/MUT/SOL`, `ADD INPUT FX`, loop tokens), making labels feel clipped/tightly packed even when technically visible. **brief suggested fix:** Add horizontal padding and minimum field widths; abbreviate consistently only where space is constrained.

3. **severity: medium | screenshot/page: `timeline-focused.png` | issue:** In focused mode, large empty regions and dense right-pane note data create an imbalanced layout; key “focused” state is not strongly communicated beyond subtle header text. **brief suggested fix:** Add a clearer focused-state banner/badge and rebalance panes (or add contextual helper text) so mode change is immediately obvious.

4. **severity: high | screenshot/page: `mappings.png` | issue:** The bottom shortcut/help strip is overloaded and hard to scan; actions, navigation, and edit commands are presented in one dense line, increasing misuse risk. **brief suggested fix:** Group shortcuts by category (Navigation/Edit/Learn), add separators or two-line layout, and prioritize only context-relevant commands.

5. **severity: medium | screenshot/page: `mappings-overlay.png` | issue:** Key-hint conflict: overlay header shows `F5 CLOSE`, while persistent footer still shows `F5 MAPPINGS`, which is ambiguous in this state. **brief suggested fix:** Suppress or remap conflicting global footer hints while overlay is active.

6. **severity: medium | screenshot/page: `midi-io.png` | issue:** Large gray list areas with minimal row affordance make list state unclear (empty space can read as disabled/missing data rather than available slots). **brief suggested fix:** Add explicit list-row structure, empty/loading messaging, or subtle row guides to communicate interaction model.

7. **severity: low | screenshot/page: `midi-io.png` | issue:** Tiny status chips (`DEF`, `SEL`) are visually cramped in card corners and easy to miss. **brief suggested fix:** Increase chip padding/size and anchor them with consistent spacing from borders.

8. **severity: medium | screenshot/page: `routing.png` | issue:** The top-right `TAP VALUE` control feels weakly associated with any specific editable field, which can mislead users about what will be modified. **brief suggested fix:** Context-bind it to the currently focused field (inline or adjacent), or add a clear active-target indicator near the button.