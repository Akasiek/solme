<script setup lang="ts">
import { computed } from "vue";

const { genres } = defineProps<{
  genres: string[];
}>();

const albumGenres = computed(() => genres.filter(Boolean));
</script>

<template>
  <div v-if="albumGenres.length > 0" class="flex flex-wrap gap-1.5 @min-[64rem]:gap-2">
    <span
      v-for="genre in albumGenres"
      :key="genre"
      class="rounded-md bg-zinc-800 px-2 py-0.5 text-xs text-zinc-200 @min-[64rem]:px-2.5 @min-[64rem]:py-1 @min-[64rem]:text-sm"
    >
      {{ genre }}
    </span>
  </div>
</template>
