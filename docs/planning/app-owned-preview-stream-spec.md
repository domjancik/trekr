# App-Owned Preview Stream Spec

Implement an app-owned network preview/recording stream for `trekr` so OBS can capture the Raspberry Pi KMSDRM output without using `ffmpeg kmsgrab` or competing for the DRM device.

## Context

- Repo: `C:\Users\magne\dev\trekr`
- App is Rust/SDL3 and can run on Pi with `--video-mode kmsdrm-console`.
- External `ffmpeg kmsgrab` blocks `trekr` because both compete for the KMSDRM device.
- Capture while `trekr` is running must be produced inside `trekr` after rendering or from an app-owned framebuffer/capture path.
- Follow `AGENTS.md` and `trekr` repo conventions.
- Keep `README.md` updated.
- Do not refresh tracked screenshots unless UI changes materially.

## Goal

Add an optional network preview server suitable for OBS recording on another machine.

## Preferred MVP

- Add a CLI/env opt-in, disabled by default:
  - CLI example: `--preview-stream mjpeg:0.0.0.0:8090`
  - Env fallback acceptable: `TREKR_PREVIEW_STREAM=0.0.0.0:8090`
- Serve an MJPEG HTTP endpoint:
  - `http://<pi-ip>:8090/preview.mjpg`
  - optional health endpoint: `http://<pi-ip>:8090/`
- OBS source type should be Browser Source or Media Source if compatible; document the working choice.
- Stream should mirror the rendered `trekr` frame, not desktop capture.
- Target 10-15 FPS by default to keep Pi CPU reasonable.
- If configurable, support `--preview-fps <n>` or env equivalent.
- Do not block rendering, MIDI, or transport timing.
- If encoding/copying falls behind, drop frames instead of buffering latency.
- Keep dependencies small and compatible with the Pi build path.

## Implementation Guidance

1. Inspect existing render/capture code first, especially renderer-owned screenshot/capture modules.
2. Reuse existing frame readback/capture helpers if possible.
3. Add a small background server/thread that receives latest encoded frame or latest raw frame snapshot through a bounded/latest-value channel.
4. Keep network streaming isolated from core MIDI/timeline logic.
5. Avoid holding SDL/renderer locks from the HTTP serving thread.
6. Prefer a single latest-frame buffer with atomic/mutex swap over queues.
7. Document expected OBS setup in `README.md` under the Pi/KMSDRM section.

## Acceptance Criteria

- `cargo fmt` passes.
- `cargo check` passes, or clearly report environment-specific failures.
- Local desktop run still works with streaming disabled.
- Streaming disabled by default has no meaningful behavior change.
- With preview enabled, the app serves a live endpoint that OBS can consume.
- `README.md` includes exact launch command and OBS source instructions.
- No `kmsgrab`/DRM contention; `trekr` remains the owner of KMSDRM.

## Important Tradeoff

If live rendered-frame readback is expensive or awkward in SDL3, implement the smallest viable internal preview first, even at low FPS. Prefer a reliable 10 FPS MJPEG stream over a complex high-performance transport.
