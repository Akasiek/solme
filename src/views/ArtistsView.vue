<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref, watch } from "vue";
import ArtistCard from "@/components/Artist/ArtistCard";
import ArtistFiltersAside from "@/components/Artist/ArtistFiltersAside.vue";
import AsyncViewState from "@/components/AsyncViewState.vue";
import LibraryCardGrid from "@/components/LibraryCardGrid.vue";
import PaginatedResults from "@/components/PaginatedResults.vue";
import { useAsyncData } from "@/composables/useAsyncData";
import type { ArtistPageSort, CachedArtist, Paginated } from "@/types";

const query = ref("");
const sort = ref<ArtistPageSort>("name");
const page = ref(1);
const pageSize = 24;
const {
  data,
  isLoading,
  hasLoaded,
  error: loadError,
  reload,
} = useAsyncData(
  () =>
    invoke<Paginated<CachedArtist>>("get_artist_page", {
      query: query.value,
      sort: sort.value,
      pagination: { offset: (page.value - 1) * pageSize, limit: pageSize },
    }),
  { items: [], total: 0 },
);
watch([query, sort, page], (values, previous, onCleanup) => {
  if (values.slice(0, 2).some((value, index) => value !== previous[index]) && page.value !== 1) {
    page.value = 1;
    return;
  }
  const timeout = window.setTimeout(() => void reload(), 200);
  onCleanup(() => window.clearTimeout(timeout));
});
</script>

<template>
  <section class="@container space-y-6 p-6">
    <header class="space-y-1">
      <h1 class="text-3xl font-bold">Artists</h1>
      <p class="font-sans text-sm text-zinc-400">Browse your complete artist library.</p>
    </header>
    <AsyncViewState :is-loading="isLoading && !hasLoaded" :error="loadError">
      <div class="grid grid-cols-1 gap-6 @min-[48rem]:grid-cols-[14rem_minmax(0,1fr)]">
        <ArtistFiltersAside v-model:query="query" v-model:sort="sort" />
        <PaginatedResults
          v-model:page="page"
          :total="data.total"
          :page-size="pageSize"
          item-label="artists"
          navigation-label="Artist pages"
        >
          <LibraryCardGrid v-if="data.items.length"
            ><ArtistCard v-for="artist in data.items" :key="artist.remoteId" :artist="artist"
          /></LibraryCardGrid>
          <div v-else class="rounded-lg border border-dashed border-zinc-800 px-6 py-16 text-center">
            <p class="font-bold text-zinc-200">No matching artists</p>
            <p class="mt-1 font-sans text-sm text-zinc-500">Try changing the artist filter.</p>
          </div>
        </PaginatedResults>
      </div>
    </AsyncViewState>
  </section>
</template>
