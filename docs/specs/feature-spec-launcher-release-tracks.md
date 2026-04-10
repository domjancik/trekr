# Feature Spec: Launcher/App Release Tracks

## Purpose

Define release policy and artifact contracts so:

- Trekr app builds remain continuously downloadable by branch/channel.
- `trekr-launcher` ships on its own cadence.
- launcher install logic reliably selects app artifacts (never launcher artifacts).

Grounding:

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `.github/workflows/launcher-release.yml`
- `src/launcher/installs.rs`
- `docs/specs/feature-spec-build-launcher.md`
- `docs/specs/feature-spec-build-launcher-ui.md`

## Policy

1. **Always build launcher in CI**
   - Every normal CI run compiles `trekr-launcher` to catch breakage early.
2. **Publish launcher only on launcher changes**
   - launcher release workflow uses path filters for launcher-related code/docs/workflow changes.
   - manual publish remains available via `workflow_dispatch`.
3. **Separate release tracks**
   - **App track**: `app-<branch>-<run>-<sha>`
   - **Launcher track**: `launcher-<branch>-<run>-<sha>`
   - both tracks are prerelease-oriented by default for branch iteration.

## Artifact Contract

### App track (consumed by launcher installer)

- Tags start with `app-`.
- Assets include platform bundles for Trekr app runtime:
  - Linux/macOS: `.tar.gz`
  - Windows: `.zip`
- Asset names include platform tokens (`linux`, `macos`/`darwin`, `windows`/`win64`).

### Launcher track

- Tags start with `launcher-`.
- Assets bundle `trekr-launcher` binary + required SDL runtime files.
- Launcher artifacts are not valid app install sources.

## Launcher Installer Behavior

- Release selection prefers tags starting with `app-`.
- If no `app-` tags exist (legacy repos), fallback to non-draft releases.
- Branch matching uses normalized branch/tag/name tokens (`/`, `_`, and spaces normalized).
- Asset selection supports `.zip`, `.tar.gz`, `.tgz`, and `.tar`.
- Extraction must handle both zip and tar-based archives.

## Acceptance Criteria

1. CI compiles `trekr-launcher` on routine pushes/PRs.
2. App release workflow publishes only app-tagged releases (`app-*`).
3. Launcher release workflow publishes only launcher-tagged releases (`launcher-*`) and only on launcher-path changes (or manual dispatch).
4. Launcher install flow selects app releases and never picks launcher releases when both are present.
5. Linux/macOS artifact installs work from tar-based archives; Windows installs work from zip archives.
6. Existing optional source-build fallback remains opt-in and unchanged in default policy.
