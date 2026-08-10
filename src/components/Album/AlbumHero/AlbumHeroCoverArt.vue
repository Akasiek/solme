<script setup lang="ts">
import { CachedAlbum } from "@/types.ts";
import { artworkSource } from "@/utils/artwork.ts";
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";

defineProps<{
  album: CachedAlbum;
}>();
</script>

<template>
  <div
    class="mx-auto aspect-square w-full max-w-64 overflow-hidden rounded-lg shadow-2xl shadow-black/40 @min-[36rem]:max-w-80 @min-[48rem]:max-w-none"
  >
    <img
      v-if="album.artworkPath"
      :src="artworkSource(album.artworkPath)"
      :alt="`${album.name} artwork`"
      class="h-full w-full object-cover object-center"
    />
    <MissingCoverImage v-else class="h-full w-full" />
  </div>
</template>
