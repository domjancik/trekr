Findings

1. **severity: high** — **screenshot/page:** `timeline.png` — **issue:** Dense six-column layout makes track-level status and edit context hard to parse quickly; many repeated micro-labels (`ARM/REC/MUT/SOL`, `+ ADD INPUT FX`, tiny loop markers) compete equally for attention. — **brief suggested fix:** Increase visual hierarchy by emphasizing active/selected track and de-emphasizing repeated secondary controls (lower contrast/smaller frequency, or reveal-on-focus).

2. **severity: high** — **screenshot/page:** `timeline-focused.png` — **issue:** Top bar still shows many global toggles with similar weight while focused mode implies single-track context; state communication between “focused” and “global” controls is ambiguous. — **brief suggested fix:** Visually separate global vs focused-track controls (group boxes or tone shift), and add an explicit “Focused Track Mode” indicator near the page title.

3. **severity: medium** — **screenshot/page:** `mappings.png` — **issue:** Row-level fields are tightly packed; long action labels and scope tags sit very close to borders, creating near-clipping feel and reduced scanability. — **brief suggested fix:** Add horizontal padding in `TARGET`/`SCOPE` cells and slightly increase row height or reduce table density.

4. **severity: medium** — **screenshot/page:** `mappings.png` — **issue:** `ROWS 1 / 30` is top-right and visually detached from the table body, so pagination/state is easy to miss. — **brief suggested fix:** Move row count into the table header line with stronger contrast and proximity to list controls.

5. **severity: medium** — **screenshot/page:** `mappings-overlay.png` — **issue:** Overlay command hints (`F5 CLOSE`, `W WRITE`) are low-emphasis and easy to overlook relative to table content; actionability is unclear. — **brief suggested fix:** Promote shortcut hints into a dedicated, high-contrast overlay header bar with clear primary action styling.

6. **severity: medium** — **screenshot/page:** `midi-io.png` — **issue:** Input/output device cards have very large empty gray bodies, but no obvious affordance for what happens inside each panel (status, meters, routing, details). — **brief suggested fix:** Add placeholder labels or compact metadata blocks (channel count, activity, connection state) to clarify panel purpose.

7. **severity: low** — **screenshot/page:** `midi-io.png` — **issue:** `DEF/SEL` badges are tiny and cramped at card corners, which hurts discoverability and can read like clipped text at a glance. — **brief suggested fix:** Increase badge padding/font size slightly and inset from card edges.

8. **severity: medium** — **screenshot/page:** `routing.png` — **issue:** Abbreviation-heavy controls (`TGL`, `P2`, `KIND`, `VEL`) and repeated `SET` buttons reduce clarity for first-pass understanding. — **brief suggested fix:** Expand key abbreviations or add short inline legends/tooltips; differentiate action buttons by purpose (e.g., `Apply`, `Toggle`, `Edit`).

9. **severity: low** — **screenshot/page:** `routing.png` — **issue:** Spacing between right-side FX sections is tight and visually busy; section boundaries are present but weak under high information density. — **brief suggested fix:** Increase vertical spacing between `INPUT FX` and `OUTPUT FX` blocks and strengthen section headers.

10. **severity: medium** — **screenshot/page:** `all pages` — **issue:** Global status/footer (`LAST ACTION: READY`, function-key hints) has very low hierarchy and blends into chrome, so important state feedback is easy to miss. — **brief suggested fix:** Give status line stronger contrast and reserve accent color for transient or changed state to improve at-a-glance feedback.