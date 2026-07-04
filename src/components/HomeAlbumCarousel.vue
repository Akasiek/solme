<script setup lang="ts">
import { CachedAlbum } from "@/types.ts";
import AlbumCard from "@/components/Album/AlbumCard/AlbumCard.vue";

defineProps<{
  title: string;
  albums: CachedAlbum[];
}>();
</script>

<template>
  <section class="space-y-3">
    <h2 class="font-serif text-2xl font-bold">{{ title }}</h2>
    <p v-if="albums.length === 0" class="text-zinc-400">No albums to show.</p>
    <div v-else class="flex snap-x gap-4 overflow-x-auto pb-3">
      <div v-for="album in albums" :key="album.remoteId" class="w-40 shrink-0 snap-start md:w-44 lg:w-48">
        <AlbumCard :album="album" />
      </div>
    </div>
  </section>
</template>
