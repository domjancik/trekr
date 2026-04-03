Findings
1. severity: medium | screenshot/page: `mappings.png` | issue: Bottom shortcut/help strip is very dense (`SHIFT+LEFT/RIGHT FIELD`, `0/E ADJUST`, `ENTER LEARN/TOGGLE`) and hard to parse quickly at this size. | brief suggested fix: Group shortcuts by function with separators and slightly increase spacing/contrast for each group.

2. severity: low | screenshot/page: `mappings.png` | issue: `SOURCE` values use mixed formats (`--`, key chords, `ANY MIDI`, `CC20`) with no visual distinction, which makes scanability weaker. | brief suggested fix: Add type-specific styling/icons (keyboard vs MIDI) and normalize placeholder formatting.

3. severity: medium | screenshot/page: `mappings-overlay.png` | issue: Overlay header communicates `ROWS 1-19 / 30`, but there is a large empty area below the table that looks unfinished or like missing rows. | brief suggested fix: Either paginate/fill visible rows consistently or reduce overlay height to match visible content.

4. severity: medium | screenshot/page: `midi-io.png` | issue: Badges `DEF` and `SEL` are cryptic and visually tiny; state communication is weak for default vs selected routing. | brief suggested fix: Use explicit labels/tooltips (`Default`, `Selected`) or a clearer legend near section headers.

5. severity: low | screenshot/page: `midi-io.png` | issue: Input/output cards have very large empty body areas, so hierarchy emphasizes blank space more than actionable metadata. | brief suggested fix: Reduce card body height or surface key details/actions in the empty region.

6. severity: medium | screenshot/page: `routing.png` | issue: Control labels rely heavily on abbreviations (`REC FX`, `MON FX`, `TGL`, `VAL`, `SET`) that are ambiguous without prior knowledge. | brief suggested fix: Expand labels where space allows or add an always-visible legend/help hint for abbreviations.

7. severity: medium | screenshot/page: `timeline.png` | issue: The page is visually overloaded with many equal-weight panels and very small text, making first-read comprehension slow. | brief suggested fix: Increase hierarchy contrast (stronger section headers, reduced simultaneous detail, clearer active track emphasis).

8. severity: low | screenshot/page: `timeline.png`, `timeline-focused.png` | issue: State change from multi-track to focused mode is subtle; users may miss what changed beyond layout width. | brief suggested fix: Add a stronger mode indicator (e.g., prominent `Focused Track` badge and dimmed/hidden non-focused context labels).