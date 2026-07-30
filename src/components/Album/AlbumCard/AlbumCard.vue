<script setup lang="ts">
import { computed } from "vue";
import { CachedAlbum } from "@/types.js";
import AlbumCardCoverArt from "./AlbumCardCoverArt.vue";

const { album, showReleaseYear } = withDefaults(
  defineProps<{
    album: CachedAlbum;
    showReleaseYear?: boolean;
  }>(),
  {
    showReleaseYear: false,
  },
);

const releaseYear = computed(
  () => album.year ?? album.originalReleaseDate?.slice(0, 4) ?? album.releaseDate?.slice(0, 4),
);
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
      <p v-if="showReleaseYear" class="line-clamp-1 font-sans font-normal text-zinc-300">
        {{ releaseYear }}
      </p>
      <RouterLink v-else :to="{ name: 'artist', params: { artistId: album.artistId } }">
        <h4 class="line-clamp-1 font-sans font-normal text-zinc-300 hover:underline" :title="album.artistName">
          {{ album.artistName }}
        </h4>
      </RouterLink>
    </div>
  </RouterLink>
</template>
