<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { CachedAlbum } from "@/types.ts";
import AlbumCard from "@/components/albums/AlbumCard.vue";
import AsyncViewState from "@/components/AsyncViewState.vue";
import { useAsyncData } from "@/composables/useAsyncData";

const {
  data: albums,
  isLoading,
  error: loadError,
} = useAsyncData(
  () =>
    invoke<CachedAlbum[]>("get_cached_albums", {
      offset: 128,
      limit: 24,
    }),
  [],
);
</script>

<template>
  <section class="space-y-6 p-6">
    <div class="space-y-2">
      <h1 class="font-serif text-4xl font-bold">Solm<span class="text-accent">ë</span></h1>
      <hr class="border-zinc-800" />
    </div>

    <AsyncViewState :is-loading="isLoading" :error="loadError">
      <div
        class="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8"
      >
        <div v-for="album in albums" :key="album.remoteId">
          <AlbumCard :album="album" />
        </div>
      </div>
    </AsyncViewState>
  </section>
</template>
