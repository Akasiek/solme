<script setup lang="ts">
import { Star } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { ref, watch } from "vue";

import { useToastStore } from "@/stores/toast";
import type { LibraryItemAnnotation, LibraryItemKind } from "@/types";

const props = defineProps<{
  itemKind: LibraryItemKind;
  itemId: string;
  itemName: string;
  rating: number | null;
}>();

const toastStore = useToastStore();
const currentRating = ref(props.rating);
let mutationQueue = Promise.resolve();

watch(
  () => props.rating,
  (value) => {
    currentRating.value = value;
  },
);

const errorMessage = (cause: unknown) =>
  typeof cause === "string" ? cause : cause instanceof Error ? cause.message : "Unexpected error.";

const setRating = (value: number) => {
  const previousRating = currentRating.value;
  const nextRating = previousRating === value ? null : value;
  const { itemId, itemKind, itemName } = props;

  currentRating.value = nextRating;

  mutationQueue = mutationQueue.then(async () => {
    try {
      await invoke<LibraryItemAnnotation>("set_library_item_rating", {
        itemKind,
        itemId,
        rating: nextRating,
      });
    } catch (cause) {
      if (props.itemId !== itemId || currentRating.value !== nextRating) return;

      currentRating.value = previousRating;
      toastStore.show(`Could not update ${itemName}: ${errorMessage(cause)}`);
    }
  });
};
</script>

<template>
  <div class="flex h-10 items-center gap-0.5 rounded-md border border-zinc-700 px-2" :aria-label="`${itemName} rating`">
    <button
      v-for="value in 5"
      :key="value"
      type="button"
      class="cursor-pointer rounded p-0.5 transition-colors hover:text-amber-300 focus:ring-2 focus:ring-zinc-500 focus:outline-none"
      :class="value <= (currentRating ?? 0) ? 'text-amber-400' : 'text-zinc-600'"
      :aria-label="currentRating === value ? `Remove ${value}-star rating` : `Rate ${value} out of 5`"
      :aria-pressed="currentRating === value"
      :title="currentRating === value ? `Remove ${value}-star rating` : `Rate ${value} out of 5`"
      @click="setRating(value)"
    >
      <Star class="size-4" :fill="value <= (currentRating ?? 0) ? 'currentColor' : 'none'" aria-hidden="true" />
    </button>
  </div>
</template>
