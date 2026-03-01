Findings

1. severity: high; screenshot/page name: `timeline`; issue: the top control strip is too dense and cryptic, with labels like `MODE OVERDUB`, `SONGLOOP ON`, `0 4/16`, `LINK OFF`, and `PEERS 0` competing at the same visual weight, so it is hard to understand transport state quickly; brief suggested fix: group controls by function, expand ambiguous labels, and give primary live states stronger emphasis than secondary settings.

2. severity: high; screenshot/page name: `timeline`; issue: track headers and loop labels at the top of each column are cramped and partially unclear, with tiny text packed against borders and decorative bars, which makes the column purpose hard to read at a glance; brief suggested fix: increase header height, add padding, and simplify the header treatment so track name and lane type remain fully legible.

3. severity: medium; screenshot/page name: `timeline`; issue: the yellow selection outline spans the first track pair in a way that makes the current focus ambiguous, especially between the track lane and loop lane; brief suggested fix: use a single, unambiguous active-state treatment and add a clearer focus model for selected track vs selected sub-panel.

4. severity: medium; screenshot/page name: `timeline`; issue: `F6 LINK` at the far right reads like a separate status or action while `LINK OFF` already exists in the main control row, creating duplicate and potentially conflicting state communication; brief suggested fix: merge link status and shortcut hint into one location or visually separate shortcut help from system state.

5. severity: medium; screenshot/page name: `mappings`; issue: the table is very dense, with long rows, minimal spacing, and uniformly styled cells, so scanning for mapping type, trigger, target, and scope is slower than it should be; brief suggested fix: increase row padding slightly and use stronger column hierarchy or zebra/group styling to improve scanability.

6. severity: medium; screenshot/page name: `mappings`; issue: scope/state values such as `ACT TRACK`, `ARMED/ACT`, `RELATIVE`, and `ABSOLUTE` are abbreviated or mixed in meaning, which makes the right side of the table harder to interpret quickly; brief suggested fix: standardize the terminology and separate scope from mode where those concepts differ.

7. severity: low; screenshot/page name: `mappings`; issue: `ROWS 1 / 26` communicates pagination/count but does not suggest whether more rows are off-screen, paged, or scrollable; brief suggested fix: label it more explicitly or pair it with a visible paging/scroll affordance.

8. severity: low; screenshot/page name: `mappings`; issue: the bottom help text is cramped and low-emphasis compared with the rest of the panel, so useful keyboard guidance is easy to miss; brief suggested fix: add spacing above it and separate shortcut hints into distinct tokens or a dedicated footer bar.

9. severity: medium; screenshot/page name: `mappings-overlay`; issue: the overlay removes the column headers, so users must infer that the three columns mean trigger, action, and scope; brief suggested fix: add short headers or subtle column labels to reduce guesswork.

10. severity: low; screenshot/page name: `mappings-overlay`; issue: helper text `F5 CLOSES   W WRITE MODE` reads like a compressed string rather than two distinct commands, which slows comprehension; brief suggested fix: split shortcuts into clearly separated labels such as `F5 Close` and `W Write Mode`.

11. severity: low; screenshot/page name: `mappings-overlay`; issue: `ROWS 1-21 / 26` suggests pagination or truncation, but there is no visible hint for how to reach the remaining rows; brief suggested fix: add an explicit next-page/scroll hint near the counter.

12. severity: low; screenshot/page name: `midi-io` and `routing`; issue: those pages were not visible from the provided screenshots, so they could not be assessed in this pass; brief suggested fix: provide the missing captures if you want the same QA pass on those screens.