<script setup lang="ts">
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";
import type { CachedArtist } from "@/types";
import { artworkSource } from "@/utils/artwork";
import { getDominantImageColor } from "@/utils/imageColor";
import { ref } from "vue";

defineProps<{
  artist: CachedArtist;
}>();

const dominantColor = ref<string>();
const handleArtworkLoad = (event: Event) => {
  try {
    dominantColor.value = getDominantImageColor(event.target as HTMLImageElement);
  } catch {
    dominantColor.value = undefined;
  }
};
</script>

<template>
  <div class="relative z-10 size-60 shrink-0 overflow-visible md:size-72 lg:size-80 xl:size-88">
    <img
      v-if="artist.artworkPath"
      :key="`glow-${artist.artworkPath}`"
      :src="artworkSource(artist.artworkPath)"
      alt=""
      class="pointer-events-none absolute -inset-6 h-[calc(100%+3rem)] w-[calc(100%+3rem)] rounded-full object-cover object-center opacity-40 blur-2xl"
      aria-hidden="true"
    />
    <div
      class="relative h-full w-full overflow-hidden rounded-full border-2 border-zinc-800 bg-zinc-900 shadow-2xl shadow-black/40"
    >
      <img
        v-if="artist.artworkPath"
        :key="`artwork-${artist.artworkPath}`"
        :src="artworkSource(artist.artworkPath)"
        :alt="`${artist.name} artwork`"
        class="h-full w-full object-cover object-center"
        @load="handleArtworkLoad"
      />
      <MissingCoverImage v-else />
    </div>
  </div>
</template>
