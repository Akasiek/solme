<script setup lang="ts">
import { computed } from "vue";

const { genres } = defineProps<{
  genres: string[];
}>();

const albumGenres = computed(() => genres.filter(Boolean));
</script>

<template>
  <div v-if="albumGenres.length > 0" class="flex flex-wrap gap-2">
    <span v-for="genre in albumGenres" :key="genre" class="rounded-md bg-zinc-800 px-2.5 py-1 text-sm text-zinc-200">
      {{ genre }}
    </span>
  </div>
</template>
