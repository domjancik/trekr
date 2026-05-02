# Thin Client Connection And Discovery Screen Spec

## Summary

Add a connection/discovery screen for `thin-client-sdl` when it is launched without an explicit `--connect` address.

The screen should let the user:

- discover available session hosts on the local network
- see a stable, human-readable session name instead of only `host:port`
- choose a host with touch or desktop input
- retry discovery or enter a manual address
- preserve the current direct-connect path when `--connect` is supplied

This spec is grounded in the current repository state:

- `README.md`
- `docs/specs/product-spec.md`
- `docs/planning/handoff-summary.md`
- `docs/planning/distributed-architecture-research.md`
- `docs/planning/federated-session-evolution-plan.md`
- `docs/dev/architecture.md`
- `src/cli.rs`
- `src/distributed.rs`
- `src/actions.rs`
- `src/app/*`

## Problem

Current thin-client startup assumes a known host address:

- `thin-client --connect <addr>`
- `thin-client-sdl --connect <addr>`

That is workable for development, but weak for the actual product direction described in the distributed docs:

- Raspberry Pi or headless host on a local network
- one or more displays/controllers joining dynamically
- touch-first operator devices
- multiple sessions possible on the same LAN

Today there is no in-app discovery or connection UX for:

- "what sessions are available?"
- "which one is mine?"
- "what is the friendly session name?"
- "how do I retry if Wi‑Fi is spotty?"
- "how do I connect if discovery fails?"

## Goals

- Launch `thin-client-sdl` without `--connect` into a dedicated discovery/connection screen.
- Keep `--connect` as the direct bypass path for development, scripting, and known-host workflows.
- Introduce automatic host advertisement on the local network.
- Introduce a human-readable session identity, not only socket address.
- Make the flow work well on both desktop and touch-first devices.
- Reuse the current action-driven architecture instead of inventing one-off input handling for the screen.
- Keep the initial network-discovery slice LAN-scoped and simple.
- Make replacement/conflict rules explicit when duplicate or stale advertisements appear.

## Non-Goals

- Internet-wide discovery.
- Authenticated remote account/session browser.
- Full peer-to-peer collaboration.
- Bluetooth discovery in the first shipped slice.
- Cross-subnet discovery guarantees.
- A generic in-app command palette for network operations.

## Current Baseline

Current code already provides:

- a headless host mode in `src/distributed.rs`
- a TCP-based session protocol
- an SDL thin client mode in `src/distributed.rs`
- CLI command parsing in `src/cli.rs`
- action-driven UI behavior through `AppAction` in `src/actions.rs`
- recent distributed refactors toward semantic intent transport

Current startup behavior:

- `thin-client-sdl` requires `--connect`
- no discovery service exists
- no advertised session metadata exists
- no connection-state UI exists before the main mirrored app UI is available

Implication:

- discovery/connection must be a pre-session shell state around the existing thin client, not a special page inside the authoritative session itself
- session selection UI is client-local UI state, not shared session state

## Recommended Discovery Model

Use a two-layer approach.

### Layer 1: local-network advertisement and discovery

Recommended primary mechanism:

- **mDNS / DNS-SD** style advertisement for the LAN

Advertised service type:

- `_trekr._tcp.local`

Why this is the preferred default:

- fits the "join on the local network" product direction
- gives a well-understood session name + host + port bundle
- naturally supports multiple simultaneous hosts
- avoids requiring users to type addresses for common cases
- maps well to desktop and mobile-class local-network UX

### Layer 2: optional lightweight fallback discovery

Recommended fallback if mDNS is unavailable on a given platform or network:

- explicit manual address entry
- optional future UDP broadcast discovery fallback

Recommended priority:

1. mDNS discovery
2. manual address entry
3. only later consider UDP broadcast fallback if mDNS platform support proves insufficient

Reasoning:

- UDP broadcast can be useful, but it is noisier, less standardized, and more environment-sensitive
- manual entry is still required anyway as an escape hatch

## Session Identity And Naming

The host should advertise both a stable machine-level address and a friendly session name.

### Required advertised fields

- `session_id`
- `session_name`
- `host_name`
- `listen_addr`
- `port`
- `protocol_version`
- `transport_capabilities`
- `host_mode`
  - for example: `run-listen` or `host-session`
- `current_client_count`
- optional `state_mode`
  - for example: `demo`, `empty`, or `persisted`

### Session name rules

Recommended session name precedence:

1. explicit operator-supplied session name
2. persisted project/session label if available
3. generated fallback name

