# Fix Spec: MIDI Runtime Scheduler Refinement

## Summary

Take the async playback runtime prototype from the reMarkable branch and refine it into the general Pi-class MIDI runtime described in `docs/specs/feature-spec-midi-runtime-decoupling.md`.

The prior implementation proves the correct architectural direction: move playback timing off the render loop and publish snapshots back to UI. The fix is to extract that idea without carrying the reMarkable-specific surface, and to replace FIFO `sleep(delay)` delayed sending with an absolute due-time scheduler.

## Parent Spec

This fix spec implements the scheduler-focused slice of:

- `docs/specs/feature-spec-midi-runtime-decoupling.md`

Related prior branch notes:

- `vk/508a-remarkable-thin:docs/planning/remarkable-playback-runtime-decoupling-handoff.md`
- `vk/508a-remarkable-thin:src/app/support/playback_runtime.rs`
- `vk/508a-remarkable-thin:src/midi_io.rs`

## Problem With The Prototype

The prototype adds delayed MIDI output commands with a `delay: Duration` and the output worker performs:

```text
dequeue command
sleep(delay)
send command
```

That is acceptable for instrumentation, but it is not the right final scheduler primitive.

Failure mode:

```text
A: playback/FX event due in 100ms
B: live passthrough note due now
```

If `A` reaches the FIFO worker first, the worker sleeps for `100ms`, and `B` is blocked behind it. That makes live input and urgent note-offs vulnerable to delayed playback/FX events.

## Required Refinement

Replace relative-delay FIFO sleeping with absolute due-time scheduling.

Each MIDI output event should carry:

```text
due_at: Instant
priority: MidiEventPriority
sequence: u64
payload: MidiEventPayload
```

Ordering:

```text
due_at, then priority, then sequence
```

Recommended priorities:

- `Panic`: all-notes-off / panic / forced silence
- `LiveImmediate`: live passthrough and direct monitoring
- `NoteOff`: note-offs at the same due time should not trail note-ons
- `Playback`: normal scheduled playback
- `DelayedFx`: delay/arp/gate-generated future events

## Scheduler Shape

Use a scheduler thread with a min-heap or equivalent priority queue.

Pseudocode:

```rust
loop {
    drain_new_events_into_heap();

    let Some(next) = heap.peek() else {
        wait_for_new_event();
        continue;
    };

    let now = Instant::now();
    if next.due_at <= now {
        send(heap.pop());
        continue;
    }

    wait_until(next.due_at, but wake early when a new event arrives);
}
```

In standard Rust terms, `mpsc::recv_timeout()` is enough for V1:

```rust
let timeout = next_due.saturating_duration_since(Instant::now());
match receiver.recv_timeout(timeout) {
    Ok(event) => heap.push(event),
    Err(RecvTimeoutError::Timeout) => send_due_events(),
    Err(RecvTimeoutError::Disconnected) => break,
}
```

When a new event arrives, recompute the earliest due time. This lets a live event due now preempt an older delayed event due later.

## Implementation Instructions

### 1. Extract The Reusable Runtime Idea

Use the reMarkable implementation as reference, not as a direct merge.

Reusable concepts:

- background runtime thread
- UI-to-runtime `SyncState` command
- runtime-to-UI playback snapshot
- opt-in timing diagnostics
- scheduler-owned playback advancement

Do not preserve:

- `Remarkable*` naming in the general Pi path
- qtfb/AppLoad-specific runtime activation rules
- broad project/display/thin-client changes from the branch
- FIFO worker `sleep(delay)` behavior

### 2. Add A General Module

Preferred location:

```text
src/app/support/midi_runtime.rs
```

The module should own:

- runtime command types
- playback/live scheduler thread
- snapshot type consumed by the UI
- due-time MIDI event queue
- timing diagnostics

### 3. Keep The First Slice Conservative

First implementation should focus on playback scheduling.

Do:

- move playback `advance_playhead()`/note scheduling off the UI frame loop
- publish `transport_ticks` and `playhead_ticks` snapshots
- sync runtime state when UI actions mutate playback-relevant state
- keep rendering best-effort

Do not initially move every MIDI mapping/control path into the runtime.

### 4. Then Move Live Passthrough

After playback timing is stable, route live input through the runtime:

```text
midir callback -> runtime input queue -> routing/Fx snapshot -> due-time scheduler
```

The UI should still receive copied/timestamped events for mapping learn, direct mapping, recording state, and status display.

### 5. Prewarm Outputs

Avoid first-note connection latency by prewarming selected/routed output connections when:

- a default output is selected
- a track route changes
- runtime state sync includes a new active output

Prewarming failure should mark the route unavailable and trigger catalog refresh, not block playback dispatch.

## Acceptance Criteria

- Delayed playback/FX events do not block live passthrough events due now.
- `AllNotesOff` / panic can bypass or clear queued delayed events.
- Note-offs at the same due time are sent before note-ons unless a test documents a different musical requirement.
- Playback timing no longer changes materially between Timeline and static pages.
- UI rendering can stall for more than `16ms` without producing playback catch-up bursts.
- Scheduler diagnostics can report scheduled due time vs actual send time.
- The implementation references the parent decoupling spec and does not import unrelated reMarkable display/thin-client work.

## Validation

Focused tests:

- immediate event preempts future delayed event
- note-off sorts before note-on at same `due_at`
- panic clears/bypasses delayed queue
- sequence preserves deterministic ordering for equal due time and priority

Device checks:

- run Pi build with runtime diagnostics enabled
- compare playback on Timeline vs MIDI I/O page
- test live USB MIDI passthrough while playback has delayed FX events queued

