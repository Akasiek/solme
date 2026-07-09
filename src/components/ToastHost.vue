<script setup lang="ts">
import { X } from "@lucide/vue";

import { useToast } from "@/composables/useToast";

const { toasts, dismissToast } = useToast();
</script>

<template>
  <div class="pointer-events-none fixed top-4 right-4 z-50 flex w-[min(24rem,calc(100vw-2rem))] flex-col gap-2">
    <div
      v-for="toast in toasts"
      :key="toast.id"
      class="pointer-events-auto flex items-start gap-3 rounded border border-zinc-700 bg-zinc-900 px-4 py-3 text-sm text-zinc-100 shadow-lg shadow-accent/5"
      role="status"
    >
      <p class="min-w-0 flex-1">{{ toast.message }}</p>
      <button
        type="button"
        class="grid size-6 shrink-0 place-items-center rounded text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
        title="Dismiss"
        @click="dismissToast(toast.id)"
      >
        <X class="size-4" />
      </button>
    </div>
  </div>
</template>
