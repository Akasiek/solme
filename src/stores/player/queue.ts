import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";
import type { CachedSong } from "@/types.ts";

export function createPlayerQueue() {
  const queue = ref<CachedSong[]>([]);
  const isQueueLoading = ref(false);
  const queueError = ref<string | null>(null);
  let refreshPromise: Promise<void> | null = null;
  let refreshRequested = false;

  const refreshQueue = async () => {
    refreshRequested = true;

    if (refreshPromise) {
      return refreshPromise;
    }

    refreshPromise = (async () => {
      isQueueLoading.value = true;

      try {
        while (refreshRequested) {
          refreshRequested = false;

          try {
            queue.value = await invoke<CachedSong[]>("get_player_queue");
            queueError.value = null;
          } catch (error) {
            queueError.value = String(error);
          }
        }
      } finally {
        isQueueLoading.value = false;
        refreshPromise = null;
      }
    })();

    return refreshPromise;
  };

  return {
    queue,
    isQueueLoading,
    queueError,
    refreshQueue,
  };
}
