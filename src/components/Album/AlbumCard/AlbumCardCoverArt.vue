<script setup lang="ts">
import { ListStart, Play, ListEnd } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";

import FavoriteButton from "@/components/LibraryItemAnnotations/FavoriteButton.vue";
import { useLayoutStore } from "@/stores/layout.ts";
import type { CachedAlbum } from "@/types.js";
import { artworkSource } from "@/utils/artwork.js";
import MissingCoverImage from "../MissingCoverImage.vue";

const { album } = defineProps<{ album: CachedAlbum }>();
const { closeSearchModal } = useLayoutStore();

const playAlbum = () => {
  void invoke("player_play_album", { albumId: album.remoteId });
  closeSearchModal();
};

const queueAlbumNext = () => {
  void invoke("player_queue_album_next", { albumId: album.remoteId });
};

const queueAlbumLast = () => {
  void invoke("player_queue_album_last", { albumId: album.remoteId });
};
</script>

<template>
  <div
    class="album-card-cover group/image relative aspect-square w-full overflow-hidden rounded-md border border-zinc-800 bg-zinc-900 transition-colors duration-300 group-hover:border-zinc-600"
  >
    <img
      v-if="album.artworkPath"
      :src="artworkSource(album.artworkPath)"
      :alt="`${album.name} artwork`"
      class="h-full w-full object-cover object-center"
    />
    <MissingCoverImage v-else />

    <RouterLink
      class="absolute inset-0 focus-visible:ring-2 focus-visible:ring-white focus-visible:outline-none focus-visible:ring-inset"
      :to="{ name: 'album', params: { albumId: album.remoteId } }"
      :aria-label="`Open ${album.name}`"
    />

    <div
      class="focus-reveal pointer-events-none absolute inset-0 bg-linear-to-t from-black/85 via-black/25 to-transparent opacity-0 transition-opacity duration-200 group-hover/image:opacity-100 [@media(hover:none)]:opacity-100"
    />

    <div
      class="focus-reveal pointer-events-none absolute inset-0 flex flex-col justify-between p-2 opacity-0 transition-opacity duration-200 group-hover/image:opacity-100 [@media(hover:none)]:opacity-100"
    >
      <div
        class="focus-actions pointer-events-none self-end group-hover/image:pointer-events-auto [@media(hover:none)]:pointer-events-auto"
        @click.stop
      >
        <FavoriteButton :item="album" variant="overlay" />
      </div>

      <div
        class="focus-actions pointer-events-none flex items-center justify-center gap-2 group-hover/image:pointer-events-auto [@media(hover:none)]:pointer-events-auto"
        @click.stop
      >
        <button
          class="cover-art-button size-9 border border-white/30 bg-zinc-950/80 text-white hover:border-white/60 hover:bg-zinc-800"
          type="button"
          :title="`Add ${album.name} to queue next`"
          :aria-label="`Add ${album.name} to queue next`"
          @click="queueAlbumNext"
        >
          <ListStart class="size-4" aria-hidden="true" />
        </button>
        <button
          class="cover-art-button size-11 border border-accent bg-accent text-white shadow-lg shadow-black/30 hover:bg-accent/85"
          type="button"
          :title="`Play ${album.name} now`"
          :aria-label="`Play ${album.name} now`"
          @click="playAlbum"
        >
          <Play class="size-5 fill-current" aria-hidden="true" />
        </button>
        <button
          class="cover-art-button size-9 border border-white/30 bg-zinc-950/80 text-white hover:border-white/60 hover:bg-zinc-800"
          type="button"
          :title="`Add ${album.name} to the end of the queue`"
          :aria-label="`Add ${album.name} to the end of the queue`"
          @click="queueAlbumLast"
        >
          <ListEnd class="size-4" aria-hidden="true" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
@reference "@/style/glob.css";

.album-card-cover:has(:focus-visible) .focus-reveal {
  opacity: 1;
}

.album-card-cover:has(:focus-visible) .focus-actions {
  pointer-events: auto;
}

.cover-art-button {
  @apply grid cursor-pointer place-items-center rounded-full transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-white focus-visible:outline-none active:scale-95;
}
</style>
