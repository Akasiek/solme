import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { ref } from "vue";
import { createPlayerLyrics } from "@/stores/player/lyrics.ts";
import { createPlayerPlayback } from "@/stores/player/playback.ts";
import { createPlayerQueue } from "@/stores/player/queue.ts";
import type { PlayerStatus } from "@/types.ts";

export const usePlayerStore = defineStore("player", () => {
  const { updateStatus, ...playbackState } = createPlayerPlayback();
  const { refreshQueue, ...queueState } = createPlayerQueue();
  const { loadLyrics, prefetchLyrics, ...lyricsState } = createPlayerLyrics();
  const isListening = ref(false);
  let startPromise: Promise<void> | null = null;

  const applyStatus = (playerStatus: PlayerStatus) => {
    updateStatus(playerStatus);
    void loadLyrics(playerStatus.currentSong?.remoteId ?? null);
  };

  const ensureCurrentLyrics = () => loadLyrics(playbackState.currentSong.value?.remoteId ?? null);

  const prefetchNextLyrics = () => {
    const queuePosition = playbackState.status.value?.queuePosition;
    const nextSong = queuePosition === undefined ? undefined : queueState.queue.value[queuePosition];
    void prefetchLyrics(nextSong?.remoteId ?? null);
  };

  const load = async () => {
    applyStatus(await invoke<PlayerStatus>("get_player_status"));
  };

  const startListening = async () => {
    if (isListening.value) {
      return;
    }

    if (startPromise) {
      return startPromise;
    }

    startPromise = (async () => {
      await Promise.all([
        listen<PlayerStatus>("player-status-changed", (event) => {
          applyStatus(event.payload);
          prefetchNextLyrics();
        }),
        listen("player-queue-changed", () => {
          void refreshQueue().then(prefetchNextLyrics);
        }),
      ]);
      isListening.value = true;
      await Promise.all([load(), refreshQueue()]);
      prefetchNextLyrics();
    })();

    try {
      await startPromise;
    } finally {
      startPromise = null;
    }
  };

  return {
    ...playbackState,
    ...queueState,
    ...lyricsState,
    ensureCurrentLyrics,
    refreshQueue,
    startListening,
  };
});
