<script setup lang="ts">
defineProps<{
  albumGenres: string[];
}>();
</script>

<template>
  <div v-if="albumGenres.length > 0" class="space-y-2 rounded border border-zinc-700 p-5">
    <h2 class="font-sans text-xs font-semibold tracking-wide text-zinc-500 uppercase">Genres</h2>
    <ul class="flex flex-wrap gap-2">
      <li
        v-for="genre in albumGenres"
        :key="genre"
        class="rounded-md border border-zinc-700 px-2.5 py-1 text-sm text-zinc-200"
      >
        {{ genre }}
      </li>
    </ul>
  </div>
</template>
