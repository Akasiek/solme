<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { storeToRefs } from "pinia";
import { computed, onMounted, ref, watch } from "vue";
import AsyncViewState from "@/components/AsyncViewState.vue";
import PaginatedResults from "@/components/PaginatedResults.vue";
import SongFiltersAside from "@/components/SongFiltersAside.vue";
import SongListRow from "@/components/SongListRow.vue";
import { useAsyncData } from "@/composables/useAsyncData";
import { useKeepAliveScrollRestoration } from "@/composables/useKeepAliveScrollRestoration";
import { useLibraryAnnotationsStore } from "@/stores/libraryAnnotations";
import { usePlayerStore } from "@/stores/player";
import type { CatalogFilter, SongPage, SongPageSort } from "@/types";

const query = ref("");
const filters = ref<CatalogFilter>({
  favoriteOnly: false,
  minimumRating: null,
  unratedOnly: false,
  fromYear: null,
  toYear: null,
  genres: [],
  neverPlayed: false,
  minimumPlayCount: null,
});
const filterKey = computed(() => JSON.stringify(filters.value));
const sort = ref<SongPageSort>("title");
const page = ref(1);
const pageSize = 50;
const emptyPage: SongPage = { items: [], genres: [], total: 0 };
const playerStore = usePlayerStore();
const { completedMutations } = storeToRefs(useLibraryAnnotationsStore());
const currentSongId = computed(() => playerStore.currentSong?.remoteId);

onMounted(async () => {
  await playerStore.startListening();
});

useKeepAliveScrollRestoration();
const {
  data,
  isLoading,
  hasLoaded,
  error: loadError,
  reload,
} = useAsyncData(
  async () => {
    const requestedQuery = query.value;
    const result = await invoke<SongPage>("get_song_page", {
      query: requestedQuery,
      filters: filters.value,
      sort: sort.value,
      pagination: { offset: (page.value - 1) * pageSize, limit: pageSize },
    });
    return { query: requestedQuery, result };
  },
  { query: "", result: emptyPage },
);
const matchingPage = computed(() => (data.value.query === query.value ? data.value.result : null));
let searchTimeout: number | undefined;
watch(query, (_, __, onCleanup) => {
  if (page.value !== 1) {
    page.value = 1;
  }
  const timeout = window.setTimeout(() => {
    searchTimeout = undefined;
    void reload();
  }, 200);
  searchTimeout = timeout;
  onCleanup(() => window.clearTimeout(timeout));
});
watch([filterKey, sort, page], (values, previous) => {
  const filtersChanged = values.slice(0, 2).some((value, index) => value !== previous[index]);
  if (filtersChanged && page.value !== 1) {
    page.value = 1;
    return;
  }
  if (searchTimeout === undefined) {
    void reload();
  }
});
watch(
  [() => completedMutations.value.song.favorite, () => completedMutations.value.song.rating],
  (counts, previous) => {
    const affectsActiveFilter =
      (counts[0] !== previous[0] && filters.value.favoriteOnly) ||
      (counts[1] !== previous[1] && (filters.value.minimumRating !== null || filters.value.unratedOnly));
    if (affectsActiveFilter && searchTimeout === undefined) {
      void reload();
    }
  },
);
</script>

<template>
  <section class="@container space-y-6 p-6">
    <header class="space-y-1">
      <h1 class="font-serif text-3xl font-bold">Songs</h1>
      <p class="font-sans text-sm text-zinc-400">Browse your complete song library.</p>
    </header>
    <AsyncViewState :is-loading="isLoading && !hasLoaded" :error="loadError">
      <div class="grid grid-cols-1 gap-6 @min-[48rem]:grid-cols-[14rem_minmax(0,1fr)]">
        <SongFiltersAside
          v-model:query="query"
          v-model:filters="filters"
          v-model:sort="sort"
          :genres="data.result.genres"
        />
        <PaginatedResults
          v-if="matchingPage"
          v-model:page="page"
          :total="matchingPage.total"
          :page-size="pageSize"
          item-label="songs"
          navigation-label="Song pages"
        >
          <div
            v-if="matchingPage.items.length"
            class="divide-y divide-zinc-800 rounded-lg border border-zinc-800 bg-zinc-900/20"
          >
            <SongListRow
              v-for="song in matchingPage.items"
              :key="song.remoteId"
              :song="song"
              :is-current="currentSongId === song.remoteId"
            />
          </div>
          <div v-else class="rounded-lg border border-dashed border-zinc-800 px-6 py-16 text-center">
            <p class="font-bold text-zinc-200">No matching songs</p>
            <p class="mt-1 font-sans text-sm text-zinc-500">Try changing one of the song filters.</p>
          </div>
        </PaginatedResults>
      </div>
    </AsyncViewState>
  </section>
</template>
