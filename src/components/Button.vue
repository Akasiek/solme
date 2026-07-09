<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    variant?: "primary" | "outline";
  }>(),
  {
    variant: "primary",
  },
);

const variantStyle = computed(() => {
  return props.variant === "primary"
    ? "bg-accent text-zinc-100 drop-shadow-lg drop-shadow-accent/25 transition-colors hover:bg-accent/85"
    : "border border-zinc-700 text-zinc-200 transition-colors hover:border-zinc-500 hover:text-white";
});
</script>

<template>
  <button
    class="inline-flex cursor-pointer items-center justify-center gap-2 rounded-md px-4 py-2 font-bold focus:ring-2 focus:ring-zinc-500 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
    :class="variantStyle"
  >
    <slot />
  </button>
</template>
