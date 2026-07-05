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
    class="aspect-square w-full max-w-80 shrink-0 overflow-hidden rounded-lg border-2 border-zinc-800 lg:w-80 xl:w-96 xl:max-w-96"
  >
    <img
      v-if="album.artworkPath"
      :src="artworkSource(album.artworkPath)"
      :alt="`${album.name} artwork`"
      class="h-full w-full object-cover object-center"
    />
    <MissingCoverImage v-else />
  </div>
</template>
