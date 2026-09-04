<script setup lang="ts">
import { Heart } from "@lucide/vue";
import { computed, watch } from "vue";

import { useLibraryAnnotationsStore } from "@/stores/libraryAnnotations";
import type { AnnotatableLibraryItem } from "@/types";
import { libraryItemName } from "@/utils/libraryItem";

const { item } = defineProps<{
  item: AnnotatableLibraryItem;
}>();

const { favoriteFor, seedAnnotation, toggleFavorite } = useLibraryAnnotationsStore();
const currentFavorite = computed(() => favoriteFor(item));
const itemName = computed(() => libraryItemName(item));

watch(
  () => item,
  (value) => seedAnnotation(value),
  { immediate: true },
);
</script>

<template>
  <button
    type="button"
    class="grid size-7 cursor-pointer place-items-center rounded-md border transition-colors focus:ring-2 focus:ring-zinc-500 focus:outline-none"
    :class="
      currentFavorite
        ? 'border-accent/60 bg-accent/10 text-accent hover:bg-accent/20'
        : 'border-zinc-700 text-zinc-300 hover:border-zinc-500 hover:text-white'
    "
    :aria-pressed="currentFavorite"
    :aria-label="currentFavorite ? `Remove ${itemName} from favorites` : `Add ${itemName} to favorites`"
    :title="currentFavorite ? `Remove ${itemName} from favorites` : `Add ${itemName} to favorites`"
    @click="toggleFavorite(item)"
  >
    <Heart class="size-3.5" :fill="currentFavorite ? 'currentColor' : 'none'" aria-hidden="true" />
  </button>
</template>
