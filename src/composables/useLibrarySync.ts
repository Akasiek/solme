import { computed, ref, watch, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useIntervalFn } from "@vueuse/core";

import type { LibrarySyncStatus, ServerInfo } from "@/types";

const RUNNING_PHASES = new Set<LibrarySyncStatus["phase"]>(["metadata", "activating", "artwork"]);

const errorText = (cause: unknown, fallback: string) => {
  if (typeof cause === "string") {
    return cause;
  }

  return cause instanceof Error ? cause.message : fallback;
};

export function useLibrarySync(profileId: Readonly<Ref<string | null | undefined>>) {
  const status = ref<LibrarySyncStatus | null>(null);
  const actionError = ref<string | null>(null);
  const statusError = ref<string | null>(null);
  const isConnected = ref(false);
  const isRunning = computed(() => RUNNING_PHASES.has(status.value?.phase ?? "idle"));
  const canSync = computed(() => isConnected.value && !isRunning.value);

  let latestStatusRequest = 0;
  let profileGeneration = 0;

  const ping = async () => {
    try {
      await invoke<ServerInfo>("ping_music_server");
      return true;
    } catch {
      return false;
    }
  };

  const refreshStatus = async () => {
    const requestId = ++latestStatusRequest;

    try {
      const nextStatus = await invoke<LibrarySyncStatus>("get_library_sync_status");
      if (requestId === latestStatusRequest) {
        status.value = nextStatus;
        statusError.value = null;
      }
    } catch (cause) {
      if (requestId === latestStatusRequest) {
        status.value = null;
        statusError.value = errorText(cause, "Failed to read sync status.");
      }
    }
  };

  const { pause, resume } = useIntervalFn(() => void refreshStatus(), 500, { immediate: false });

  const start = async () => {
    isConnected.value = await ping();
    if (!isConnected.value) {
      actionError.value = "No music server is connected.";
      return;
    }

    actionError.value = null;
    try {
      await invoke("sync_library", { force: true });
      resume();
    } catch (cause) {
      actionError.value = errorText(cause, "Failed to start library sync.");
    }
    await refreshStatus();
  };

  watch(
    profileId,
    async () => {
      const generation = ++profileGeneration;
      pause();
      latestStatusRequest += 1;
      status.value = null;
      actionError.value = null;
      statusError.value = null;

      const connected = await ping();
      if (generation !== profileGeneration) {
        return;
      }

      isConnected.value = connected;
      if (connected) {
        await refreshStatus();
        if (generation === profileGeneration) {
          resume();
        }
      }
    },
    { immediate: true },
  );

  return {
    status,
    error: computed(() => actionError.value ?? statusError.value),
    isConnected,
    isRunning,
    canSync,
    start,
  };
}
