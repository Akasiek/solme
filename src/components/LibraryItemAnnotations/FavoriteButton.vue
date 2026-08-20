<script setup lang="ts">
import { Heart } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { ref, watch } from "vue";

import { useToastStore } from "@/stores/toast";
import type { LibraryItemAnnotation, LibraryItemKind } from "@/types";

const props = defineProps<{
  itemKind: LibraryItemKind;
  itemId: string;
  itemName: string;
  favorite: boolean;
}>();

const toastStore = useToastStore();
const currentFavorite = ref(props.favorite);
let mutationQueue = Promise.resolve();

watch(
  () => props.favorite,
  (value) => {
    currentFavorite.value = value;
  },
);

const errorMessage = (cause: unknown) =>
  typeof cause === "string" ? cause : cause instanceof Error ? cause.message : "Unexpected error.";

const toggleFavorite = () => {
  const oldFavoriteValue = currentFavorite.value;
  const newFavoriteValue = !oldFavoriteValue;
  const { itemId, itemKind, itemName } = props;

  currentFavorite.value = newFavoriteValue;

  mutationQueue = mutationQueue.then(async () => {
    try {
      await invoke<LibraryItemAnnotation>("set_library_item_favorite", {
        itemKind,
        itemId,
        favorite: newFavoriteValue,
      });
    } catch (cause) {
      if (props.itemId !== itemId || currentFavorite.value !== newFavoriteValue) return;

      currentFavorite.value = oldFavoriteValue;
      toastStore.show(`Could not update ${itemName}: ${errorMessage(cause)}`);
    }
  });
};
</script>

<template>
  <button
    type="button"
    class="grid size-10 cursor-pointer place-items-center rounded-md border transition-colors focus:ring-2 focus:ring-zinc-500 focus:outline-none"
    :class="
      currentFavorite
        ? 'border-accent/60 bg-accent/10 text-accent hover:bg-accent/20'
        : 'border-zinc-700 text-zinc-300 hover:border-zinc-500 hover:text-white'
    "
    :aria-pressed="currentFavorite"
    :aria-label="currentFavorite ? `Remove ${itemName} from favorites` : `Add ${itemName} to favorites`"
    :title="currentFavorite ? `Remove ${itemName} from favorites` : `Add ${itemName} to favorites`"
    @click="toggleFavorite"
  >
    <Heart class="size-5" :fill="currentFavorite ? 'currentColor' : 'none'" aria-hidden="true" />
  </button>
</template>
