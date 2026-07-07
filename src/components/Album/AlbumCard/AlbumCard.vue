<script setup lang="ts">
import { CachedAlbum } from "@/types.js";
import AlbumCardCoverArt from "./AlbumCardCoverArt.vue";

defineProps<{
  album: CachedAlbum;
}>();
</script>

<template>
  <RouterLink
    class="group grid gap-4 transition-colors duration-300 ease-in-out"
    :to="{ name: 'album', params: { albumId: album.remoteId } }"
  >
    <AlbumCardCoverArt :album="album" />

    <div
      class="space-y-1 gap-x-2.5 rounded-md border border-zinc-800 px-2.5 py-3 transition-colors duration-300 ease-in-out group-hover:border-zinc-600"
    >
      <h3 class="line-clamp-1 font-serif font-bold hover:underline" :title="album.name">{{ album.name }}</h3>
      <RouterLink :to="{ name: 'artist', params: { artistId: album.artistId } }">
        <h4 class="line-clamp-1 font-sans font-normal text-zinc-300 hover:underline" :title="album.artistName">
          {{ album.artistName }}
        </h4>
      </RouterLink>
    </div>
  </RouterLink>
</template>
