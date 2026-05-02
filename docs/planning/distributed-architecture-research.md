# Distributed Architecture Research

## Purpose

This document describes the changes needed to evolve trekr from a single-process app into an engine-authoritative distributed system where:

- a headless or semi-headless engine can run on a device such as a Raspberry Pi
- one or more front-ends can render shared state
- local and remote inputs can control the same session
- clients can survive lossy links such as Wi-Fi and constrained links such as Bluetooth

Primary context:

- `docs/dev/architecture.md`
- `docs/planning/handoff-summary.md`
- `docs/planning/implementation-plan.md`

## Current Baseline

Today the architecture is still mostly single-process and UI-led:

- `App` owns project state, transport state, MIDI I/O runtimes, Link runtime, undo history, and rendering state in one composition root.
- the SDL event loop polls pointer and keyboard events, polls MIDI input, advances transport/playhead time, and renders in the same main loop
- persisted state is a local snapshot, not a replicated session model
- `AppAction` already gives us the right source-agnostic control boundary and includes `ActionSource::Remote`, which is a strong starting point
- the current code already supports more than one presentation shell in principle (`trekr` SDL app, `trekr-tui`, capture mode), but they are not backed by a shared network session yet

This means the repo already has the right *command vocabulary*, but not yet the right *process boundaries*.

## Key Design Decision

Use an **engine-authoritative session**.

That means:

- exactly one engine process is the authority for transport, recording, loop state, routing, MIDI timing, Link participation, and undoable document changes
- every UI, touch surface, keyboard, or remote controller sends canonical commands to that engine
- every display renders from replicated engine state, not from locally-simulated truth
- local input devices attached directly to the engine are treated as just another command source

This is the safest model for:

- tight MIDI timing
- multiple concurrent clients
- reconnect after packet loss
- mixed local + remote control
- keeping a single undo/history truth

Do **not** make each client a peer-authoritative editor for V1 of distributed support. That would force conflict-free replicated editing, distributed undo semantics, and cross-device timing arbitration before the product is ready.

## Preserve Single-Device Performance As A First-Class Requirement

Distributed support must not make the default single-device build feel heavier on small hardware.

Recommended rule:

- the local single-device path should use the same engine/client protocol model conceptually, but with an optimized in-process transport that avoids network stacks, serialization churn, and duplicated copies where possible

Practical constraints:

- no mandatory background networking work when no remote client is attached
- no per-frame full-state serialization for the local UI
- no extra locks or heap churn in the MIDI/audio timing path just because remote mode exists
- no requirement that a single-device session spin up multiple heavyweight processes

Recommended implementation shape:

- keep one authoritative engine runtime
- allow an **in-process loopback client** for the normal local SDL app
- use direct function calls or bounded channels with compact event structs for local command delivery
- serialize only at the actual network boundary, not between tightly-coupled local modules

Best practice:

- treat distributed support as an outer transport layer around the engine boundary, not as a reason to make every local operation pay network-style costs

This preserves the current low-overhead direction from `docs/dev/architecture.md`, especially for Raspberry Pi and other constrained targets.

## Target Topology

Recommended topology:

1. **Engine host**
   - owns transport clock and document state
   - owns MIDI input/output and Link session participation
   - produces authoritative session snapshots and deltas
2. **Display/controller clients**
   - subscribe to session state
   - render timeline/pages locally
   - send canonical input commands
3. **Optional local shell on the engine host**
   - same protocol as remote clients, even if in-process
   - useful for single-device fallback and testing

This supports all requested variations:

- Pi engine + separate touch display
- touch display plus keyboard connected to the Pi
- multiple displays observing the same session
- multiple operators sending input into one shared state

## Per-Device UI State Must Be Explicit

Each connected device should hold its own UI state unless a specific feature is intentionally shared.

Recommended model:

- one shared engine session
- one per-device or per-client UI state object
- optional explicitly shared editing scopes layered on top

Per-device UI state should usually include:

- current page
- overlay visibility
- focused row/field/widget
- viewport size and density mode
- theme and scaling preferences
- local browse selection
- playhead follow preference
- local panel expansion/collapse state

Why this matters:

- a wall display may stay on Timeline while a tablet stays on Routing
- one user may keep a discoverability overlay open while another wants a clean performance screen
- accessibility, scale, and touch density can differ per device
- a low-resolution device may need a different presentation state from a larger external monitor

Important rule:

- per-device UI state should not live inside the shared project document or the shared transport model

Only promote UI-adjacent state into the shared session when collaboration truly depends on it, such as:

- a lease-backed shared edit selection
- a shared presentation mode intentionally mirrored to multiple screens
- an operator-controlled “follow active track” mode for a specific linked display group

