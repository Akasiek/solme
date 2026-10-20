import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";
import type { CachedSong, PlayerStatus } from "@/types.ts";

export function createPlayerQueue(applyStatus: (status: PlayerStatus) => void) {
  const queue = ref<CachedSong[]>([]);
  const queueRevision = ref<string | null>(null);
  const queueEntryIds = ref<string[]>([]);
  const queueStatus = ref<PlayerStatus | null>(null);
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
            const snapshot = await invoke<{
              songs: CachedSong[];
              entryIds: string[];
              revision: string;
              status: PlayerStatus;
            }>("get_player_queue");
            queue.value = snapshot.songs;
            queueRevision.value = snapshot.revision;
            queueEntryIds.value = snapshot.entryIds;
            queueStatus.value = snapshot.status;
            applyStatus(snapshot.status);
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

  const moveQueueEntry = async (from: number, to: number, revision: string, currentIndex: number) => {
    try {
      await invoke("player_move_queue_entry", {
        from,
        to,
        expectedRevision: revision,
        expectedCurrentIndex: currentIndex < 0 ? null : currentIndex,
      });
    } finally {
      await refreshQueue();
    }
  };

  return {
    queue,
    queueRevision,
    queueEntryIds,
    queueStatus,
    moveQueueEntry,
    isQueueLoading,
    queueError,
    refreshQueue,
  };
}
