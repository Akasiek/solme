<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { watchDebounced } from "@vueuse/core";
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";

import ModalContainer from "@/components/ModalContainer/ModalContainer.vue";
import { useHotkey } from "@/composables/useHotkey.ts";
import type { CachedAlbum, CachedArtist, CachedSong } from "@/types";
import SearchModalAlbumResults from "./SearchModalAlbumResults.vue";
import SearchModalArtistResults from "./SearchModalArtistResults.vue";
import SearchModalSearchForm from "./SearchModalSearchForm.vue";
import SearchModalTrackResults from "./SearchModalTrackResults.vue";

const show = defineModel<boolean>("show", { default: false });
const router = useRouter();
const query = ref("");
const albums = ref<CachedAlbum[]>([]);
const artists = ref<CachedArtist[]>([]);
const songs = ref<CachedSong[]>([]);
const isLoading = ref(false);
const error = ref<string>();
const hasSearched = ref(false);
let queryVersion = 0;

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
  queryVersion += 1;
  query.value = "";
  clearResults();
};

const closeModal = () => {
  show.value = false;
};

const showAllResults = () => {
  const trimmedQuery = query.value.trim();
  if (!trimmedQuery) {
    return;
  }

  void router.push({ name: "search", query: { q: trimmedQuery } });
  closeModal();
};

const search = async (searchQuery: string, version: number) => {
  hasSearched.value = true;

  try {
    const [albumResults, artistResults, songResults] = await Promise.all([
      invoke<CachedAlbum[]>("search_cached_albums", { query: searchQuery, limit: 5 }),
      invoke<CachedArtist[]>("search_cached_artists", { query: searchQuery, limit: 5 }),
      invoke<CachedSong[]>("search_cached_songs", { query: searchQuery, limit: 5 }),
    ]);

    if (version !== queryVersion) {
      return;
    }

    albums.value = albumResults;
    artists.value = artistResults;
    songs.value = songResults;
  } catch (cause) {
    if (version !== queryVersion) {
      return;
    }

    error.value = cause instanceof Error ? cause.message : String(cause || "Search failed.");
    albums.value = [];
    artists.value = [];
    songs.value = [];
  } finally {
    if (version === queryVersion) {
      isLoading.value = false;
    }
  }
};

watch(query, (value) => {
  queryVersion += 1;
  const trimmedQuery = value.trim();

  if (!trimmedQuery) {
    clearResults();
    return;
  }

  isLoading.value = true;
  error.value = undefined;
});

watchDebounced(
  query,
  (value) => {
    const trimmedQuery = value.trim();
    if (!trimmedQuery) {
      return;
    }

    void search(trimmedQuery, queryVersion);
  },
  { debounce: 300 },
);

watch(show, (isOpen) => {
  if (!isOpen) {
    resetSearch();
  }
});

useHotkey(
  (event) => event.key.toLowerCase() === "s" && event.shiftKey,
  () => (show.value = true),
);
</script>

<template>
  <ModalContainer :show="show" label="Search" @close="closeModal">
    <div
      class="flex max-h-[calc(100dvh-4rem)] w-[min(80rem,calc(100vw-4rem))] min-w-0 flex-col overflow-hidden rounded border border-zinc-800 bg-zinc-900 text-zinc-100 shadow-2xl shadow-black/40"
    >
      <div class="shrink-0 space-y-5 border-b border-zinc-800 p-4 lg:p-8">
        <h2 class="font-serif text-xl font-semibold text-zinc-100">Search</h2>
        <SearchModalSearchForm v-model="query" @submit="showAllResults" />
      </div>

      <div v-if="query.trim()" class="min-h-0 flex-1 overflow-y-auto p-4 lg:p-8">
        <p v-if="isLoading" class="font-sans text-sm text-zinc-400">Searching...</p>
        <p v-else-if="error" class="font-sans text-sm text-red-400">{{ error }}</p>

        <div v-else-if="hasSearched && hasResults" class="space-y-10">
          <SearchModalAlbumResults v-if="albums.length" :albums="albums" @select="closeModal" />
          <SearchModalArtistResults v-if="artists.length" :artists="artists" @select="closeModal" />
          <SearchModalTrackResults v-if="songs.length" :songs="songs" @select="closeModal" />
        </div>

        <p v-else-if="hasSearched" class="font-sans text-sm text-zinc-400">No results found.</p>
      </div>
    </div>
  </ModalContainer>
</template>
