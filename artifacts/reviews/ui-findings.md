Findings

1. **severity: high** | **screenshot/page:** `timeline-clip-align.png` (Timeline / Clip Align) | **issue:** The Clip Align modal is visually merged into the track canvas; the right side of the modal blends with underlying content, making it unclear where the dialog ends and page begins. | **brief suggested fix:** Add a stronger modal container (opaque background, thicker border, shadow/dim backdrop) and increase separation from underlying track panels.

2. **severity: high** | **screenshot/page:** `timeline-clip-align.png` | **issue:** Top control row text overlaps/clips around `MUT`/`SOL` and nearby track labels while the modal is open, creating unreadable state labels. | **brief suggested fix:** Reserve a fixed header lane above content when dialogs open, or push/clip underlying rows so no text renders into the same vertical band.

3. **severity: medium** | **screenshot/page:** `mappings.png` | **issue:** Header controls (`TAP MODE`, `TAP LEARN`, `TAP DIRECT MAP`) are very similar in shape/tone despite different states; status is hard to scan quickly. | **brief suggested fix:** Use clearer state styling (filled active, muted inactive, explicit status chips like `READ ONLY`, `IDLE`) with stronger contrast differences.

4. **severity: medium** | **screenshot/page:** `mappings.png` and `mappings-overlay.png` | **issue:** Table density is very high; row separators and content spacing are tight, which slows row-by-row parsing and increases misread risk. | **brief suggested fix:** Increase row height/padding slightly and reduce nonessential border weight to improve legibility.

5. **severity: medium** | **screenshot/page:** `mappings-overlay.png` | **issue:** Overlay top metadata (`ROWS 1-19 / 30`, `SCOPE`) is detached from column headers and appears floating, weakening hierarchy. | **brief suggested fix:** Group summary metadata into a dedicated header bar aligned with table columns.

6. **severity: medium** | **screenshot/page:** `midi-io.png` | **issue:** Device cards have large empty interiors with little affordance; it is unclear whether they are selectable list items, meters, or placeholders. | **brief suggested fix:** Add explicit card purpose labels/actions (`Select`, `Default`, `Monitor`) and reduce empty area or populate with key secondary info.

7. **severity: medium** | **screenshot/page:** `routing.png` | **issue:** Multiple small `SET`/`TGL` buttons repeat across dense sections; action intent is ambiguous without immediate context and easy to mis-click. | **brief suggested fix:** Replace generic labels with contextual verbs (`Set Kind`, `Toggle FX`, `Set Channel`) or add inline icons/tooltips.

8. **severity: low** | **screenshot/page:** `routing.png` | **issue:** Spacing/alignment is inconsistent between left `SIGNAL` column and right `REC/MON` + FX blocks, making the page feel visually unbalanced. | **brief suggested fix:** Normalize vertical rhythm and align section tops/baselines across the two columns.

9. **severity: medium** | **screenshot/page:** `timeline.png` | **issue:** Primary state indicators (`PLAY OFF`, `REC OFF`, `SONG LOOP ON`, etc.) share similar visual weight, so critical transport state is not immediately dominant. | **brief suggested fix:** Elevate transport-critical states with stronger color hierarchy and prioritize active-critical indicators visually.

10. **severity: low** | **screenshot/page:** all pages | **issue:** Bottom-right shortcut strip (`F5 MAPPINGS`, `F7 DISCOVER`, `F8 DIRECT`) looks like static footer text and may be missed as actionable navigation. | **brief suggested fix:** Increase button affordance (contrast, padding, active hover/focus style) and optionally add `NAV` label to clarify purpose.