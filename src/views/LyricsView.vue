<script setup lang="ts">
import AsyncViewState from "@/components/AsyncViewState.vue";
import { usePlayerStore } from "@/stores/player.ts";
import { storeToRefs } from "pinia";
import { onMounted } from "vue";
import LyricsDisplay from "@/components/Lyrics/LyricsDisplay.vue";

const playerStore = usePlayerStore();
const { currentSong, hasLyrics, isLyricsLoading, lyrics, lyricsError, playbackPositionSeconds } =
  storeToRefs(playerStore);

onMounted(() => {
  void playerStore.startListening();
});
</script>

<template>
  <AsyncViewState :is-loading="isLyricsLoading" :error="lyricsError">
    <section class="p-8">
      <p v-if="!currentSong">No song is currently playing.</p>
      <p v-else-if="!hasLyrics">No lyrics available.</p>
      <div v-else>
        <LyricsDisplay :lyrics="lyrics[0]" :playback-position-seconds="playbackPositionSeconds" />
      </div>
    </section>
  </AsyncViewState>
</template>
