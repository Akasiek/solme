<script setup lang="ts">
import { CachedAlbum } from "@/types.ts";
import { artworkSource } from "@/utils/artwork.ts";
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";

defineProps<{
  album: CachedAlbum;
}>();
</script>

<template>
  <div class="size-80 flex-none basis-80 overflow-hidden rounded-lg shadow-2xl shadow-black/40">
    <img
      v-if="album.artworkPath"
      :src="artworkSource(album.artworkPath)"
      :alt="`${album.name} artwork`"
      class="h-full w-full object-cover object-center"
    />
    <MissingCoverImage v-else class="h-full w-full" />
  </div>
</template>
