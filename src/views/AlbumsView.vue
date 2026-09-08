<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, ref, watch } from "vue";

import AlbumCard from "@/components/Album/AlbumCard";
import AlbumFiltersAside from "@/components/Album/AlbumFiltersAside.vue";
import AsyncViewState from "@/components/AsyncViewState.vue";
import LibraryCardGrid from "@/components/LibraryCardGrid.vue";
import PaginatedResults from "@/components/PaginatedResults.vue";
import { useAsyncData } from "@/composables/useAsyncData";
import type { AlbumPage, AlbumPageSort } from "@/types";

const query = ref("");
const selectedAlbumTypes = ref<string[]>([]);
const albumTypeKey = computed(() => selectedAlbumTypes.value.join("\0"));
const sort = ref<AlbumPageSort>("artist");
const page = ref(1);
const pageSize = 24;

const {
  data: albumPage,
  isLoading,
  hasLoaded,
  error: loadError,
  reload,
} = useAsyncData(
  () =>
    invoke<AlbumPage>("get_album_page", {
      query: query.value,
      albumTypes: selectedAlbumTypes.value,
      sort: sort.value,
      pagination: { offset: (page.value - 1) * pageSize, limit: pageSize },
    }),
  { items: [], albumTypes: [], total: 0 },
);

watch([query, albumTypeKey, sort, page], (values, previousValues, onCleanup) => {
  const filtersChanged = values.slice(0, 3).some((value, index) => value !== previousValues[index]);
  if (filtersChanged && page.value !== 1) {
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
      <h1 class="font-serif text-3xl font-bold">Albums</h1>
      <p class="font-sans text-sm text-zinc-400">Browse your complete album library.</p>
    </header>

    <AsyncViewState :is-loading="isLoading && !hasLoaded" :error="loadError">
      <div class="grid grid-cols-1 gap-6 @min-[48rem]:grid-cols-[14rem_minmax(0,1fr)]">
        <AlbumFiltersAside
          v-model:query="query"
          v-model:selected-album-types="selectedAlbumTypes"
          v-model:sort="sort"
          :album-types="albumPage.albumTypes"
        />

        <PaginatedResults
          v-model:page="page"
          :total="albumPage.total"
          :page-size="pageSize"
          item-label="albums"
          navigation-label="Album pages"
        >
          <LibraryCardGrid v-if="albumPage.items.length > 0">
            <AlbumCard v-for="album in albumPage.items" :key="album.remoteId" :album="album" />
          </LibraryCardGrid>
          <div v-else class="rounded-lg border border-dashed border-zinc-800 px-6 py-16 text-center">
            <p class="font-bold text-zinc-200">No matching albums</p>
            <p class="mt-1 font-sans text-sm text-zinc-500">Try changing the title, artist, or album type filter.</p>
          </div>
        </PaginatedResults>
      </div>
    </AsyncViewState>
  </section>
</template>
