<script setup lang="ts">
import SearchTrackRow from "@/components/Search/SearchTrackRow";
import type { CachedSong } from "@/types";

defineProps<{
  songs: CachedSong[];
}>();

defineEmits<{
  select: [];
}>();
</script>

<template>
  <section class="space-y-3">
    <h2 class="font-serif text-2xl font-bold">Tracks</h2>
    <div class="divide-y divide-zinc-800">
      <SearchTrackRow v-for="song in songs" :key="song.remoteId" :song="song" @select="$emit('select')" />
    </div>
  </section>
</template>
