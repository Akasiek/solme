<script setup lang="ts">
import { useTimeoutFn, useWindowFocus } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { computed, onMounted, ref, watch } from "vue";
import LyricsDisplay from "@/components/Lyrics/LyricsDisplay.vue";
import { usePlayerStore } from "@/stores/player.ts";
import { useSettingsStore } from "@/stores/settings";
import { useRouter } from "vue-router";

const AWAY_DELAY_MS = 3_000;

const router = useRouter();
const playerStore = usePlayerStore();
const settingsStore = useSettingsStore();
const { lyrics, isLyricsLoading, playbackPositionSeconds } = storeToRefs(playerStore);
const { settings, isLoaded: areSettingsLoaded } = storeToRefs(settingsStore);
const syncedLyrics = computed(() => lyrics.value.find((lyricsVariant) => lyricsVariant.synced) ?? null);
const previousSyncedLyrics = ref(syncedLyrics.value);
const modalLyrics = computed(() => syncedLyrics.value ?? (isLyricsLoading.value ? previousSyncedLyrics.value : null));

const isWindowFocused = useWindowFocus();
const isInLyricsView = computed(() => router.currentRoute.value.name === "lyrics");
const isAwayLyricsEnabled = computed(() => areSettingsLoaded.value && settings.value.awayLyricsEnabled);
const show = ref(false);
const visibleModalLyrics = computed(() =>
  show.value && isAwayLyricsEnabled.value && !isInLyricsView.value ? modalLyrics.value : null,
);

const { start: startAwayTimer, stop: stopAwayTimer } = useTimeoutFn(
  () => {
    show.value = !isWindowFocused.value && isAwayLyricsEnabled.value && modalLyrics.value !== null;
  },
  AWAY_DELAY_MS,
  { immediate: false },
);

watch(syncedLyrics, (lyricsVariant) => {
  if (lyricsVariant) {
    previousSyncedLyrics.value = lyricsVariant;
  }
});

watch(
  [isWindowFocused, isAwayLyricsEnabled],
  ([focused, enabled]) => {
    stopAwayTimer();
    show.value = false;

    if (!focused && enabled && modalLyrics.value) {
      startAwayTimer();
    }
  },
  { immediate: true },
);

watch(modalLyrics, (lyricsVariant) => {
  if (!lyricsVariant) {
    stopAwayTimer();
    show.value = false;
  } else if (isAwayLyricsEnabled.value && !isWindowFocused.value && !show.value) {
    startAwayTimer();
  }
});

onMounted(() => {
  void settingsStore.load();
});
</script>

<template>
  <Transition
    enter-active-class="transition-opacity duration-300 ease-out"
    enter-from-class="opacity-0"
    leave-active-class="transition-opacity duration-150 ease-in"
    leave-to-class="opacity-0"
  >
    <section
      v-if="visibleModalLyrics"
      role="dialog"
      aria-modal="true"
      aria-label="Lyrics"
      class="lyrics-away-modal absolute inset-0 z-50 overflow-y-auto bg-zinc-950/95 p-8 font-serif text-zinc-100 backdrop-blur-sm"
    >
      <LyricsDisplay
        :lyrics="visibleModalLyrics"
        :playback-position-seconds="playbackPositionSeconds"
        :allow-seek="false"
      />
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
