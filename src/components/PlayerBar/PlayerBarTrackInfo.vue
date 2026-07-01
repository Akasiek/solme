<script setup lang="ts">
import { CachedSong } from "@/types.ts";
import { artworkSource } from "@/utils/artwork.ts";

defineProps<{ currentSong: CachedSong }>();
</script>

<template>
  <div class="flex h-16 w-full min-w-0 items-center gap-4">
    <div class="size-16 shrink-0 overflow-hidden rounded">
      <img
        :src="artworkSource(currentSong.artworkPath)"
        alt="Artwork"
        class="h-full w-full object-cover object-center"
      />
    </div>
    <div class="flex min-w-0 flex-col gap-1">
      <span class="line-clamp-1 font-serif font-bold">{{ currentSong.title }}</span>
      <RouterLink
        :to="{ name: 'artist', params: { artistId: currentSong.artistId } }"
        class="line-clamp-1 text-sm text-zinc-300 hover:underline"
      >
        {{ currentSong.artistName }}
      </RouterLink>
    </div>
  </div>
</template>
