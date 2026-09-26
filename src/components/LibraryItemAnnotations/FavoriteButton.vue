<script setup lang="ts">
import { Heart } from "@lucide/vue";
import { computed, watch } from "vue";

import { useLibraryAnnotationsStore } from "@/stores/libraryAnnotations";
import type { AnnotatableLibraryItem } from "@/types";
import { libraryItemName } from "@/utils/libraryItem";

const { item, variant = "default" } = defineProps<{
  item: AnnotatableLibraryItem;
  variant?: "default" | "overlay";
}>();

const { favoriteFor, seedAnnotation, toggleFavorite } = useLibraryAnnotationsStore();
const currentFavorite = computed(() => favoriteFor(item));
const itemName = computed(() => libraryItemName(item));
const buttonClasses = computed(() => {
  if (variant === "overlay") {
    if (currentFavorite.value) {
      return "size-9 rounded-md border-accent/70 bg-zinc-950/75 text-accent shadow-lg shadow-black/30 hover:bg-zinc-900/90";
    }

    return "size-9 rounded-md border-white/25 bg-zinc-950/75 text-white shadow-lg shadow-black/30 hover:border-white/60 hover:bg-zinc-900";
  }

  if (currentFavorite.value) {
    return "size-7 rounded-md border-accent/60 bg-accent/10 text-accent hover:bg-accent/20";
  }

  return "size-7 rounded-md border-zinc-700 text-zinc-300 hover:border-zinc-500 hover:text-white";
});
const iconClass = computed(() => (variant === "overlay" ? "size-4" : "size-3.5"));
const actionLabel = computed(() =>
  currentFavorite.value ? `Remove ${itemName.value} from favorites` : `Add ${itemName.value} to favorites`,
);

watch(
  () => item,
  (value) => seedAnnotation(value),
  { immediate: true },
);
</script>

<template>
  <button
    type="button"
    class="grid cursor-pointer place-items-center border transition-colors focus-visible:ring-2 focus-visible:ring-white focus-visible:outline-none"
    :class="buttonClasses"
    :aria-pressed="currentFavorite"
    :aria-label="actionLabel"
    :title="actionLabel"
    @click="toggleFavorite(item)"
  >
    <Heart :class="iconClass" :fill="currentFavorite ? 'currentColor' : 'none'" aria-hidden="true" />
  </button>
</template>
