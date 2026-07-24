<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { HomeAlbumSections } from "@/types.ts";
import AsyncViewState from "@/components/AsyncViewState.vue";
import HomeHero from "@/components/Home/HomeHero";
import AlbumCarousel from "@/components/Album/AlbumCarousel";
import { useAsyncData } from "@/composables/useAsyncData";

const {
  data: albumSections,
  isLoading,
  error: loadError,
} = useAsyncData(
  () =>
    invoke<HomeAlbumSections>("get_home_album_sections", {
      limit: 48,
    }),
  {
    heroRandomAlbums: [],
    recentlyPlayedAlbums: [],
    mostPlayedAlbums: [],
    randomAlbums: [],
    newlyAddedAlbums: [],
    newlyReleasedAlbums: [],
  },
);
</script>

<template>
  <section class="space-y-8 p-6">
    <!--    <WindowFocusTest />-->

    <AsyncViewState :is-loading="isLoading" :error="loadError">
      <div class="space-y-8">
        <HomeHero :album-sections="albumSections" />
        <div id="explore-library" class="scroll-mt-6">
          <AlbumCarousel title="Explore library" :albums="albumSections.randomAlbums" />
        </div>
        <div id="recently-played" class="scroll-mt-6">
          <AlbumCarousel title="Recently played" :albums="albumSections.recentlyPlayedAlbums" />
        </div>
        <div id="most-played" class="scroll-mt-6">
          <AlbumCarousel title="Most played" :albums="albumSections.mostPlayedAlbums" />
        </div>
        <div id="recently-added" class="scroll-mt-6">
          <AlbumCarousel title="Recently added albums" :albums="albumSections.newlyAddedAlbums" />
        </div>
        <div id="recently-released" class="scroll-mt-6">
          <AlbumCarousel title="Recently released albums" :albums="albumSections.newlyReleasedAlbums" />
        </div>
      </div>
    </AsyncViewState>
  </section>
</template>