## Headless Engine And Thin-Client Display Mode

The architecture should explicitly support a deployment where:

- the engine runs on a device with no display
- another device renders the UI as a thin client
- inputs may come from either side

Typical example:

- a keyboard or MIDI controller is connected to the engine host
- a separate tablet or monitor device displays the UI
- the displayed UI reflects actions originating from the engine host as well as actions originating from the thin client itself

Recommended rule:

- the thin client owns its own per-device UI state
- the engine owns shared session state and applies canonical commands
- engine-local inputs that depend on UI context must resolve against an explicitly assigned UI state, not against some implicit nonexistent “main screen”

This means the system needs a way to associate engine-local inputs with a specific client UI context when those inputs are context-sensitive.

## Required Architectural Changes

### 1. Split app state from view state

Create a strict separation between:

- **session/domain state**: project, transport, routing, mappings, Link-relevant transport state, record state, undoable edits
- **engine runtime state**: clocks, active notes, live FX runtime, connected devices, per-client sessions
- **client view state**: current page, overlays, pointer hover, local viewport size, local focus ring, local theme, local UI scale
- **per-device UI session state**: persisted client-side navigation and presentation state for that specific connected device

Important rule:

- page selection, overlay visibility, and viewport-specific UI state should become **per-client state**, not global session state, unless deliberately shared
- model this as **many simultaneous UI states**, one for each connected client, rather than one “current UI state” for the whole session

If this is not separated, one user changing pages would unexpectedly move every connected screen.

### 2. Extract an engine service boundary

Refactor the current `App` into three layers:

- `trekr_core`
  - pure domain model and reducers
  - action validation
  - snapshot/delta serialization model
- `trekr_engine`
  - authoritative runtime
  - MIDI I/O, Link, transport tick, recording pipeline, session replication
- `trekr_client`
  - SDL UI and future remote display shells
  - local input translation into canonical actions
  - rendering from replicated state

Near-term repo shape can stay single crate initially, but code ownership should follow those boundaries.

### 3. Make the action layer network-ready

Keep `AppAction` as the canonical control vocabulary, but wrap it in a richer command envelope:

```text
ClientCommand {
  session_id,
  client_id,
  client_revision,
  command_id,
  sent_at,
  source,
  target_client_scope?,
  action,
}
```

Additional requirements:

- every command needs a unique id for dedupe during reconnect/retry
- commands must carry client identity
- some commands will be **session-global** and some **client-local**
- commands should be validated against the sender's capabilities and current lease/focus rules

### 4. Introduce session snapshots plus ordered deltas

Do not stream raw mutable structs ad hoc.

Use:

- **full snapshot** on connect or resync
- **ordered deltas/events** after that
- **monotonic session revision** assigned by the engine

Recommended model:

```text
Engine -> Client
  SessionHello
  SessionSnapshot { revision, state }
  SessionDelta { from_revision, to_revision, changes[] }
  PresenceUpdate
  DeviceCatalogUpdate
  Heartbeat
```

Best practice:

- deltas should be idempotent or replay-safe
- clients must be able to detect a revision gap and request a fresh snapshot
- snapshots should include enough state to render immediately without waiting for replay

### 5. Decide what is global vs client-local

This is one of the most important product decisions.

Recommended **global shared session state**:

- project content
- transport play/stop/tempo/quantize
- global loop and track loop data
- track arm/mute/solo/passthrough
- routing and MIDI FX configuration
- recording state
- undoable edits
- Link transport participation state

Recommended **client-local state** by default:

- selected page
- mappings overlay visibility
- discoverability overlay visibility
- UI scale and theme preference
- temporary pointer hover/drag affordance
- local selection focus for browsing
- local widget focus and local page-specific cursor state
- device-specific presentation/layout preferences

Potentially shared only if explicitly enabled:

- active track selection
- selected notes/regions
- edit caret/playhead-follow mode

A good default is:

- **transport is shared**
- **document edits are shared**
- **navigation chrome is local**
- **editing selection is lease-controlled**

### 6. Add edit leases instead of global UI ownership

Multiple people will otherwise stomp each other.

Recommended model:

- optimistic shared viewing
- explicit short-lived **leases** for destructive or high-conflict edit scopes

Example lease scopes:

- transport controls
- track N note editing
- mappings table editing
- routing page editing
- global settings

Lease rules:

- leases should time out automatically
- clients should see who holds the lease
- read-only viewing should remain available without leases
- non-conflicting actions can still proceed concurrently

This avoids over-serializing the whole app while still protecting conflict-heavy areas.

