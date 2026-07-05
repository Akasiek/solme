<script setup lang="ts">
import { useAsyncData } from "@/composables/useAsyncData.ts";
import { invoke } from "@tauri-apps/api/core";
import { watch } from "vue";
import { CachedAlbumDetails } from "@/types.ts";
import AsyncViewState from "@/components/AsyncViewState.vue";
import AlbumHero from "@/components/Album/AlbumHero/AlbumHero.vue";

const props = defineProps<{ albumId: string }>();

const {
  data: albumDetails,
  isLoading,
  error: loadError,
  reload,
} = useAsyncData(
  () =>
    invoke<CachedAlbumDetails>("get_cached_album", {
      albumId: props.albumId,
    }),
  null,
);

watch(
  () => props.albumId,
  () => {
    void reload();
  },
);
</script>

<template>
  <AsyncViewState :is-loading="isLoading" :error="loadError">
    <AlbumHero v-if="albumDetails" :album="albumDetails.album" :genres="albumDetails.genres" />
  </AsyncViewState>
</template>
