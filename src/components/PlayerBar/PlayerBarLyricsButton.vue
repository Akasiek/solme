<script setup lang="ts">
import { MicVocal } from "@lucide/vue";
import { computed, onMounted } from "vue";
import { RouterLink } from "vue-router";

import { usePlayerStore } from "@/stores/player.ts";

const playerStore = usePlayerStore();
const unavailableTitle = computed(() => {
  if (playerStore.isLyricsLoading) {
    return "Loading lyrics";
  }

  return playerStore.lyricsError ?? "Lyrics unavailable";
});

onMounted(() => {
  void playerStore.ensureCurrentLyrics();
});
</script>

<template>
  <component
    :is="playerStore.hasLyrics ? RouterLink : 'button'"
    v-bind="
      playerStore.hasLyrics
        ? { to: { name: 'lyrics' }, title: 'Show lyrics', 'aria-label': 'Show lyrics' }
        : { type: 'button', disabled: true, title: unavailableTitle, 'aria-label': unavailableTitle }
    "
    class="flex items-center rounded p-2"
    :class="
      playerStore.hasLyrics ? 'text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100' : 'cursor-not-allowed text-zinc-600'
    "
  >
    <MicVocal class="size-5" />
  </component>
</template>
