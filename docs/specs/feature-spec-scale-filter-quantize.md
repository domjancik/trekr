# Feature Spec: Scale Filter and Scale Quantize

## Summary

Trekr exposes two distinct major-scale MIDI effects:

- `Scale Filter` passes only notes already in the selected scale.
- `Scale Quantize` preserves the existing behavior: it moves every note to the
  nearest note in the selected scale.

Both effects accept `Root` and `Tgt` (`Loc` or `Gbl`). `Loc` uses the effect's
root; `Gbl` uses the timeline `Harmony` root. The initial scale is the existing
major scale: root-relative pitch classes `0, 2, 4, 5, 7, 9, 11`.

## Behavior

### Scale Filter

- An in-scale note passes unchanged: pitch, velocity, start tick, and duration
  are preserved.
- An out-of-scale note is suppressed.
- The same decision is made for playback transformations and live note events.
- A suppressed live note-on and its note-off emit no output. A passed live
  note-on keeps its original-pitch note-off.
- Membership is evaluated by pitch class, so it repeats across all MIDI octaves,
  including pitches `0` and `127`.

### Scale Quantize

- Every note is retained and has its pitch selected by the existing nearest
  major-scale algorithm.
- Existing tie-breaking and octave/boundary behavior are intentionally
  preserved. This effect is a rename at the user-facing surface, not a tuning
  behavior change.
- Live note-offs continue to follow the pitch selected for their matching
  note-on.

## UI and Effect Selection

- The effect picker and routing configuration show the unambiguous names
  `Scale Filter` and `Scale Quantize`.
- Compact timeline labels distinguish them as `SFL` and `SQT` (with two-letter
  fallbacks `SF` and `SQ`).
- Both effects show the same inline controls: `Root` and `Tgt`.
- `Scale` is not used as the user-facing name of either effect.

## Serialization and Migration

- The existing serialized `MidiFx::ScaleQuantize` name remains unchanged and
  continues to deserialize as `Scale Quantize`; saved projects therefore retain
  their current pitch-moving behavior without migration.
- `ScaleFilter` is a new serialized effect variant. It defaults to the same
  `QuantizeTarget::Local` target when an older or partial representation omits
  `target`.
- No existing project is converted to `Scale Filter` automatically.

## Chain Semantics

- Both effects run in normal chain order on input and output chains.
- `Scale Filter -> Scale Quantize` quantizes only notes that passed the filter;
  `Scale Quantize -> Scale Filter` will pass the quantizer's scale-corrected
  output.
- Input monitoring, post-input-FX recording, and output playback use their
  existing chain-routing rules; the effects do not mutate stored source notes.

## Acceptance Criteria

- In-scale filter notes pass unchanged; out-of-scale notes are absent.
- Filter membership works at MIDI pitches `0` and `127` and for non-C roots.
- Quantize retains the legacy nearest-note result, including the tested octave
  boundary case.
- Playback and live paths distinguish suppression from pitch remapping.
- Existing serialized `ScaleQuantize` projects still deserialize and act as
  `Scale Quantize`.
