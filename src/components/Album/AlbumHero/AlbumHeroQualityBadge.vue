<script setup lang="ts">
import { computed } from "vue";

const { audioFormats } = defineProps<{
  audioFormats: string[];
}>();

const audioFormatLabel = computed(() => audioFormats.join(" / "));
</script>

<template>
  <span
    v-if="audioFormatLabel"
    class="mb-1 inline-flex rounded-md border border-accent bg-zinc-900 px-2.5 py-1 font-sans text-xs font-medium text-accent drop-shadow-md drop-shadow-accent/20"
  >
    {{ audioFormatLabel }}
  </span>
</template>
