# Recording Clarification Sheet

Use this when reporting or discussing loop-recording behavior.

## 1. Repro Setup

- Branch/commit:
- Page/layout:
- Quantize:
- `RecWrap`:
- Song loop enabled:
- Song loop range:
- Track loop enabled:
- Track loop range:
- Armed tracks / active track:

## 2. Repro Steps

1. Start playhead at:
2. Start recording at:
3. Play or input notes at:
4. Let it pass loop end:
5. Stop recording at:

## 3. Expected Region Behavior

Choose one:

- Clamp at loop end
- Expand to full loop
- Keep growing beyond one loop

Expected region start:

Expected region end or length:

## 4. Expected Note Behavior

Choose one:

- Notes keep their musical positions inside the loop
- Notes re-pack from loop start

Expected first note position:

Expected wrapped note position:

## 5. Stop Condition

Choose one:

- Manual stop only
- Stop at loop end
- Stop when playhead returns to clip start

## 6. Actual Result

- Region did:
- Notes did:
- It stopped when:
- Screenshot/video:

## 7. One-Sentence Intent

Format:

`After wrapping, recording should <region behavior>, notes should <note behavior>, and recording should <stop condition>.`
