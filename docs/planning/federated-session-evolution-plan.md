# Federated Session Evolution Plan

## Purpose

This document captures the longer-term path from the current engine-authoritative distributed architecture toward a future where:

- participants can join and leave fluidly
- each participant may bring their own tracks, saved loops, rhythm-session assets, and connected MIDI hardware
- multiple devices may edit a shared session
- Ableton Link provides the real-time synchronization glue
- collaborative document ownership can later evolve toward CRDT or replicated-op models where needed

Primary context:

- `docs/dev/architecture.md`
- `docs/planning/handoff-summary.md`
- `docs/planning/implementation-plan.md`
- `docs/planning/distributed-architecture-research.md`

## Core Framing

The future system should be treated as four related layers, not one monolith:

1. **real-time sync layer**
   - shared tempo
   - beat phase
   - optional transport participation
   - best handled by Ableton Link
2. **shared session/document layer**
   - tracks
   - clips/loops
   - saved rhythm-session assets
   - mappings, metadata, and edit history
3. **endpoint fabric**
   - MIDI inputs and outputs physically attached to different participant devices
   - endpoint capability advertisement and availability
4. **per-device UI layer**
   - page state
   - overlays
   - focus
   - viewport/density/theme/scaling
   - local browsing and presentation state

This separation is what keeps the path realistic.

## Recommended Long-Term Principle

Use a **hybrid distributed model**:

- **Link for time**
- **shared session protocol for state**
- **endpoint-local execution for hardware**
- **engine-authoritative or coordinator-authoritative runtime for near-term collaboration**
- **CRDT or replicated-op document ownership only where multi-writer editing truly needs it later**

Do not try to make every concern peer-to-peer at once.

## Why This Path Fits The Current Architecture

The current direction already helps because:

- the action model gives a canonical command surface
- distributed research already separates session state from per-device UI state
- Link is already a known transport-sync concept in the repo
- the product is already MIDI-first, which makes endpoint identity and routing important from the start

What this means in practice:

- the present engine-authoritative model is still the right next step
- but the data model should avoid assuming all hardware belongs to one host forever

## Ableton Link's Role

Ableton Link should be treated as the **real-time synchronization glue**, not as the full distributed session protocol.

Link is a good fit for:

- tempo synchronization
- beat-phase synchronization
- optional start/stop participation
- low-friction join/leave behavior for live collaboration

Link does **not** solve:

- shared document replication
- track/loop ownership
- endpoint discovery and capabilities
- routing persistence
- permissions and leases
- undo/history
- conflict resolution
- asset transfer or merge semantics

Recommended rule:

- use Link to keep participants musically aligned in time, while a separate session layer manages content and coordination

## Session Model For Bring-Your-Own-Rig Collaboration

Future sessions may include participants who bring both:

- their own musical material
- their own attached MIDI hardware

That means the session model should support:

- participant identity
- participant-contributed tracks/clips/loops/assets
- participant-advertised MIDI endpoints
- routing assignments that reference abstract endpoints instead of only local host ports

The session should not assume:

- all MIDI I/O is attached to the engine host
- all playback must be physically emitted from one machine
- one device owns all routings forever

## Endpoint Fabric Model

Treat MIDI devices as network-visible **endpoints** with stable ids and metadata.

Recommended concepts:

- `ParticipantId`
- `DeviceEndpointId`
- `EndpointHostClientId`
- `EndpointCapabilities`
- `EndpointAvailability`
- `RoutingAssignment`

Important distinction:

- **logical routing** belongs to the shared session
- **physical execution** belongs to the device that actually owns the endpoint

So a track may be assigned to an endpoint that lives on another participant's machine, while that remote machine remains responsible for opening and driving the actual MIDI port.

## Authority Model Over Time

### Near term

- one engine or coordinator is authoritative for session state and transport decisions
- attached local hardware is primary
- remote displays/controllers are clients

### Intermediate phase

