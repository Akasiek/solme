<script setup lang="ts">
import LyricsDisplayLayout from "@/components/Lyrics/LyricsDisplayLayout.vue";
import type { SongLyrics } from "@/types.ts";

defineProps<{ lyrics: SongLyrics }>();
</script>

<template>
  <LyricsDisplayLayout class="pb-8">
    <p v-for="[index, line] in lyrics.line.entries()" :key="index" class="lyrics-line whitespace-pre-line">
      <template v-if="line.value !== ''">
        {{ line.value }}
      </template>
      <span v-else>...</span>
    </p>
  </LyricsDisplayLayout>
</template>
