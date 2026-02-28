#include "link_bridge.hpp"

#include <ableton/Link.hpp>
#include <chrono>
#include <new>
#include <utility>

struct TrekrLinkHandle {
  explicit TrekrLinkHandle(double bpm)
    : link(bpm) {}

  ableton::Link link;
};

extern "C" {

TrekrLinkHandle* trekr_link_new(double bpm)
{
  return new (std::nothrow) TrekrLinkHandle(bpm);
}

void trekr_link_free(TrekrLinkHandle* handle)
{
  delete handle;
}

uint8_t trekr_link_is_enabled(const TrekrLinkHandle* handle)
{
  return (handle && handle->link.isEnabled()) ? 1 : 0;
}

void trekr_link_set_enabled(TrekrLinkHandle* handle, uint8_t enabled)
{
  if (handle) {
    handle->link.enable(enabled != 0);
  }
}

uint8_t trekr_link_is_start_stop_sync_enabled(const TrekrLinkHandle* handle)
{
  return (handle && handle->link.isStartStopSyncEnabled()) ? 1 : 0;
}

void trekr_link_set_start_stop_sync_enabled(TrekrLinkHandle* handle, uint8_t enabled)
{
  if (handle) {
    handle->link.enableStartStopSync(enabled != 0);
  }
}

void trekr_link_snapshot(
  TrekrLinkHandle* handle,
  double quantum,
  TrekrLinkSnapshot* snapshot)
{
  if (!handle || !snapshot) {
    return;
  }

  const auto micros = handle->link.clock().micros();
  const auto state = handle->link.captureAppSessionState();
  snapshot->enabled = handle->link.isEnabled() ? 1 : 0;
  snapshot->start_stop_sync = handle->link.isStartStopSyncEnabled() ? 1 : 0;
  snapshot->is_playing = state.isPlaying() ? 1 : 0;
  snapshot->reserved = 0;
  snapshot->peers = handle->link.numPeers();
  snapshot->tempo_bpm = state.tempo();
  snapshot->beat = state.beatAtTime(micros, quantum);
  snapshot->phase = state.phaseAtTime(micros, quantum);
  snapshot->micros = micros.count();
}

void trekr_link_commit_tempo(TrekrLinkHandle* handle, double bpm)
{
  if (!handle) {
    return;
  }

  auto state = handle->link.captureAppSessionState();
  const auto micros = handle->link.clock().micros();
  state.setTempo(bpm, micros);
  handle->link.commitAppSessionState(std::move(state));
}

void trekr_link_commit_playing(
  TrekrLinkHandle* handle,
  uint8_t is_playing,
  double beat,
  double quantum)
{
  if (!handle) {
    return;
  }

  auto state = handle->link.captureAppSessionState();
  const auto micros = handle->link.clock().micros();
  state.setIsPlayingAndRequestBeatAtTime(is_playing != 0, micros, beat, quantum);
  handle->link.commitAppSessionState(std::move(state));
}

} // extern "C"