Generated fallback examples:

- `trekr on raspberrypi`
- `Demo Session on MAGNE-PI`
- `Persisted Session on studio-host`

Requirements:

- friendly names must be short enough for touch UIs
- session name is presentation metadata, not authority identity
- `session_id` remains the actual stable identity

## Launch Behavior

### SDL thin client

#### If `--connect` is supplied

- skip discovery screen entirely
- connect directly using the existing behavior
- if the connection fails, show a local connection-error screen with:
  - error summary
  - retry
  - edit address
  - back to discovery

#### If `--connect` is not supplied

- start on the discovery/connection screen
- immediately begin discovery
- show available sessions as they appear
- allow manual entry even while discovery is running

### Terminal thin client

Keep the first slice simpler:

- terminal thin client may continue to require `--connect` initially
- optional later enhancement: text-mode discovery list

This keeps the first UX investment focused on the SDL client, where touch/discovery matters most.

## UX Flow

### Primary flow

1. user launches `thin-client-sdl`
2. client enters local `Discovery` state
3. screen shows:
   - title
   - spinner / searching state
   - discovered sessions list
   - manual connect affordance
4. user selects a discovered session
5. client enters `Connecting`
6. on success, client transitions into the existing mirrored app shell
7. on failure, client enters `ConnectFailed` and offers retry/back/manual connect

### No sessions found flow

If no sessions are found after a reasonable interval:

- keep discovery active
- show a calm empty state, not a hard error
- offer:
  - `Retry`
  - `Manual Address`
  - `Refresh`

Recommended copy shape:

- `Searching for trekr sessions...`
- `No sessions found yet`
- `Check that a host is running on this network, or connect manually`

### Stale or disappearing session flow

If a discovered host disappears before selection:

- remove it from the list
- if it was highlighted, move selection predictably
- do not keep obviously stale entries indefinitely

If a host disappears during connection attempt:

- show `Session no longer available`
- offer `Back to Discovery` and `Retry`

## Screen State Machine

This screen should be modeled as client-local state, not as server session state.

Recommended states:

1. `Discovery`
   - active search running
2. `DiscoveryEmpty`
   - no sessions found yet
3. `DiscoveryResults`
   - one or more sessions found
4. `ManualAddressEntry`
   - local text-entry mode for address input
5. `Connecting`
6. `ConnectFailed`
7. `Connected`
   - hands off to the existing mirrored thin-client runtime

## Action Model Reuse

Do not treat the discovery screen as an exception to the action model.

Recommended approach:

- add a small set of canonical **client-shell actions** for connection UX
- keep them local to the thin-client shell rather than sending them to the host

Recommended actions:

- `StartDiscovery`
- `RefreshDiscovery`
- `StopDiscovery`
- `SelectNextDiscoveredSession`
- `SelectPreviousDiscoveredSession`
- `ActivateSelectedDiscoveredSession`
- `OpenManualConnect`
- `EditManualConnectAddress`
- `SubmitManualConnect`
- `CancelManualConnect`
- `RetryConnection`
- `BackToDiscovery`

Important rule:

- these actions are **not** shared session actions and should **not** go over the host protocol
- they belong to the thin-client local shell layer, similar to pre-session UI state

This keeps the architecture consistent:

- session actions remain canonical app/session actions
- connection-shell actions remain canonical local shell actions

## Desktop And Touch Interaction

### Desktop

Required interactions:

- arrow keys move selection
- `Enter` connects to selected session
- `R` or a visible refresh button refreshes discovery
- `M` or visible button opens manual address mode
- `Escape` backs out of manual-entry/failure sub-states where applicable
- pointer click selects and activates list rows/buttons

### Touch

Required interactions:

- session rows must be comfortably tappable
- refresh and manual-connect affordances must be explicit buttons
- manual address mode must not depend on hover
- the selected session should be visually obvious without precision cursor affordances
- first tap selects, second tap or explicit `Connect` button may activate if accidental joins are a concern

Recommended touch behavior:

- if only one session is present and the user taps it, connect immediately
- if several sessions are present, either:
  - connect immediately on tap, or
  - use select-then-connect if testing shows accidental misjoins

Preferred first-slice recommendation:

- **single tap selects, explicit `Connect` button activates**

That is safer for touch and still acceptable on desktop.

## Scope Behavior

Connection/discovery state is **per-client UI state**.

That means:

- one thin client choosing a host does not affect another thin client's discovery screen
- session list selection is local
- manual address text is local
- discovery status is local
- once connected, the client transitions into the normal per-client UI/session model already described in `docs/planning/distributed-architecture-research.md`

