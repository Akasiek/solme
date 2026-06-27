<script setup lang="ts">
import { useAsyncData } from "@/composables/useAsyncData.ts";
import { invoke } from "@tauri-apps/api/core";
import { CachedAlbum } from "@/types.ts";
import AsyncViewState from "@/components/AsyncViewState.vue";

const { albumId } = defineProps<{ albumId: string }>();

const {
  data: album,
  isLoading,
  error: loadError,
} = useAsyncData(
  () =>
    invoke<CachedAlbum>("get_cached_album", {
      albumId: albumId,
    }),
  null,
);
</script>

<template>
  <AsyncViewState :is-loading="isLoading" :error="loadError">
    <div v-if="album === null">DUPA</div>
    <div v-else>
      <h1>{{ album.name }}</h1>
    </div>
  </AsyncViewState>
</template>
