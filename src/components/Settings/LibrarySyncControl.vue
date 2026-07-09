<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw } from "@lucide/vue";
import { useIntervalFn } from "@vueuse/core";

import { LibrarySyncStatus, ServerInfo } from "@/types";

const props = defineProps<{
  profileId?: string | null;
}>();

const syncStatus = ref<LibrarySyncStatus | null>(null);
const syncError = ref<string | null>(null);
const isConnected = ref(false);

const isRunning = computed(() => {
  return ["metadata", "activating", "artwork"].includes(syncStatus.value?.phase ?? "");
});
const canSync = computed(() => isConnected.value && !isRunning.value);
const label = computed(() => {
  if (!isConnected.value) {
    return "No server connected";
  }
  if (syncError.value) {
    return "Sync failed";
  }
  if (!syncStatus.value) {
    return "Sync status";
  }

  return `Sync: ${syncStatus.value.phase}`;
});

const { pause, resume } = useIntervalFn(
  () => {
    void refreshSyncStatus();
  },
  500,
  { immediate: false },
);

const refreshConnection = async () => {
  try {
    await invoke<ServerInfo>("ping_music_server");
    isConnected.value = true;
    return true;
  } catch {
    isConnected.value = false;
    return false;
  }
};

const refreshSyncStatus = async () => {
  try {
    const nextStatus = await invoke<LibrarySyncStatus>("get_library_sync_status");
    syncStatus.value = nextStatus;
    syncError.value = nextStatus.lastError ?? null;

    if (["failed", "idle", "completed"].includes(nextStatus.phase)) {
      pause();
    }
  } catch (cause) {
    syncStatus.value = null;
    syncError.value = cause instanceof Error ? cause.message : "Failed to read sync status.";
    pause();
  }
};

const startSync = async () => {
  if (!(await refreshConnection())) {
    syncError.value = "No music server is connected.";
    return;
  }

  syncError.value = null;

  try {
    await invoke("sync_library", { force: true });
    await refreshSyncStatus();
    resume();
  } catch (cause) {
    syncError.value = cause instanceof Error ? cause.message : "Failed to start library sync.";
    await refreshSyncStatus();
  }
};

watch(
  () => props.profileId,
  async () => {
    pause();
    syncStatus.value = null;
    syncError.value = null;

    if (await refreshConnection()) {
      await refreshSyncStatus();
    } else {
      pause();
    }
  },
);

onMounted(async () => {
  if (await refreshConnection()) {
    await refreshSyncStatus();
  }
});
</script>

<template>
  <button
    type="button"
    class="flex min-h-9 items-center gap-2 rounded border border-zinc-700 px-3 py-2 text-sm text-zinc-100 hover:bg-zinc-800 disabled:cursor-not-allowed disabled:opacity-60"
    :class="{ 'border-red-500/60': syncError }"
    :disabled="!canSync"
    :title="syncError ?? 'Synchronize library'"
    @click="startSync"
  >
    <RefreshCw class="size-4" :class="{ 'animate-spin': isRunning }" />
    <span class="capitalize">{{ label }}</span>
  </button>
</template>