### 7. Make undo/redo engine-owned

Undo cannot remain purely local once multiple clients edit one session.

Recommended rule:

- the engine owns the canonical undo log
- undo/redo operates on committed session commands, not local UI intentions
- local client-only chrome changes are not part of shared undo

Best practice:

- record enough metadata to show who performed the action and what scope it affected
- group high-frequency gestures into undo transactions on the engine side

### 8. Separate real-time timing from replication timing

The engine must not wait on the network to schedule MIDI.

Use separate loops:

- **real-time engine loop** for transport and MIDI scheduling
- **replication loop** for snapshots/deltas/presence/device catalog updates
- **client render loop** for drawing

The replication loop can be slower than the real-time loop. For example:

- MIDI/transport clocking: sample/tick accurate inside the engine
- state replication: event-driven plus capped broadcast rate
- rendering: local frame rate per client

Important rule:

- remote clients render a reflected state view; they do not become the timing authority

Single-device implication:

- when no remote clients are connected, replication work should collapse to the minimum needed for the local shell instead of running as if the app were broadcasting over Wi-Fi

### 9. Add clock synchronization for display quality

Clients need an estimate of engine time to render playhead motion smoothly.

Recommended approach:

- engine sends transport snapshots with engine monotonic timestamp and musical position
- clients estimate offset and drift relative to engine time
- clients may interpolate playhead rendering locally between authoritative updates
- clients must snap back to authoritative state on divergence

This gives smooth visuals on Ethernet and acceptable recovery on Wi-Fi without making the client authoritative.

### 10. Treat attached local hardware as normal command producers

For the mixed scenario where a keyboard is attached to the Pi while a remote touch display is also connected:

- local keyboard/MIDI/touch devices connected to the engine should generate the same canonical commands as remote clients
- the engine should stamp these with a distinct client/source identity such as `engine-local-keyboard`
- remote clients should observe resulting state changes exactly as they would for remote-originated commands

This keeps behavior consistent and makes remote session logs understandable.

## Networking Best Practices

### Transport choices

Prefer a transport abstraction rather than baking networking into the UI.

Recommended priority:

1. **TCP/WebSocket or QUIC over Ethernet/Wi-Fi** for full-state display/control clients
2. **BLE only for lightweight remote control or provisioning** unless testing proves full replicated UI is acceptable on the target device

Why:

- full replicated state plus multiple clients is easier over a reliable ordered stream
- BLE throughput and packet sizing are awkward for rich UI replication
- Wi-Fi/Ethernet can carry snapshots, deltas, presence, and device catalogs much more comfortably

Practical guidance:

- if implementation speed matters most, start with length-prefixed messages over TCP or WebSocket
- if roaming/reconnect/latency tuning becomes central, consider QUIC later
- do not make the session protocol depend on SDL or a browser stack

### Message design

- use versioned protocol messages from day one
- keep messages small and typed
- separate high-rate transport telemetry from low-rate document/config changes
- never require clients to reconstruct truth from lossy ephemeral events alone

Suggested channel split:

- reliable ordered channel: commands, acks, snapshots, document deltas, undo events, leases
- throttled telemetry channel: transport position updates, meter-like status, presence pings

Even if both channels initially run over one TCP connection, keep them logically separate in the protocol.

### Reconnect and spotty link handling

Required behavior:

- client keeps last good snapshot revision
- client retries commands with stable command ids until acked or rejected
- engine deduplicates repeated command ids
- if client misses revisions, engine sends a fresh snapshot
- UI should clearly show `connected`, `degraded`, `reconnecting`, or `read-only stale`

Best practice:

- avoid pretending the client still has live authority when disconnected
- let clients keep rendering stale state, but disable destructive controls unless an offline mode is explicitly designed

### Presence and capability reporting

Track per-client metadata:

- client id and display name
- connection quality
- input capabilities: touch, keyboard, MIDI, pointer, hardware buttons
- role: viewer, performer, editor, admin
- current page and edit lease

This is needed for multi-user clarity and for routing context-sensitive UI hints.

### Bluetooth guidance

Bluetooth can be useful, but it should be scoped carefully.

Recommended use cases:

- transport controls
- track mute/solo/arm toggles
- simple status display
- provisioning Wi-Fi credentials or pairing metadata

Higher-risk use cases on BLE:

- full multi-page state replication
- dense timeline editing UIs
- multiple concurrent rich clients

Recommendation:

- treat BLE as a secondary control plane first
- use Wi-Fi/Ethernet for full collaborative screens

## Session Semantics For Multi-User Work

### Roles

Start with explicit roles:

