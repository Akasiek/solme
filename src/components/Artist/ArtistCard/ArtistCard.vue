<script setup lang="ts">
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";
import type { CachedArtist } from "@/types";
import { artworkSource } from "@/utils/artwork";

defineProps<{
  artist: CachedArtist;
}>();

defineEmits<{
  select: [];
}>();
</script>

<template>
  <RouterLink
    class="group grid gap-4 transition-colors duration-300 ease-in-out"
    :to="{ name: 'artist', params: { artistId: artist.remoteId } }"
    @click="$emit('select')"
  >
    <div
      class="aspect-square w-full overflow-hidden rounded-full border border-zinc-800 bg-zinc-950 transition-colors duration-300 ease-in-out group-hover:border-zinc-600"
    >
      <img
        v-if="artist.artworkPath"
        :src="artworkSource(artist.artworkPath)"
        :alt="`${artist.name} artwork`"
        class="h-full w-full object-cover object-center"
      />
      <MissingCoverImage v-else />
    </div>

    <div class="text-center">
      <h3 class="line-clamp-1 font-serif font-bold group-hover:underline" :title="artist.name">
        {{ artist.name }}
      </h3>
    </div>
  </RouterLink>
</template>
