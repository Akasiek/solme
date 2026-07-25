<script setup lang="ts">
import { computed, onMounted } from "vue";
import { SquareArrowOutDownLeft } from "@lucide/vue";
import { usePlayerStore } from "@/stores/player.ts";
import { artworkSource } from "@/utils/artwork.ts";
import { useLayoutStore } from "@/stores/layout.ts";

const playerStore = usePlayerStore();
const artworkPath = computed(() => playerStore.currentSong?.artworkPath);

defineProps<{
  isCollapsed: boolean;
}>();
const layoutStore = useLayoutStore();
const isBigArtworkShown = computed(() => layoutStore.isBigArtworkShown);

onMounted(async () => {
  await playerStore.startListening();
});
</script>

<template>
  <Transition
    appear
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="translate-y-4 scale-95 opacity-0"
    enter-to-class="translate-y-0 scale-100 opacity-100"
    leave-active-class="transition duration-150 ease-in"
    leave-from-class="translate-y-0 scale-100 opacity-100"
    leave-to-class="translate-y-3 scale-95 opacity-0"
  >
    <div
      v-if="artworkPath && !isCollapsed && isBigArtworkShown"
      class="group relative flex aspect-square w-full shrink-0 origin-bottom items-center justify-center overflow-hidden bg-zinc-800"
    >
      <div
        class="absolute top-2 right-2 flex cursor-pointer items-center justify-center rounded bg-black/50 p-2 opacity-0 transition-opacity group-hover:opacity-100"
        @click="layoutStore.toggleBigArtwork()"
      >
        <SquareArrowOutDownLeft class="size-5" />
      </div>
      <img :src="artworkSource(artworkPath)" alt="Artwork" class="h-full w-full object-cover" />
    </div>
  </Transition>
</template>
