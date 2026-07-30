<script setup lang="ts">
import { computed, toRef } from "vue";
import { RefreshCw } from "@lucide/vue";

import { useLibrarySync } from "@/composables/useLibrarySync";
import type { LibrarySyncStatus } from "@/types";

const props = defineProps<{
  profileId?: string | null;
}>();

const {
  status: syncStatus,
  error: syncRequestError,
  isConnected,
  isRunning,
  canSync,
  start: startSync,
} = useLibrarySync(toRef(props, "profileId"));

const syncWarning = computed(() => (syncStatus.value?.phase === "completed" ? syncStatus.value.lastError : null));
const syncError = computed(
  () => syncRequestError.value ?? (syncStatus.value?.phase === "failed" ? syncStatus.value.lastError : null),
);

const feedback = computed(() => {
  if (syncWarning.value) {
    const message = syncWarning.value.toLowerCase().includes("artwork")
      ? "Your library is synchronized, but some covers or artist images could not be downloaded."
      : "Your library is synchronized, but some optional data could not be updated.";
    return { message, details: syncWarning.value, role: "status" as const, tone: "warning" as const };
  }
  if (syncError.value) {
    return {
      message: "The library could not be fully synchronized. You can try again in a moment.",
      details: syncError.value,
      role: "alert" as const,
      tone: "error" as const,
    };
  }
  return null;
});

const PHASE_LABELS: Record<LibrarySyncStatus["phase"], string> = {
  idle: "Sync library",
  metadata: "Syncing metadata",
  activating: "Saving library",
  artwork: "Syncing artwork",
  completed: "Library synchronized",
  failed: "Sync failed",
};

const label = computed(() => {
  if (!isConnected.value) {
    return "No server connected";
  }
  if (syncError.value) {
    return "Sync failed";
  }
  if (!syncStatus.value) {
    return "Checking sync status";
  }
  if (syncWarning.value) {
    return "Sync completed with warnings";
  }

  return PHASE_LABELS[syncStatus.value.phase];
});
</script>

<template>
  <div class="flex max-w-md flex-col items-start gap-2 md:items-end">
    <button
      type="button"
      class="flex min-h-9 items-center gap-2 rounded border border-zinc-700 px-3 py-2 text-sm text-zinc-100 hover:bg-zinc-800 disabled:cursor-not-allowed disabled:opacity-60"
      :class="{
        'border-red-500/60': syncError,
        'border-amber-500/60': syncWarning && !syncError,
      }"
      :disabled="!canSync"
      :title="syncError ?? syncWarning ?? 'Synchronize library'"
      @click="startSync"
    >
      <RefreshCw class="size-4" :class="{ 'animate-spin': isRunning }" />
      <span class="capitalize">{{ label }}</span>
    </button>
  </div>

  <div
    v-if="feedback"
    :role="feedback.role"
    class="w-full rounded border px-3 py-2 text-left font-sans text-xs"
    :class="
      feedback.tone === 'warning'
        ? 'border-amber-500/30 bg-amber-500/10 text-amber-100'
        : 'border-red-500/30 bg-red-500/10 text-red-100'
    "
  >
    <p>{{ feedback.message }}</p>
    <details class="mt-1 opacity-80">
      <summary class="cursor-pointer select-none hover:opacity-100">Show details</summary>
      <p class="mt-1 break-words">{{ feedback.details }}</p>
    </details>
  </div>
</template>
