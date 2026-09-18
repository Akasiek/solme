<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed } from "vue";
import { RouterLink } from "vue-router";

import AsyncViewState from "@/components/AsyncViewState.vue";
import { useAsyncData } from "@/composables/useAsyncData";
import type { GenreSummary } from "@/types";

const sizeClasses = [
  "px-2.5 py-1.5 text-xs",
  "px-3.5 py-2.5 text-lg",
  "px-4.5 py-3 text-2xl",
  "px-5.5 py-4 text-4xl",
  "px-7 py-5 text-5xl @min-[48rem]:text-6xl",
] as const;

const countFormatter = new Intl.NumberFormat();
const { data: genres, isLoading, error: loadError } = useAsyncData(() => invoke<GenreSummary[]>("get_genres"), []);

const sizedGenres = computed(() => {
  if (genres.value.length === 0) {
    return [];
  }

  const counts = genres.value.map((genre) => Math.log1p(genre.albumCount));
  const minimum = Math.min(...counts);
  const maximum = Math.max(...counts);

  return genres.value.map((genre) => {
    const count = Math.log1p(genre.albumCount);
    const tier = minimum === maximum ? 2 : Math.round(((count - minimum) / (maximum - minimum)) * 4);
    return { ...genre, sizeClass: sizeClasses[tier] };
  });
});
</script>

<template>
  <section class="@container space-y-6 p-6">
    <header class="space-y-1">
      <h1 class="font-serif text-3xl font-bold">Genres</h1>
      <p class="font-sans text-sm text-zinc-400">Explore your album library by genre.</p>
    </header>

    <AsyncViewState :is-loading="isLoading" :error="loadError">
      <div v-if="sizedGenres.length" class="flex flex-wrap items-center justify-center gap-3">
        <RouterLink
          v-for="genre in sizedGenres"
          :key="genre.name"
          :to="{ name: 'albums', query: { genre: genre.name } }"
          :class="genre.sizeClass"
          class="group inline-flex max-w-full items-baseline gap-2 rounded-lg border border-zinc-800 bg-zinc-900/40 font-serif font-bold text-zinc-200 transition-colors hover:border-zinc-600 hover:bg-zinc-800/70 hover:text-white focus-visible:ring-2 focus-visible:ring-accent/60 focus-visible:outline-none"
          :aria-label="`${genre.name}, ${genre.albumCount} albums`"
        >
          <span class="truncate">{{ genre.name }}</span>
          <span class="shrink-0 font-sans text-[0.625rem] font-semibold text-zinc-500 group-hover:text-zinc-400">
            {{ countFormatter.format(genre.albumCount) }}
          </span>
        </RouterLink>
      </div>

      <div v-else class="rounded-lg border border-dashed border-zinc-800 px-6 py-16 text-center">
        <p class="font-bold text-zinc-200">No genres found</p>
        <p class="mt-1 font-sans text-sm text-zinc-500">Sync your library to make genres available.</p>
      </div>
    </AsyncViewState>
  </section>
</template>
