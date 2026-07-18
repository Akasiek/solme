<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { watchDebounced } from "@vueuse/core";
import { computed, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";

import type { CachedAlbum, CachedArtist, CachedSong } from "@/types";
import SearchSectionResults from "./SearchSectionResults.vue";
import SearchSectionSearchForm from "./SearchSectionSearchForm.vue";

const props = defineProps<{
  variant: "modal" | "view";
  isOpen?: boolean;
}>();

defineOptions({ name: "SearchSection" });

const emit = defineEmits<{
  navigate: [];
}>();

const route = useRoute();
const router = useRouter();
const query = ref("");
const albums = ref<CachedAlbum[]>([]);
const artists = ref<CachedArtist[]>([]);
const songs = ref<CachedSong[]>([]);
const isLoading = ref(false);
const error = ref<string>();
const hasSearched = ref(false);
let searchVersion = 0;

const isModal = computed(() => props.variant === "modal");
const resultLimit = computed(() => (isModal.value ? 5 : 500));
const hasResults = computed(() => albums.value.length > 0 || artists.value.length > 0 || songs.value.length > 0);

const clearResults = () => {
  albums.value = [];
  artists.value = [];
  songs.value = [];
  error.value = undefined;
  hasSearched.value = false;
  isLoading.value = false;
};

const resetSearch = () => {
  searchVersion += 1;
  query.value = "";
  clearResults();
};

const search = async (searchQuery: string, version: number) => {
  hasSearched.value = true;

  try {
    const [albumResults, artistResults, songResults] = await Promise.all([
      invoke<CachedAlbum[]>("search_cached_albums", { query: searchQuery, limit: resultLimit.value }),
      invoke<CachedArtist[]>("search_cached_artists", { query: searchQuery, limit: resultLimit.value }),
      invoke<CachedSong[]>("search_cached_songs", { query: searchQuery, limit: resultLimit.value }),
    ]);

    if (version !== searchVersion) {
      return;
    }

    albums.value = albumResults;
    artists.value = artistResults;
    songs.value = songResults;
  } catch (cause) {
    if (version !== searchVersion) {
      return;
    }

    error.value = cause instanceof Error ? cause.message : String(cause || "Search failed.");
    albums.value = [];
    artists.value = [];
    songs.value = [];
  } finally {
    if (version === searchVersion) {
      isLoading.value = false;
    }
  }
};

const submitSearch = () => {
  const trimmedQuery = query.value.trim();
  if (!trimmedQuery) {
    if (!isModal.value) {
      void router.replace({ name: "search" });
      resetSearch();
    }
    return;
  }

  if (isModal.value) {
    void router.push({ name: "search", query: { q: trimmedQuery } });
    emit("navigate");
    return;
  }

  if (route.query.q === trimmedQuery) {
    const version = ++searchVersion;
    isLoading.value = true;
    error.value = undefined;
    void search(trimmedQuery, version);
    return;
  }

  void router.replace({ name: "search", query: { q: trimmedQuery } });
};

watch(query, (value) => {
  if (!isModal.value) {
    return;
  }

  searchVersion += 1;
  if (!value.trim()) {
    clearResults();
    return;
  }

  isLoading.value = true;
  error.value = undefined;
});

watchDebounced(
  query,
  (value) => {
    if (!isModal.value) {
      return;
    }

    const trimmedQuery = value.trim();
    if (trimmedQuery) {
      void search(trimmedQuery, searchVersion);
    }
  },
  { debounce: 300 },
);

watch(
  () => route.query.q,
  (routeQuery) => {
    if (isModal.value) {
      return;
    }

    const nextQuery = (Array.isArray(routeQuery) ? routeQuery[0] : routeQuery)?.trim() ?? "";
    query.value = nextQuery;
    const version = ++searchVersion;
    if (!nextQuery) {
      clearResults();
      return;
    }

    isLoading.value = true;
    error.value = undefined;
    void search(nextQuery, version);
  },
  { immediate: true },
);

watch(
  () => props.isOpen,
  (isOpen) => {
    if (isModal.value && !isOpen) {
      resetSearch();
    }
  },
);
</script>

<template>
  <div :class="isModal ? 'flex min-h-0 flex-1 flex-col' : 'space-y-8'">
    <div v-if="isModal" class="shrink-0 space-y-5 border-b border-zinc-800 p-4 lg:p-8">
      <h2 class="font-serif text-xl font-semibold text-zinc-100">Search</h2>
      <SearchSectionSearchForm v-model="query" placeholder="Search..." @submit="submitSearch" />
    </div>

    <SearchSectionSearchForm
      v-else
      v-model="query"
      placeholder="Search albums, artists and tracks"
      @submit="submitSearch"
    />

    <div v-if="query.trim()" :class="isModal ? 'min-h-0 flex-1 overflow-y-auto p-4 lg:p-8' : ''">
      <p v-if="isLoading" class="font-sans text-sm text-zinc-400">Searching...</p>
      <p v-else-if="error" class="font-sans text-sm text-red-400">{{ error }}</p>
      <SearchSectionResults
        v-else-if="hasSearched && hasResults"
        :albums="albums"
        :artists="artists"
        :songs="songs"
        :compact="isModal"
        @select="isModal && $emit('navigate')"
      />
      <p v-else-if="hasSearched" class="font-sans text-sm text-zinc-400">No results found.</p>
    </div>
  </div>
</template>
