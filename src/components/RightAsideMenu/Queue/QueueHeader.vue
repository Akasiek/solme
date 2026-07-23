<script setup lang="ts">
import { ListMusic } from "@lucide/vue";

defineProps<{
  songCount: number;
}>();
</script>

<template>
  <header class="flex h-16 shrink-0 items-center justify-between border-b border-zinc-800 px-5">
    <div class="flex items-center gap-2">
      <ListMusic class="size-5 text-zinc-400" aria-hidden="true" />
      <h2 class="font-serif text-lg font-semibold text-zinc-100">Queue</h2>
    </div>
    <span class="font-sans text-sm text-zinc-500 tabular-nums">{{ songCount }}</span>
  </header>
</template>
