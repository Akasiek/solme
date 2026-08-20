<script setup lang="ts">
import AlbumCard from "@/components/Album/AlbumCard";
import type { CachedAlbum } from "@/types";

defineProps<{
  albums: CachedAlbum[];
}>();
</script>

<template>
  <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,12rem),1fr))] gap-x-2 gap-y-4">
    <AlbumCard v-for="album in albums" :key="album.remoteId" :album="album" show-release-year />
  </div>
</template>
