# Feature Spec: Stored Loops (Future / Aspirational)

This document tracks stored-loop capabilities intentionally deferred beyond shipped V1.

## Data Model Extensions

- stable per-loop ids decoupled from slot index
- user-visible loop names/labels
- optional explicit color metadata per stored loop

## Workflow Extensions

- create stored loops directly from selected regions
- richer overwrite/duplicate management
- bank-level import/export or copy workflows

## UI Extensions

- expanded stored-loop inspector/editor for active track
- richer inline editing for start/end/length and naming
- advanced overflow management beyond current fit-to-width slot rendering

## Action/Mapping Extensions

- optional explicit `Recall Stored Loop Slot X Quantized` action variants
- `Next Stored Loop` / `Previous Stored Loop` navigation actions
- potential broader absolute-track defaults depending on live workflow feedback

## Transport/Launch Extensions

- additional launch modes beyond current grid + `LoopEnd`
- optional policy controls for queue persistence/behavior during deeper transport state changes
