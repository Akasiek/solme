<script setup lang="ts">
import { Star } from "@lucide/vue";
import { computed, watch } from "vue";

import { useLibraryAnnotationsStore } from "@/stores/libraryAnnotations";
import type { AnnotatableLibraryItem } from "@/types";
import { libraryItemName } from "@/utils/libraryItem";

const { item } = defineProps<{
  item: AnnotatableLibraryItem;
}>();

const { ratingFor, seedAnnotation, setRating } = useLibraryAnnotationsStore();
const currentRating = computed(() => ratingFor(item));
const itemName = computed(() => libraryItemName(item));

watch(
  () => item,
  (value) => seedAnnotation(value),
  { immediate: true },
);
</script>

<template>
  <div class="flex h-7 items-center gap-0 rounded-md border border-zinc-700 px-1" :aria-label="`${itemName} rating`">
    <button
      v-for="value in 5"
      :key="value"
      type="button"
      class="cursor-pointer rounded p-0.5 transition-colors hover:text-amber-300 focus:ring-2 focus:ring-zinc-500 focus:outline-none"
      :class="value <= (currentRating ?? 0) ? 'text-amber-400' : 'text-zinc-600'"
      :aria-label="currentRating === value ? `Remove ${value}-star rating` : `Rate ${value} out of 5`"
      :aria-pressed="currentRating === value"
      :title="currentRating === value ? `Remove ${value}-star rating` : `Rate ${value} out of 5`"
      @click="setRating(item, value)"
    >
      <Star class="size-3.5" :fill="value <= (currentRating ?? 0) ? 'currentColor' : 'none'" aria-hidden="true" />
    </button>
  </div>
</template>
