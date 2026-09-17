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
  const isListening = ref(false);
  let startPromise: Promise<void> | null = null;

  const applyStatus = (playerStatus: PlayerStatus) => {
    updateStatus(playerStatus);
  };

  const progressTimer = window.setInterval(() => {
    const now = performance.now();
    const elapsedSeconds = (now - lastPositionUpdate) / 1000;
    lastPositionUpdate = now;

    if (status.value?.state !== "playing") {
      return;
    }

    playbackPositionSeconds.value = Math.min(
      status.value.durationSeconds,
      playbackPositionSeconds.value + elapsedSeconds,
    );
  }, 250);

  onScopeDispose(() => window.clearInterval(progressTimer));

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
        }),
        listen("player-queue-changed", () => {
          void refreshQueue();
        }),
      ]);
      isListening.value = true;
      await Promise.all([load(), refreshQueue()]);
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
    refreshQueue,
    startListening,
  };
});
