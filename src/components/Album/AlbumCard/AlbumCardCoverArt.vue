<script setup lang="ts">
import { ListStart, Play, ListEnd } from "@lucide/vue";
import { CachedAlbum } from "@/types.js";
import { artworkSource } from "@/utils/artwork.js";
import MissingCoverImage from "../MissingCoverImage.vue";
import { invoke } from "@tauri-apps/api/core";

const { album } = defineProps<{
  album: CachedAlbum;
}>();

const stopAlbumLinkNavigation = (event: MouseEvent) => {
  event.preventDefault();
  event.stopPropagation();
};

const playAlbum = (event: MouseEvent) => {
  stopAlbumLinkNavigation(event);
  invoke("player_play_album", { albumId: album.remoteId });
};

const queueAlbumNext = (event: MouseEvent) => {
  stopAlbumLinkNavigation(event);
  invoke("player_queue_album_next", { albumId: album.remoteId });
};

const queueAlbumLast = (event: MouseEvent) => {
  stopAlbumLinkNavigation(event);
  invoke("player_queue_album_last", { albumId: album.remoteId });
};
</script>

<template>
  <div
    class="group/image relative aspect-square w-full overflow-hidden rounded border border-zinc-800 transition-colors duration-300 ease-in-out group-hover:border-zinc-600"
  >
    <img
      v-if="album.artworkPath"
      :src="artworkSource(album.artworkPath)"
      :alt="`${album.name} artwork`"
      class="h-full w-full object-cover object-center"
    />
    <MissingCoverImage v-else />

    <div
      class="absolute inset-0 flex items-center justify-center gap-4 bg-zinc-800/80 text-white opacity-0 transition-opacity duration-300 group-hover/image:opacity-100"
    >
      <button
        class="cover-art-button"
        type="button"
        :title="`Play ${album.name} next`"
        :aria-label="`Play ${album.name} next`"
        @click="queueAlbumNext"
      >
        <ListStart aria-hidden="true" />
      </button>
      <button
        class="cover-art-button"
        type="button"
        :title="`Add ${album.name} to queue next`"
        :aria-label="`Play ${album.name} now`"
        @click="playAlbum"
      >
        <Play aria-hidden="true" class="fill-white" />
      </button>
      <button
        class="cover-art-button"
        type="button"
        :title="`Add ${album.name} to the end of the queue`"
        :aria-label="`Add ${album.name} to the end of the queue`"
        @click="queueAlbumLast"
      >
        <ListEnd aria-hidden="true" />
      </button>
    </div>
  </div>
</template>

<style scoped>
@reference "@/style/glob.css";

.cover-art-button {
  @apply cursor-pointer rounded-full bg-accent p-2 text-zinc-100 transition-colors duration-300 ease-in-out hover:bg-accent/80 active:scale-95;
}
</style>