## Conflict And Replacement Rules

### Duplicate advertisements

If several advertisements refer to the same `session_id`:

- deduplicate them into one list entry
- prefer the freshest advertisement
- if transport metadata differs, prefer the highest compatible protocol version
- optionally surface alternate addresses in a details view later, but not in the first slice

### Duplicate names

If two sessions share the same `session_name`:

- keep both entries
- disambiguate with host name and/or address

Example:

- `Demo Session — raspberrypi.local`
- `Demo Session — studio-host.local`

### Protocol mismatch

If a discovered host advertises an incompatible protocol version:

- show it as unavailable or incompatible
- do not allow connection without an explicit override path

### Full host / admission refusal

If the host refuses a connection because of a future client limit or policy:

- show a clear local error state
- keep the user on the client shell
- do not enter the mirrored session UI

## Automatic Reconnect Relationship

The discovery screen spec should compose with current reconnect behavior.

Recommended rule:

- if a client was connected and then disconnected unexpectedly, first try silent reconnect to the same host
- only return to the discovery screen if reconnect fails or the host disappears long enough

This keeps discovery from becoming disruptive during transient Wi‑Fi drops.

## Acceptance Criteria

### Core startup behavior

- launching `thin-client-sdl` without `--connect` opens a discovery screen instead of failing argument parsing
- launching `thin-client-sdl --connect <addr>` still bypasses discovery and connects directly

### Discovery behavior

- a host launched with `run --listen` or `host-session --listen` advertises itself on the LAN
- the discovery screen lists compatible hosts automatically
- each list entry shows at minimum:
  - session name
  - host name or address
  - client count
- empty-state copy is shown when no sessions are found

### Manual connect behavior

- the user can enter `host:port` manually from the discovery screen
- invalid manual addresses produce a local validation message
- failed connection attempts show a recoverable error state

### Interaction behavior

- desktop keyboard-only navigation can discover, select, and connect
- mouse users can click through the entire flow
- touch users can complete the flow without hover or keyboard assumptions

### Conflict behavior

- duplicate advertisements for the same session do not produce duplicated list rows
- duplicate names remain connectable and visually distinguishable
- incompatible protocol versions are clearly marked and not silently joined

### Architecture behavior

- connection-screen actions are local shell actions and are not forwarded into the session protocol
- once connected, the existing thin-client mirrored app behavior still takes over cleanly
- the host protocol remains session-oriented; discovery is adjacent, not mixed into the session command stream

## Likely Code Touch Points

### CLI and startup

- `src/cli.rs`
  - make `thin-client-sdl` accept optional `--connect`
  - keep terminal thin client stricter for now unless intentionally expanded

### Thin-client runtime

- `src/distributed.rs`
  - add discovery screen state machine
  - add advertiser lifecycle for hosts
  - add discovery client lifecycle for SDL thin client
  - add manual-address connection flow and failure states

### New local shell model

Likely new modules or internal sections:

- thin-client local shell state
- discovered-session metadata model
- discovery transport abstraction
- advertiser abstraction

### Diagnostics

- `src/diagnostics.rs`
  - log discovery start/stop, advertisements found, connect attempts, and failures

### Documentation

- `README.md`
  - runnable surface and quick-start commands once implementation ships
- `docs/planning/distributed-architecture-research.md`
  - optional short cross-reference after implementation if design constraints evolve

## Recommended Implementation Order

1. define discovered-session metadata and local shell state
2. allow `thin-client-sdl` without `--connect`
3. implement manual-connect screen first
4. add host advertisement and discovery list
5. add session naming and protocol-version metadata
6. add reconnect integration and stale-entry handling

This order ensures the UX is still usable even if network discovery support varies by platform.

## Open Questions

- exact crate choice for mDNS / DNS-SD support across target platforms
- whether to expose explicit session naming on `host-session` and `run --listen` at the CLI level in the first slice
- whether touch should connect on first tap or require explicit `Connect`
- whether terminal thin client should gain discovery in the same milestone or later

## Recommendation

Ship the first discovery slice with:

- optional `--connect` for `thin-client-sdl`
- local discovery/connection screen
- manual connect fallback
- mDNS advertisement/discovery as the primary LAN path
- explicit friendly session names
- local-shell actions separated from host session actions

That best fits the current repo direction:

- action-driven UI
- engine-authoritative session model
- per-client UI ownership
- headless-host plus thin-client deployments on local networks
