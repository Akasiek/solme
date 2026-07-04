<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { HomeAlbumSections } from "@/types.ts";
import AsyncViewState from "@/components/AsyncViewState.vue";
import HomeAlbumCarousel from "@/components/HomeAlbumCarousel.vue";
import { useAsyncData } from "@/composables/useAsyncData";

const {
  data: albumSections,
  isLoading,
  error: loadError,
} = useAsyncData(
  () =>
    invoke<HomeAlbumSections>("get_home_album_sections", {
      limit: 24,
    }),
  {
    randomAlbums: [],
    newlyAddedAlbums: [],
    newlyReleasedAlbums: [],
  },
);
</script>

<template>
  <section class="space-y-6 p-6">
    <div class="space-y-2">
      <h1 class="font-serif text-4xl font-bold">Home</h1>
      <hr class="border-zinc-800" />
    </div>

    <AsyncViewState :is-loading="isLoading" :error="loadError">
      <div class="space-y-8">
        <HomeAlbumCarousel title="Random albums" :albums="albumSections.randomAlbums" />
        <HomeAlbumCarousel title="Recently added albums" :albums="albumSections.newlyAddedAlbums" />
        <HomeAlbumCarousel title="Recently released albums" :albums="albumSections.newlyReleasedAlbums" />
      </div>
    </AsyncViewState>
  </section>
</template>
