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
    class="inline-flex size-10 cursor-pointer items-center justify-center rounded-md border border-zinc-800 bg-zinc-950/85 text-zinc-200 shadow-lg transition-colors hover:border-zinc-600 hover:bg-zinc-900 hover:text-white disabled:cursor-not-allowed disabled:opacity-35"
    :aria-label="variant === 'next' ? 'Next albums' : 'Previous albums'"
    :disabled="isDisabled"
    @click="handleClick"
  >
    <ChevronLeft v-if="variant === 'previous'" class="size-5" aria-hidden="true" />
    <ChevronRight v-else class="size-5" aria-hidden="true" />
  </button>
</template>
