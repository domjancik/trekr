Findings

1. severity: **high** | screenshot/page: **timeline.png** | issue: Top control bands are too dense; small labels (`MODE OVERDUB`, `LAUNCHQ OFF`, `QUANT 1/16`, `PEERS`) and shortcut hints compete at the same visual level, making state hard to scan quickly. | brief suggested fix: Group controls into 2–3 clearly separated clusters (transport, launch, quantization), increase vertical spacing, and reserve a stronger style for active states only.

2. severity: **high** | screenshot/page: **timeline-focused.png** | issue: Right-edge track header metadata appears cramped/clipped (`+12`, symbols near far right), which looks truncated and is hard to interpret. | brief suggested fix: Add right padding and minimum width for the header meta region; collapse less-critical tokens into a compact submenu or tooltip.

3. severity: **medium** | screenshot/page: **mappings.png** | issue: Mapping rows are visually repetitive with weak column hierarchy; `DEVICE`, `SOURCE`, `TARGET`, `SCOPE`, `ON` blend together and slow parsing. | brief suggested fix: Increase contrast between column headers and row values, add stronger vertical separators, and lighten only the active/editable column per row.

4. severity: **medium** | screenshot/page: **mappings-overlay.png** | issue: Overlay header/status text (`ROWS 1-19 / 30`, `SCOPE`) feels detached from table columns and alignment is inconsistent with body rows. | brief suggested fix: Align status metadata to the table grid and place it in a dedicated header row with consistent left/right anchors.

5. severity: **medium** | screenshot/page: **midi-io.png** | issue: Large empty gray blocks in device cards look like missing content or disabled controls; state communication is unclear. | brief suggested fix: Add explicit placeholders (`No activity`, `No channels shown`) or lightweight meters/labels so empty areas read as intentional.

6. severity: **medium** | screenshot/page: **routing.png** | issue: Repeated tiny action labels (`SET`, `TGL`, `+`, `-`) are ambiguous and require memorization; interaction intent is unclear. | brief suggested fix: Add explicit micro-labels/icons (`Toggle`, `Apply`, `Increment`) or a short legend near the panel title.

7. severity: **low** | screenshot/page: **routing.png** | issue: Multiple saturated panel colors (signal/rec-mon/input-fx/output-fx) carry similar visual weight, weakening hierarchy of primary vs secondary sections. | brief suggested fix: Reduce saturation for secondary panels and reserve strongest contrast for currently focused/editing block.

8. severity: **low** | screenshot/page: **mappings.png** and **timeline.png** | issue: Bottom function-key strip and status line text are very small with low contrast, reducing discoverability of key actions. | brief suggested fix: Increase font size/contrast one step and add slightly more vertical padding to the bottom command bar.