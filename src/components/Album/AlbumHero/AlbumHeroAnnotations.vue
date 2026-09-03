<script setup lang="ts">
import { FavoriteButton, RatingStars } from "@/components/LibraryItemAnnotations";
import type { CachedAlbum } from "@/types.ts";

defineProps<{
  album: CachedAlbum;
}>();
</script>

<template>
  <div class="flex flex-wrap items-center gap-2">
    <FavoriteButton :item="album" />
    <RatingStars :item="album" />
  </div>
</template>