- one coordinator remains authoritative for session/document mutations
- participant devices can advertise their own endpoints
- endpoint execution becomes distributed even if document authority stays centralized

### Longer term

- collaborative document ownership can become replicated or CRDT-backed for selected objects
- endpoint ownership remains local to each participant device
- Link remains the realtime glue for tempo/phase alignment

This is the likely stable split:

- **time is federated through Link**
- **hardware is federated through endpoint ownership**
- **document ownership may gradually decentralize where useful**

## What Should Become Mergeable Later

Not everything needs CRDT treatment.

Good candidates for later replicated ownership:

- track metadata
- clip and loop collections
- rhythm-session assets
- arrangement objects
- non-realtime mappings and annotations
- participant-contributed content packs

Poor candidates for full peer-authoritative ownership:

- sample-accurate playback timing
- live MIDI scheduling
- immediate transport clock decisions
- short-horizon realtime note emission

Recommendation:

- keep the live runtime authoritative/coordinated even if document objects become multi-writer later

## Design Constraints To Preserve Now

To keep this path open, preserve these constraints immediately:

### 1. Stable ids everywhere

Use durable ids for:

- participants
- clients/devices
- tracks
- clips/regions/loops
- mappings
- endpoints
- commands/operations
- sessions

### 2. Action/operation boundary

Keep edits flowing through canonical actions now, so they can later map to replicated operations.

### 3. Document vs runtime split

Keep these clearly separate:

- shared document state
- realtime runtime state
- per-device UI state
- endpoint inventory/capabilities

### 4. Provenance

Track who created or modified objects, especially for imported tracks, loops, and saved session material.

### 5. Object-level mutation

Avoid project-wide blind replacement when possible; prefer operations against identified objects.

## Multi-User Editing Strategy

A practical staged strategy is:

### Stage A: coordinator-owned edits

- all edits still flow through one authoritative engine/coordinator
- users can join, contribute material, and edit shared state through that coordinator

### Stage B: replicated object history

- object histories become more operation-oriented
- imported assets and tracks can retain provenance and merge metadata

### Stage C: selective CRDT adoption

- only high-value shared document objects adopt CRDT/replicated-op semantics
- realtime scheduling and endpoint execution remain outside that layer

This avoids overcommitting the whole product to CRDT complexity too early.

## Join And Leave Semantics

A future session should support:

- participant joins and advertises endpoints/capabilities
- participant may import or expose tracks/loops/assets to the session
- routing may bind shared tracks to participant-owned endpoints
- participant can leave without corrupting the document
- missing endpoints degrade gracefully instead of destroying shared musical content

Recommended behavior on leave:

- content remains in the session unless explicitly removed
- endpoint-backed routing is marked unavailable
- session shows unresolved routings clearly
- Link participation disappears naturally with the leaving device

## Per-Device UI State In This Future

Even in the federated future, each connected device should keep its own UI state by default.

That means:

- one participant can browse Routing while another stays on Timeline
- one device can use a dense desktop layout while another uses a touch layout
- edit selections can remain local unless deliberately shared through leases or collaborative editing scopes

This is important because federated sessions increase display diversity rather than reducing it.

## Headless Engine Plus Thin-Client Displays

The future architecture should also support a mode where:

- one device runs the engine headlessly
- one or more other devices act as thin display/control clients
- input hardware may still be attached to the headless engine host

In this mode:

- shared session state is still owned by the engine/coordinator
- each display client owns its own UI state
- engine-local keyboard or MIDI inputs still enter through the same canonical action path as remote inputs

This avoids coupling “who renders the UI” to “who captures the input hardware.”

## Who Owns UI State

Recommended rule:

- the device that renders a UI owns that UI state's local navigation, overlays, focus, and presentation preferences

So:

- a thin client display owns its own page, overlay, focus, and layout state
- another thin client display owns a different UI state object
- a headless engine host owns no visual UI state by default unless it also runs a local UI shell

Shared session logic should never assume that the engine host automatically owns the “real” UI state.

## Resolving UI-Dependent Shortcuts

