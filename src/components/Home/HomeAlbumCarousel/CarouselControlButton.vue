<script setup lang="ts">
import { ChevronLeft, ChevronRight } from "@lucide/vue";

defineProps<{
  isDisabled: boolean;
  handleClick: () => void;
  variant: "previous" | "next";
}>();
</script>

<template>
  <button
    type="button"
    class="absolute top-1/2 z-10 -mt-12 inline-flex size-10 -translate-y-1/2 cursor-pointer items-center justify-center rounded-md border border-zinc-800 bg-zinc-950/85 text-zinc-200 shadow-lg transition-colors hover:border-zinc-600 hover:bg-zinc-900 hover:text-white disabled:cursor-not-allowed disabled:opacity-35"
    :class="variant === 'next' ? 'right-2 left-auto' : 'left-2'"
    :aria-label="variant === 'next' ? 'Next albums' : 'Previous albums'"
    :disabled="isDisabled"
    @click="handleClick"
  >
    <ChevronLeft v-if="variant === 'previous'" class="size-5" aria-hidden="true" />
    <ChevronRight v-else class="size-5" aria-hidden="true" />
  </button>
</template>
