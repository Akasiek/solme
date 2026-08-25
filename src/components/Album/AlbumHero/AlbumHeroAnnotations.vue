<script setup lang="ts">
import { FavoriteButton, RatingStars } from "@/components/LibraryItemAnnotations";
import type { CachedAlbum } from "@/types.ts";

defineProps<{
  album: CachedAlbum;
}>();
</script>

<template>
  <div class="flex flex-wrap items-center gap-2">
    <FavoriteButton item-kind="album" :item-id="album.remoteId" :item-name="album.name" :favorite="album.favorite" />
    <RatingStars item-kind="album" :item-id="album.remoteId" :item-name="album.name" :rating="album.rating" />
  </div>
</template>
