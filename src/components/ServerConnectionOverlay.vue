<script setup lang="ts">
import { computed } from "vue";

import { useServerConnectionStore } from "@/stores/serverConnection";

const { status: serverConnectionStatus } = useServerConnectionStore();

const isVisible = computed(() => serverConnectionStatus.phase !== "idle");

const title = computed(() => {
  switch (serverConnectionStatus.phase) {
    case "primary":
      return "Connecting to primary server";
    case "secondary":
      return "Primary server did not respond";
    case "failed":
      return "Could not connect to music server";
    default:
      return "";
  }
});

const description = computed(() => {
  switch (serverConnectionStatus.phase) {
    case "primary":
      return serverConnectionStatus.primaryUrl
        ? `Trying ${serverConnectionStatus.primaryUrl}`
        : "Trying the saved primary server.";
    case "secondary":
      return serverConnectionStatus.secondaryUrl
        ? `Trying fallback ${serverConnectionStatus.secondaryUrl}`
        : "Trying the saved fallback server.";
    case "failed":
      return serverConnectionStatus.error ?? "Check the saved server settings.";
    default:
      return "";
  }
});
</script>

<template>
  <Transition
    enter-active-class="transition duration-150 ease-out"
    enter-from-class="opacity-0"
    enter-to-class="opacity-100"
    leave-active-class="transition duration-300 ease-in"
    leave-from-class="opacity-100"
    leave-to-class="opacity-0"
  >
    <div v-if="isVisible" class="fixed inset-0 z-50 grid place-items-center bg-zinc-900/95 p-6 text-zinc-100">
      <div class="w-full max-w-md space-y-4 text-center transition duration-300 ease-in-out">
        <div
          v-if="serverConnectionStatus.phase !== 'failed'"
          class="mx-auto size-10 animate-spin rounded-full border-2 border-zinc-700 border-t-zinc-100"
        />
        <div v-else class="mx-auto grid size-10 place-items-center rounded-full bg-red-500/15 text-xl text-red-200">
          !
        </div>

        <div class="space-y-2">
          <h1 class="font-serif text-2xl font-bold">{{ title }}</h1>
          <p class="text-sm wrap-break-word text-zinc-400">{{ description }}</p>
        </div>

        <div v-if="serverConnectionStatus.secondaryUrl" class="space-y-2 text-left text-xs text-zinc-500">
          <div class="flex items-center justify-between gap-3">
            <span>Primary</span>
            <span>{{ serverConnectionStatus.phase === "primary" ? "Connecting" : "Failed" }}</span>
          </div>
          <div class="flex items-center justify-between gap-3">
            <span>Secondary</span>
            <span>{{ serverConnectionStatus.phase === "secondary" ? "Connecting" : "Waiting" }}</span>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>
