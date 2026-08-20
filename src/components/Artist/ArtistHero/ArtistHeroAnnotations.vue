<script setup lang="ts">
import { FavoriteButton, RatingStars } from "@/components/LibraryItemAnnotations";
import type { CachedArtist } from "@/types";

const { artist } = defineProps<{
  artist: CachedArtist;
}>();
</script>

<template>
  <div class="flex flex-wrap items-center gap-3 pt-2 font-sans">
    <FavoriteButton
      item-kind="artist"
      :item-id="artist.remoteId"
      :item-name="artist.name"
      :favorite="artist.favorite"
    />
    <RatingStars item-kind="artist" :item-id="artist.remoteId" :item-name="artist.name" :rating="artist.rating" />
  </div>
</template>
