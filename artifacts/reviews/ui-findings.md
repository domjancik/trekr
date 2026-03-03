Findings

1. severity: high; screenshot/page name: `routing`; issue: The `TAP +/-` control is visually merged into the selected input device row and reads like part of the field value rather than a separate action. brief suggested fix: Separate it with stronger spacing/border treatment or move it into a dedicated action column/button area.

2. severity: high; screenshot/page name: `routing`; issue: The selected state is hard to parse because entire rows are heavily color-filled while labels/actions (`SELECT`, `TOGGLE`, `+`) remain low-contrast and visually secondary. brief suggested fix: Reduce fill dominance and add a clearer focused/active treatment around the interactive subcontrol being edited.

3. severity: medium; screenshot/page name: `mappings`; issue: Top toolbar labels (`TAP MODE: READ ONLY`, `TAP LEARN: IDLE`, `TAP DIRECT MAP`) are cramped together and read as one strip of similar-weight boxes, which weakens hierarchy and slows scanning. brief suggested fix: Increase spacing between segments and differentiate status pills from action buttons.

4. severity: medium; screenshot/page name: `mappings`; issue: Bottom shortcut hints are dense and inconsistent in emphasis, making the command bar hard to decode quickly. brief suggested fix: Group shortcuts by function and add more separation/padding between tokens.

5. severity: medium; screenshot/page name: `mappings-overlay`; issue: The overlay lacks a strong visual cue that it is modal beyond the darkened background, so its relationship to the underlying page is not immediately obvious. brief suggested fix: Add a clearer overlay frame/header treatment and a more prominent modal title/close affordance.

6. severity: medium; screenshot/page name: `midi-io`; issue: The two output panels have inconsistent visual weight and spacing, and the lower output card looks secondary without clear reason. brief suggested fix: Standardize panel sizing/alignment or label the lower panel’s role more explicitly if it is intentionally different.

7. severity: medium; screenshot/page name: `midi-io`; issue: `DEF` and `SEL` badges are tucked into the top-right corners of cards and are easy to miss, so default/selected state is not communicated strongly. brief suggested fix: Promote these states with stronger contrast and more breathing room, or move them nearer the card title.

8. severity: low; screenshot/page name: `timeline`; issue: The control strip above the track grid is visually crowded, with many similar chips competing equally for attention. brief suggested fix: Group controls by purpose and give primary transport/mode states stronger hierarchy than secondary status chips.

9. severity: low; screenshot/page name: `timeline`; issue: The selected loop/track state relies on subtle yellow borders that are easy to lose against the already busy grid. brief suggested fix: Strengthen selected-state contrast with a thicker outline, tinted background, or clearer header highlight.

10. severity: low; screenshot/page name: `mappings-overlay`; issue: The right-aligned `ROWS 1-19 / 30` and `SCOPE` metadata feels detached from the table and slightly awkwardly spaced relative to the header. brief suggested fix: Align metadata to a clearer header grid or place it on a dedicated subheader row.