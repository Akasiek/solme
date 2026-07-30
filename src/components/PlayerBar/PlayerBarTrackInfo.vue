<script setup lang="ts">
import { SquareArrowOutUpRight } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { CachedSong } from "@/types.ts";
import { artworkSource } from "@/utils/artwork.ts";
import { useLayoutStore } from "@/stores/layout.ts";

defineProps<{ currentSong: CachedSong }>();

const layoutStore = useLayoutStore();
const { toggleBigArtwork } = layoutStore;
const { isBigArtworkShown, isLeftAsideCollapsed } = storeToRefs(layoutStore);
</script>

<template>
  <div class="flex h-16 w-full min-w-0 items-center gap-4">
    <Transition
      appear
      enter-active-class="transition duration-300 ease-out"
      enter-from-class="translate-y-3 scale-95 opacity-0"
      enter-to-class="translate-y-0 scale-100 opacity-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="translate-y-0 scale-100 opacity-100"
      leave-to-class="translate-y-2 scale-95 opacity-0"
    >
      <div
        v-if="!isBigArtworkShown || isLeftAsideCollapsed"
        class="relative size-16 shrink-0 origin-bottom overflow-hidden rounded"
      >
        <div
          class="absolute inset-0 flex cursor-pointer items-center justify-center bg-black/50 opacity-0 transition-opacity hover:opacity-100"
          @click="toggleBigArtwork"
        >
          <SquareArrowOutUpRight />
        </div>
        <img
          :src="artworkSource(currentSong.artworkPath)"
          alt="Artwork"
          class="h-full w-full object-cover object-center"
        />
      </div>
    </Transition>
    <div class="flex min-w-0 flex-col gap-1">
      <RouterLink
        :to="{ name: 'album', params: { albumId: currentSong.albumId } }"
        :title="currentSong.title"
        class="line-clamp-1 font-serif font-bold text-zinc-100 hover:underline"
      >
        {{ currentSong.title }}
      </RouterLink>
      <RouterLink
        :to="{ name: 'artist', params: { artistId: currentSong.artistId } }"
        :title="currentSong.artistName"
        class="line-clamp-1 font-sans text-sm text-zinc-300 hover:underline"
      >
        {{ currentSong.artistName }}
      </RouterLink>
    </div>
  </div>
</template>
