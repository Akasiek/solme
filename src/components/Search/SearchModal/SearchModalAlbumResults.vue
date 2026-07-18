<script setup lang="ts">
import { computed } from "vue";

import AlbumCard from "@/components/Album/AlbumCard";
import type { CachedAlbum } from "@/types";

const props = defineProps<{
  albums: CachedAlbum[];
}>();

defineEmits<{
  select: [];
}>();

const gridClass = computed(() =>
  props.albums.length === 5
    ? "grid-flow-col auto-cols-[9rem] md:grid-flow-row md:grid-cols-5 md:auto-cols-auto"
    : "grid-flow-col auto-cols-[9rem] md:auto-cols-[11rem] lg:auto-cols-[13rem]",
);
</script>

<template>
  <section class="space-y-3">
    <h2 class="font-serif text-2xl font-bold">Albums</h2>
    <div class="grid justify-start gap-4 overflow-x-auto pb-1" :class="gridClass">
      <AlbumCard v-for="album in albums" :key="album.remoteId" :album="album" @click="$emit('select')" />
    </div>
  </section>
</template>
