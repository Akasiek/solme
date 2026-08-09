<script setup lang="ts">
import { computed } from "vue";
import { CachedAlbum } from "@/types.js";
import AlbumCardCoverArt from "./AlbumCardCoverArt.vue";

const { album, showReleaseYear = false } = defineProps<{
  album: CachedAlbum;
  showReleaseYear?: boolean;
}>();

const releaseYear = computed(
  () => album.year ?? album.originalReleaseDate?.slice(0, 4) ?? album.releaseDate?.slice(0, 4),
);
</script>

<template>
  <RouterLink
    class="group grid gap-2 transition-colors duration-300 ease-in-out"
    :to="{ name: 'album', params: { albumId: album.remoteId } }"
  >
    <AlbumCardCoverArt :album="album" />

    <div
      class="space-y-0.5 gap-x-2.5 rounded-md border border-zinc-800 px-2 py-2.5 transition-colors duration-300 ease-in-out group-hover:border-zinc-600"
    >
      <h3 class="line-clamp-1 font-serif text-md font-bold hover:underline" :title="album.name">{{ album.name }}</h3>
      <p v-if="showReleaseYear" class="line-clamp-1 font-sans text-sm font-normal text-zinc-300">
        {{ releaseYear }}
      </p>
      <RouterLink v-else :to="{ name: 'artist', params: { artistId: album.artistId } }">
        <h4 class="line-clamp-1 font-sans text-sm font-normal text-zinc-300 hover:underline" :title="album.artistName">
          {{ album.artistName }}
        </h4>
      </RouterLink>
    </div>
  </RouterLink>
</template>
