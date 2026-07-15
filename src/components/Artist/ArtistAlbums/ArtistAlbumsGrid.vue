<script setup lang="ts">
import AlbumCard from "@/components/Album/AlbumCard";
import type { CachedAlbum } from "@/types";

defineProps<{
  albums: CachedAlbum[];
}>();
</script>

<template>
  <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
    <AlbumCard v-for="album in albums" :key="album.remoteId" :album="album" />
  </div>
</template>