- **viewer**: can observe only
- **controller**: can trigger transport and assigned actions
- **editor**: can change document state within lease rules
- **admin**: can manage routing, clients, and session policy

### Conflict policy

Use simple deterministic rules first:

- commands are applied in engine receive order
- commands rejected due to lease/capability violations produce explicit rejection messages
- clients show why the action did not apply

Do not silently drop conflicting edits.

### Shared selection policy

The repo should avoid conflating “what I am looking at” with “what the session is editing.”

Recommended initial rule:

- each client has a local browse selection
- a shared edit selection exists only while holding an edit lease for that scope

This gives collaborative visibility without forcing every cursor movement into the shared session.

## Data Model Additions

The current project model is not enough by itself. Add:

- `SessionId`
- `ClientId`
- `SessionRevision`
- `CommandId`
- `ClientPresence`
- `ClientCapabilities`
- `LeaseScope`
- `LeaseHolder`
- `EngineSnapshot`
- `EngineDelta`
- `CommandAck` / `CommandReject`
- `ConnectionQuality`

Likely also needed later:

- `SessionPolicy` for role and lease configuration
- `ClientLocalState` persisted per controller/display
- `ClientUiState` for live per-device navigation/focus/layout state
- `DeviceEndpoint` abstraction for local-vs-remote control surfaces

## Security And Safety

Even on a local network, add basic controls:

- explicit pairing or join approval for edit-capable clients
- role-based permissions
- session token after join
- engine-side validation for every command
- optional read-only guest mode

Never trust the client to enforce routing, recording, or destructive edit constraints.

## Recommended Migration Plan

### Phase 1: Internal separation without networking

Goal:

- separate core session state from UI-local state
- move action application into a reducer-like core
- move transport/MIDI timing into an engine runtime module
- prove that the local-only path is still lightweight after the split

Exit criteria:

- the local SDL app talks to the engine boundary through the same command/state API that remote clients will use
- local single-device performance remains at least comparable to the current architecture for idle/render/input workloads

### Phase 2: In-process loopback client

Goal:

- run a local client against an in-process engine task using snapshots and deltas

Exit criteria:

- renderer no longer reads mutable engine internals directly
- command ids, revisions, and acks exist even before network transport

### Phase 3: Single remote client over reliable transport

Goal:

- support one remote display/controller
- implement connect, snapshot, deltas, ack/reject, reconnect, and stale-state UX

Exit criteria:

- Pi-hosted engine + separate display device can fully control and observe a session over Ethernet/Wi-Fi

### Phase 4: Multi-client presence and leases

Goal:

- support multiple concurrent displays/controllers
- add presence, roles, and edit leases

Exit criteria:

- two or more users can share a session without hidden state stomping

### Phase 5: Bluetooth or constrained-link profile

Goal:

- add a reduced-bandwidth control profile for BLE or similarly constrained transports

Exit criteria:

- low-bandwidth clients use a capability-limited protocol/profile rather than pretending to be full rich clients

## Recommended Near-Term Repo Tasks

1. extract a `SessionCore` reducer from `App` for project/transport/routing mutations
2. move page/overlay/focus/viewport state into a clearly client-local structure
3. create `ClientCommand`, `EngineEvent`, `SessionSnapshot`, and `SessionDelta` types
4. introduce session revisioning and command ack/reject plumbing in-process
5. move undo ownership behind the engine boundary
6. create an engine-hosted loopback client path for the current SDL shell
7. define lease scopes for transport, timeline editing, mappings, and routing
8. prototype a simple reliable wire protocol over TCP/WebSocket on the Pi target before considering BLE-rich UI replication
9. add profiling checks that compare single-device local mode versus distributed-capable local mode before enabling remote features by default

## Biggest Risks

- keeping page/selection state global and making collaboration frustrating
- letting remote latency leak into the MIDI timing path
- trying to solve full peer-to-peer editing too early
- treating BLE as equivalent to Wi-Fi for replicated UI workloads
- leaving undo local when document edits become shared
- failing to provide explicit conflict and reconnect UX
- accidentally forcing single-device users to pay serialization, tasking, or process overhead that only remote mode needs

## Recommended Default Product Policy

For the first distributed version:

- engine-authoritative single writer
- transport and document state shared
- page/layout chrome local per client
- one reliable network protocol with snapshots + deltas + command acks
- multi-user editing protected by short-lived leases
- Wi-Fi/Ethernet first for rich displays
- BLE limited to lightweight control profiles unless proven sufficient in practice
- local single-device mode uses the same architecture with an optimized in-process loopback path

This gives the cleanest path from the current codebase to a Pi-hosted collaborative system without sacrificing timing integrity.
