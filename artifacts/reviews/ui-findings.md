Findings

1. **Severity: high** — **Screenshot/Page:** `timeline.png`  
   **Issue:** Track headers and control rows are visually overloaded; several labels are truncated (for example `ADD I...`) and become hard to parse at a glance.  
   **Suggested fix:** Reduce per-track header density (fewer always-visible fields), prioritize key states, and expose secondary controls via focus/expand.

2. **Severity: high** — **Screenshot/Page:** `timeline-focused.png`  
   **Issue:** State communication is weak between global vs focused mode; the page still looks nearly as dense as all-tracks view, so focus mode benefit is unclear.  
   **Suggested fix:** In focused mode, increase contrast/size for the active track context and hide or strongly mute non-essential global controls.

3. **Severity: medium** — **Screenshot/Page:** `timeline.png` and `timeline-focused.png`  
   **Issue:** Heavy abbreviation (`TRN`, `MUT`, `SOL`, `ADD O...`, etc.) makes controls ambiguous for first-pass understanding.  
   **Suggested fix:** Expand key labels where possible or add a persistent legend/help strip for abbreviations and state colors.

4. **Severity: medium** — **Screenshot/Page:** `routing.png`  
   **Issue:** The standalone `TAP VALUE` control appears detached from a clear target field, making interaction intent unclear.  
   **Suggested fix:** Attach it to the currently editable value panel (inline or grouped) and show explicit “editing X” state.

5. **Severity: medium** — **Screenshot/Page:** `routing.png`  
   **Issue:** Inconsistent spacing and grouping depth across right-side sections (`REC/MON`, `INPUT FX`, `OUTPUT FX`) weakens visual hierarchy.  
   **Suggested fix:** Standardize section padding, header-to-content spacing, and control row heights across all cards.

6. **Severity: medium** — **Screenshot/Page:** `mappings.png`  
   **Issue:** Bottom command strip is dense and cryptic (`TAP AGAIN ACT`, `W WRITE`, `F8 DIRECT`) with weak separation between shortcuts and actions.  
   **Suggested fix:** Split into labeled groups (navigation/edit/modes), add separators, and increase spacing between tokens.

7. **Severity: low** — **Screenshot/Page:** `midi-io.png`  
   **Issue:** Very large empty list panes read as inactive/placeholder areas; selected/default state tags are small and easy to miss.  
   **Suggested fix:** Add empty/loaded list cues and enlarge/strengthen selected/default badges for faster scan.

8. **Severity: low** — **Screenshot/Page:** `mappings-overlay.png`  
   **Issue:** Overlay and base screen look very similar in contrast and structure, so mode change is subtle.  
   **Suggested fix:** Increase overlay distinction (stronger backdrop dimming, clearer title bar treatment, and stronger active-row emphasis).