#pragma once

#include <stdint.h>
#include <stddef.h>

struct TrekrLinkHandle;

struct TrekrLinkSnapshot {
  uint8_t enabled;
  uint8_t start_stop_sync;
  uint8_t is_playing;
  uint8_t reserved;
  size_t peers;
  double tempo_bpm;
  double beat;
  double phase;
  int64_t micros;
};

extern "C" {
TrekrLinkHandle* trekr_link_new(double bpm);
void trekr_link_free(TrekrLinkHandle* handle);

uint8_t trekr_link_is_enabled(const TrekrLinkHandle* handle);
void trekr_link_set_enabled(TrekrLinkHandle* handle, uint8_t enabled);
uint8_t trekr_link_is_start_stop_sync_enabled(const TrekrLinkHandle* handle);
void trekr_link_set_start_stop_sync_enabled(TrekrLinkHandle* handle, uint8_t enabled);

void trekr_link_snapshot(
  TrekrLinkHandle* handle,
  double quantum,
  TrekrLinkSnapshot* snapshot);
void trekr_link_commit_tempo(TrekrLinkHandle* handle, double bpm);
void trekr_link_commit_playing(
  TrekrLinkHandle* handle,
  uint8_t is_playing,
  double beat,
  double quantum);
}
