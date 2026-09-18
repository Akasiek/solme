<script setup lang="ts">
import { useTimeoutFn, useWindowFocus } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import LyricsDisplay from "@/components/Lyrics/LyricsDisplay.vue";
import { usePlayerStore } from "@/stores/player.ts";
import { useRouter } from "vue-router";

const AWAY_DELAY_MS = 3_000;

const router = useRouter();
const playerStore = usePlayerStore();
const { lyrics, playbackPositionSeconds } = storeToRefs(playerStore);
const syncedLyrics = computed(() => lyrics.value.find((lyricsVariant) => lyricsVariant.synced) ?? null);

const isWindowFocused = useWindowFocus();
const isInLyricsView = computed(() => router.currentRoute.value.name === "lyrics");
const show = ref(false);

const { start: startAwayTimer, stop: stopAwayTimer } = useTimeoutFn(
  () => {
    show.value = !isWindowFocused.value && syncedLyrics.value !== null;
  },
  AWAY_DELAY_MS,
  { immediate: false },
);

watch(
  [isWindowFocused, syncedLyrics],
  ([focused, lyricsVariant]) => {
    stopAwayTimer();
    show.value = false;

    if (!focused && lyricsVariant) {
      startAwayTimer();
    }
  },
  { immediate: true },
);
</script>

<template>
  <Transition
    enter-active-class="transition-opacity duration-300 ease-out"
    enter-from-class="opacity-0"
    leave-active-class="transition-opacity duration-150 ease-in"
    leave-to-class="opacity-0"
  >
    <section
      v-if="show && syncedLyrics && !isInLyricsView"
      role="dialog"
      aria-modal="true"
      aria-label="Lyrics"
      class="lyrics-away-modal absolute inset-0 z-50 overflow-y-auto bg-zinc-950/95 p-8 font-serif text-zinc-100 backdrop-blur-sm"
    >
      <LyricsDisplay :lyrics="syncedLyrics" :playback-position-seconds="playbackPositionSeconds" :allow-seek="false" />
    </section>
  </Transition>
</template>

<style scoped>
.lyrics-away-modal {
  scrollbar-width: none;
}

.lyrics-away-modal::-webkit-scrollbar {
  display: none;
}
</style>
