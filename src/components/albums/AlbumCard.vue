<script setup lang="ts">
import { CachedAlbum } from "@/types.js";
import AlbumCardImageSection from "@/components/albums/AlbumCardImageSection.vue";

defineProps<{
  album: CachedAlbum;
}>();
</script>

<template>
  <RouterLink
    class="group grid gap-4 transition-colors duration-300 ease-in-out"
    :to="{ name: 'album', params: { albumId: album.remoteId } }"
  >
    <AlbumCardImageSection :album="album" />

    <div
      class="space-y-1 gap-x-2.5 rounded-md border-2 border-zinc-800 px-3 py-3 transition-colors duration-300 ease-in-out group-hover:border-zinc-600"
    >
      <h4 class="line-clamp-1 font-serif font-bold" :title="album.name">{{ album.name }}</h4>
      <RouterLink :to="{ name: 'artist', params: { artistId: album.artistId } }">
        <h3 class="line-clamp-1 font-sans font-normal text-zinc-300 hover:underline" :title="album.artistName">
          {{ album.artistName }}
        </h3>
      </RouterLink>
    </div>
  </RouterLink>
</template>