Some actions are global and self-contained, such as:

- play/stop
- record
- transport toggles

Other shortcuts are context-sensitive and depend on UI state, such as:

- page-relative selection
- focused-widget adjustment
- delete/activate within the currently focused page element
- shortcuts whose meaning changes with overlays or local editing mode

In distributed and thin-client scenarios, these should resolve using an explicit rule:

### Rule 1: context-free shortcuts can execute anywhere

If an action does not depend on UI focus or local page state, the engine can apply it directly regardless of where the input originated.

### Rule 2: UI-dependent shortcuts must target a specific UI state owner

If a shortcut depends on UI context, the command should carry or resolve a target client UI context, such as:

- `engine-local-keyboard` targeting thin client A's UI state
- remote keyboard on client B targeting client B's own UI state

Suggested command-envelope addition:

- `target_ui_client_id`

That allows the engine to interpret context-sensitive shortcuts against the correct per-client UI state.

### Rule 3: headless hosts need explicit binding for contextual shortcuts

If a keyboard is attached to a headless engine host, and that keyboard triggers context-sensitive shortcuts, the system should explicitly bind that keyboard to:

- one specific thin client's UI state, or
- a dedicated non-visual operator UI state, if such a mode exists later

Without that binding, context-sensitive shortcuts are ambiguous and should not be interpreted implicitly.

### Rule 4: shared session actions should remain canonical after resolution

Even when a shortcut depends on a target UI state, it should resolve into a canonical shared action after the context lookup step.

That means:

- raw shortcut -> resolve against target UI state -> canonical action or command -> apply to shared session

This preserves the action model while still allowing per-client UI ownership.

## Security And Trust Model

Bring-your-own-rig collaboration requires explicit trust boundaries.

Recommended baseline:

- participants authenticate to join a session
- endpoint advertisement is permissioned
- routing to participant-owned endpoints is explicit
- edit permissions are role-based
- imported content retains provenance

Do not assume that joining the Link session alone implies full permission to edit or route shared session content.

## Recommended Migration Path

### Phase 1: engine-authoritative distributed sessions

- finish the engine/client/session split from `docs/planning/distributed-architecture-research.md`
- preserve per-device UI state
- keep single-device local mode lightweight

### Phase 2: abstract MIDI endpoints

- promote MIDI ports from local host handles to abstract endpoint objects with ids and capabilities
- keep actual local port ownership at the device that hosts them

### Phase 3: participant presence and advertised endpoints

- let clients join with identity, capabilities, and available endpoints
- support routing assignment against endpoint ids

### Phase 4: participant-contributed session objects

- support tracks/loops/assets contributed by participants
- record provenance and ownership metadata

### Phase 5: operation-oriented document history

- move shared document edits toward replayable object-level operations
- keep undo/history/coordinator semantics compatible with future replication

### Phase 6: selective replicated ownership

- evaluate CRDT or equivalent replicated-op models for the shared document areas that need multi-writer editing
- keep transport timing and endpoint execution out of the CRDT critical path

## Highest-Value Near-Term Decisions

1. decide whether future routing should target abstract endpoint ids rather than only local ports
2. define participant identity and presence types early
3. preserve provenance fields for imported/shared content
4. keep actions and future operations structurally close
5. avoid coupling Link state to document ownership semantics
6. treat per-device UI state as local by default even in collaborative mode

## Summary Recommendation

Yes, there is a coherent path from the current architecture to a future where:

- people join and leave fluidly
- each device can bring its own MIDI hardware
- Ableton Link provides the realtime synchronization glue
- shared session state becomes richer over time
- document collaboration can later evolve toward CRDT-backed ownership where useful

The safest and cleanest path is not to jump straight to full peer-to-peer everything.

Instead, evolve in layers:

- keep **Link for time**
- build a **shared session layer for state**
- model **hardware as participant-owned endpoints**
- preserve **per-device UI state**
- move toward **replicated document ownership only where it adds real value**
