import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { CachedSong, PlayerStatus } from "@/types.ts";

export const usePlayerStore = defineStore("player", () => {
  const status = ref<PlayerStatus | null>(null);
  const queue = ref<CachedSong[]>([]);
  const isQueueLoading = ref(false);
  const queueError = ref<string | null>(null);
  const isListening = ref(false);
  let startPromise: Promise<void> | null = null;
  let queueRefreshPromise: Promise<void> | null = null;
  let queueRefreshRequested = false;

  const currentSong = computed(() => status.value?.currentSong ?? null);

  const load = async () => {
    status.value = await invoke<PlayerStatus>("get_player_status");
  };

  const refreshQueue = async () => {
    queueRefreshRequested = true;

    if (queueRefreshPromise) {
      return queueRefreshPromise;
    }

    queueRefreshPromise = (async () => {
      isQueueLoading.value = true;

      try {
        while (queueRefreshRequested) {
          queueRefreshRequested = false;

          try {
            queue.value = await invoke<CachedSong[]>("get_player_queue");
            queueError.value = null;
          } catch (error) {
            queueError.value = String(error);
          }
        }
      } finally {
        isQueueLoading.value = false;
        queueRefreshPromise = null;
      }
    })();

    return queueRefreshPromise;
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
          status.value = event.payload;
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
    status,
    queue,
    isQueueLoading,
    queueError,
    currentSong,
    refreshQueue,
    startListening,
  };
});
