<script setup lang="ts">
import ArtistCard from "@/components/Artist/ArtistCard";
import type { CachedArtist } from "@/types";

defineProps<{
  artists: CachedArtist[];
}>();

defineEmits<{
  select: [];
}>();
</script>

<template>
  <section class="space-y-3">
    <h2 class="font-serif text-2xl font-bold">Artists</h2>
    <div
      class="grid auto-cols-[9rem] grid-flow-col justify-start gap-4 overflow-x-auto pb-1 md:auto-cols-[11rem] lg:auto-cols-[13rem]"
    >
      <ArtistCard v-for="artist in artists" :key="artist.remoteId" :artist="artist" @select="$emit('select')" />
    </div>
  </section>
</template>
