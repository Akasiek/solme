<script setup lang="ts">
import type { CachedArtistDetails } from "@/types";
import ArtistHeroArtwork from "@/components/Artist/ArtistHero/ArtistHeroArtwork.vue";
import ArtistHeroInfo from "@/components/Artist/ArtistHero/ArtistHeroInfo.vue";

defineProps<{
  artistDetails: CachedArtistDetails;
}>();
</script>

<template>
  <section class="my-6 px-6">
    <div class="container mx-auto flex flex-col items-center justify-center rounded-lg p-8 md:flex-row md:p-10 lg:p-12">
      <ArtistHeroArtwork :artist="artistDetails.artist" />
      <ArtistHeroInfo :artist="artistDetails.artist" />
    </div>
  </section>
</template>
