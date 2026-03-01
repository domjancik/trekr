Findings

1. `severity: high` `screenshot/page: mappings.png` `issue: The mappings table is too dense and the selected row blends into surrounding rows; TYPE/DEVICE/SOURCE/TARGET/SCOPE/ON all compete equally, so it is hard to scan the active mapping quickly.` `brief suggested fix: Increase row contrast for selection, reduce nonessential column emphasis, and add stronger grouping or spacing between trigger fields and action fields.`

2. `severity: high` `screenshot/page: midi-io.png` `issue: The large empty gray device panels look like disabled or missing content areas rather than selectable lists, and the tiny DEF/SEL badges are easy to miss.` `brief suggested fix: Add visible list rows, empty-state copy, or stronger selection affordances; make default/selected state labels larger and visually tied to the device name.`

3. `severity: medium` `screenshot/page: routing.png` `issue: The right-edge controls mix different meanings (+/-, SELECT, TOGGLE, TAP +/-) with inconsistent sizing and labeling, so it is not obvious which controls change values versus open pickers.` `brief suggested fix: Standardize control patterns by interaction type and use clearer labels like Edit, Choose, Toggle, Nudge.`

4. `severity: medium` `screenshot/page: routing.png` `issue: The top mode strip (ACTIVE 1, THRU OFF, TRACK 1, TAP VALUE) has weak hierarchy and reads like unrelated badges instead of a coherent routing summary.` `brief suggested fix: Reformat it as a structured status header with labels and values, and visually separate mode, target track, and tap state.`

5. `severity: medium` `screenshot/page: timeline.png` `issue: The toolbar is crowded with many equal-weight pills (PLAY OFF, REC OFF, MODE OVERDUB, SONGLOOP ON, TEMPO 120, ...), which makes primary transport state hard to parse.` `brief suggested fix: Promote transport and recording states visually, demote secondary metrics, and add spacing between functional groups.`

6. `severity: medium` `screenshot/page: timeline.png` `issue: The first two columns are highlighted, but the meaning of the dual highlight is unclear; users may not know whether they are selecting a track, a lane, or a loop region.` `brief suggested fix: Differentiate selected track versus selected subpanel with distinct highlight styles and explicit labels.`

7. `severity: low` `screenshot/page: mappings-overlay.png` `issue: The overlay header compresses controls and status text into one line (F5 CLOSE, W WRITE, ROWS 1-19 / 26, SCOPE), which weakens readability and forces users to decode the layout.` `brief suggested fix: Split actions and status into separate aligned regions and give the title more breathing room.`

8. `severity: low` `screenshot/page: mappings.png` `issue: Footer hints such as TAP ROW, TAP FIELD, W WRITE, N NEW, DEL REMOVE, and the bottom-right shortcuts are small and visually secondary even though they explain key actions.` `brief suggested fix: Increase contrast and spacing for command hints, or group them into a clearer command bar with action categories.`

9. `severity: low` `screenshot/page: all screenshots` `issue: Label spacing and padding are inconsistent across tabs, headers, table cells, and badges, which makes the interface feel less structured than it is.` `brief suggested fix: Normalize horizontal padding, header offsets, and badge sizing across pages to tighten alignment and improve scanability.`