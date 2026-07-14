<script setup lang="ts">
import { computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Library, Music, Tags, Users } from "@lucide/vue";
import AsyncViewState from "@/components/AsyncViewState.vue";
import { useAsyncData } from "@/composables/useAsyncData";
import type { LibrarySummary } from "@/types";

const emptySummary: LibrarySummary = {
  artistCount: 0,
  albumCount: 0,
  songCount: 0,
  genreCount: 0,
};

const {
  data: summary,
  isLoading,
  error: loadError,
} = useAsyncData(() => invoke<LibrarySummary>("get_library_summary"), emptySummary);

const formatter = new Intl.NumberFormat();
const stats = computed(() => [
  {
    label: "Artists",
    value: summary.value.artistCount,
    icon: Users,
  },
  {
    label: "Albums",
    value: summary.value.albumCount,
    icon: Library,
  },
  {
    label: "Songs",
    value: summary.value.songCount,
    icon: Music,
  },
  {
    label: "Genres",
    value: summary.value.genreCount,
    icon: Tags,
  },
]);
</script>

<template>
  <section class="overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900">
    <AsyncViewState :is-loading="isLoading" :error="loadError">
      <div class="grid grid-cols-2 divide-x divide-y divide-zinc-800 md:grid-cols-4 md:divide-y-0">
        <div v-for="stat in stats" :key="stat.label" class="min-w-0 p-4 md:p-5">
          <div class="flex items-center gap-2 text-zinc-400">
            <component :is="stat.icon" class="size-4 shrink-0" aria-hidden="true" />
            <span class="truncate font-sans text-xs font-semibold uppercase">{{ stat.label }}</span>
          </div>
          <p class="mt-2 truncate text-2xl leading-tight font-bold text-white md:text-3xl">
            {{ formatter.format(stat.value) }}
          </p>
        </div>
      </div>
    </AsyncViewState>
  </section>
</template>
