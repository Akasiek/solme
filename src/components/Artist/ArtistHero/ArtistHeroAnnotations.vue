<script setup lang="ts">
import { FavoriteButton, RatingStars } from "@/components/LibraryItemAnnotations";
import type { CachedArtist } from "@/types";

const { artist } = defineProps<{
  artist: CachedArtist;
}>();
</script>

<template>
  <div class="flex flex-wrap items-center gap-2 pt-2 font-sans">
    <FavoriteButton :item="artist" />
    <RatingStars :item="artist" />
  </div>
</template>
