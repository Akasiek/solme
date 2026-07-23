<script setup lang="ts">
import { ListEnd, ListStart, Play } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { CachedAlbum } from "@/types.ts";
import Button from "@/components/Button.vue";

const { album } = defineProps<{
  album: CachedAlbum;
}>();

const playAlbum = () => {
  invoke("player_play_album", { albumId: album.remoteId });
};

const queueAlbumNext = () => {
  invoke("player_queue_album_next", { albumId: album.remoteId });
};

const queueAlbumLast = () => {
  invoke("player_queue_album_last", { albumId: album.remoteId });
};
</script>

<template>
  <div class="ml-0.5 flex flex-wrap gap-2">
    <Button type="button" @click="playAlbum">
      <Play class="size-4" aria-hidden="true" />
      Play
    </Button>
    <Button type="button" variant="outline" @click="queueAlbumNext">
      <ListStart class="size-4" aria-hidden="true" />
      Queue next
    </Button>
    <Button type="button" variant="outline" @click="queueAlbumLast">
      <ListEnd class="size-4" aria-hidden="true" />
      Queue last
    </Button>
  </div>
</template>
