<script setup lang="ts">
import NormalLyricsDisplay from "@/components/Lyrics/NormalLyricsDisplay.vue";
import SyncedLyricsDisplay from "@/components/Lyrics/SyncedLyricsDisplay.vue";
import type { SongLyrics } from "@/types.ts";

defineProps<{
  lyrics: SongLyrics;
  playbackPositionSeconds: number;
  allowSeek?: boolean;
}>();
</script>

<template>
  <SyncedLyricsDisplay
    v-if="lyrics.synced"
    :lyrics="lyrics"
    :playback-position-seconds="playbackPositionSeconds"
    :allow-seek="allowSeek"
  />
  <NormalLyricsDisplay v-else :lyrics="lyrics" />
</template>
