<script setup lang="ts">
import type { AnnotatableLibraryItem } from "@/types.ts";
import FavoriteButton from "./FavoriteButton.vue";
import RatingStars from "./RatingStars.vue";

defineProps<{
  item: AnnotatableLibraryItem;
}>();
</script>

<template>
  <div class="flex flex-wrap items-center gap-1.5">
    <FavoriteButton :item="item" />
    <RatingStars :item="item" />
  </div>
</template>
