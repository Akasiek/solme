<script setup lang="ts">
import { ListStart, Play, ListEnd } from "@lucide/vue";
import { CachedAlbum } from "@/types.js";
import { artworkSource } from "@/utils/artwork.js";
import MissingCoverImage from "./MissingCoverImage.vue";
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

const queueAlbumAtStart = (event: MouseEvent) => {
  stopAlbumLinkNavigation(event);
  invoke("player_queue_album_at_start", { albumId: album.remoteId });
};

const queueAlbumAtEnd = (event: MouseEvent) => {
  stopAlbumLinkNavigation(event);
  invoke("player_queue_album_at_end", { albumId: album.remoteId });
};
</script>

<template>
  <div
    class="group/image relative overflow-hidden rounded border-2 border-zinc-800 transition-colors duration-300 ease-in-out group-hover:border-zinc-600"
  >
    <img
      v-if="album.artworkPath"
      :src="artworkSource(album.artworkPath)"
      :alt="`${album.name} artwork`"
      class="h-full w-full object-cover object-center"
    />
    <MissingCoverImage v-else />

    <div
      class="absolute inset-0 flex items-center justify-center gap-4 bg-zinc-800/80 text-white opacity-0 transition-opacity duration-300 *:cursor-pointer *:rounded-full *:bg-accent *:p-2 group-hover/image:opacity-100"
    >
      <button type="button" @click="queueAlbumAtStart">
        <ListStart />
      </button>
      <button type="button" @click="playAlbum">
        <Play />
      </button>
      <button type="button" @click="queueAlbumAtEnd">
        <ListEnd />
      </button>
    </div>
  </div>
</template>
