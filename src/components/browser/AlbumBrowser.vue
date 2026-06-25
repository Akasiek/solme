<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";

import type { CachedAlbum, LibrarySummary } from "@/types";
import { artworkSource } from "@/utils/artwork";

const emit = defineEmits<{
  play: [];
}>();

interface BrowserAlbum extends CachedAlbum {
  artworkSource?: string;
}

const PAGE_SIZE = 20;
const SEARCH_LIMIT = 30;
const SEARCH_DELAY_MS = 250;

const albums = shallowRef<BrowserAlbum[]>([]);
const searchResults = shallowRef<BrowserAlbum[]>([]);
const albumCount = ref(0);
const query = ref("");
const isLoading = ref(false);
const isSearching = ref(false);
const playingAlbumId = ref<string | null>(null);
const errorMessage = ref("");
const page = ref(0);
let searchTimer: ReturnType<typeof setTimeout> | undefined;
let searchRequestId = 0;

const isSearchActive = computed(() => query.value.trim().length > 0);
const visibleAlbums = computed(() => (isSearchActive.value ? searchResults.value : albums.value));

const canGoPrevious = computed(() => page.value > 0);
const canGoNext = computed(() => (page.value + 1) * PAGE_SIZE < albumCount.value);

function prepareAlbums(items: CachedAlbum[]): BrowserAlbum[] {
  return items.map((album) => ({
    ...album,
    artworkSource: artworkSource(album.artworkPath),
  }));
}

async function loadAlbums(targetPage = page.value) {
  if (isLoading.value) {
    return;
  }

  isLoading.value = true;
  errorMessage.value = "";

  try {
    const pageItems = await invoke<CachedAlbum[]>("get_cached_albums", {
      offset: targetPage * PAGE_SIZE,
      limit: PAGE_SIZE,
    });

    albums.value = prepareAlbums(pageItems);
    page.value = targetPage;
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    isLoading.value = false;
  }
}

function changePage(direction: -1 | 1) {
  const targetPage = page.value + direction;
  if (targetPage < 0) {
    return;
  }

  void loadAlbums(targetPage);
}

async function searchAlbums(value: string) {
  const requestId = ++searchRequestId;
  isSearching.value = true;
  errorMessage.value = "";

  try {
    const results = await invoke<CachedAlbum[]>("search_cached_albums", {
      query: value,
      limit: SEARCH_LIMIT,
    });

    if (requestId === searchRequestId) {
      searchResults.value = prepareAlbums(results);
    }
  } catch (error) {
    if (requestId === searchRequestId) {
      errorMessage.value = String(error);
    }
  } finally {
    if (requestId === searchRequestId) {
      isSearching.value = false;
    }
  }
}

async function playAlbum(album: BrowserAlbum) {
  if (playingAlbumId.value) {
    return;
  }

  playingAlbumId.value = album.remoteId;
  errorMessage.value = "";

  try {
    await invoke("play_album", {
      albumId: album.remoteId,
      startSongId: null,
    });
    emit("play");
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    playingAlbumId.value = null;
  }
}

watch(query, (value) => {
  if (searchTimer) {
    clearTimeout(searchTimer);
  }

  const normalizedQuery = value.trim();
  if (!normalizedQuery) {
    searchRequestId += 1;
    searchResults.value = [];
    isSearching.value = false;
    return;
  }

  searchResults.value = [];
  isSearching.value = true;
  searchTimer = setTimeout(() => {
    void searchAlbums(normalizedQuery);
  }, SEARCH_DELAY_MS);
});

onMounted(async () => {
  try {
    const summary = await invoke<LibrarySummary>("get_library_summary");
    albumCount.value = summary.albumCount;
  } catch (error) {
    errorMessage.value = String(error);
  }

  await loadAlbums();
});

onBeforeUnmount(() => {
  if (searchTimer) {
    clearTimeout(searchTimer);
  }
  searchRequestId += 1;
});
</script>

<template>
  <section class="browser-shell relative z-10 flex min-h-0 w-full max-w-5xl flex-1 flex-col p-2">
    <header class="flex items-center gap-4 border-b border-black/80 px-4 py-3">
      <div class="min-w-0 flex-1">
        <h1 class="font-display-condensed text-lg font-bold tracking-wide text-zinc-900 uppercase">
          Albums
        </h1>
        <p class="font-display-condensed text-xs tracking-wide text-zinc-600">
          {{ albumCount }} albums
        </p>
      </div>

      <label class="search-field flex w-64 items-center rounded-sm bg-zinc-950 px-3 py-2">
        <span class="sr-only">Search albums</span>
        <input
          v-model="query"
          type="search"
          placeholder="ALBUM OR ARTIST"
          class="w-full bg-transparent font-display-condensed text-sm tracking-wide text-green-500 outline-none placeholder:text-zinc-600"
        />
      </label>
    </header>

    <div class="browser-recess flex min-h-0 flex-1 flex-col bg-zinc-950 p-1">
      <div class="browser-panel min-h-0 flex-1 overflow-y-auto p-4">
        <div
          v-if="visibleAlbums.length"
          class="grid grid-cols-2 gap-x-4 gap-y-6 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5"
        >
          <button
            v-for="album in visibleAlbums"
            :key="album.remoteId"
            type="button"
            class="album-card group min-w-0 text-left"
            :disabled="playingAlbumId !== null"
            @click="playAlbum(album)"
          >
            <span
              class="album-artwork relative mb-2 block aspect-square overflow-hidden bg-zinc-900"
            >
              <img
                v-if="album.artworkSource"
                :src="album.artworkSource"
                :alt="`${album.name} cover`"
                loading="eager"
                decoding="async"
                class="h-full w-full object-cover"
              />
              <span
                v-else
                class="flex h-full items-center justify-center p-4 text-center font-display text-xs tracking-widest text-zinc-600 uppercase"
              >
                No cover
              </span>
              <span
                v-if="playingAlbumId === album.remoteId"
                class="absolute inset-0 flex items-center justify-center bg-black/70 font-display text-sm tracking-widest text-green-400 uppercase"
              >
                Loading
              </span>
            </span>

            <span class="block truncate font-display text-sm text-zinc-200">
              {{ album.name }}
            </span>
            <span class="block truncate font-display text-xs text-zinc-500">
              {{ album.artistName }}
              <template v-if="album.year"> · {{ album.year }}</template>
            </span>
          </button>
        </div>

        <div
          v-else-if="!isLoading && !isSearching"
          class="flex h-full min-h-48 items-center justify-center font-display-condensed text-sm tracking-widest text-zinc-600 uppercase"
        >
          {{ isSearchActive ? "No matching albums" : "Library is empty" }}
        </div>
      </div>

      <div
        v-if="!isSearchActive"
        class="flex shrink-0 items-center justify-center gap-4 border-t border-zinc-800 bg-zinc-950 px-4 py-3"
      >
        <button
          type="button"
          class="rounded-sm border border-zinc-700 bg-zinc-900 px-4 py-2 font-display-condensed text-xs tracking-widest text-zinc-300 uppercase hover:text-green-400 disabled:opacity-30"
          :disabled="!canGoPrevious || isLoading"
          @click="changePage(-1)"
        >
          Previous
        </button>
        <span class="font-display-condensed text-xs tracking-widest text-zinc-600">
          {{ page + 1 }}
        </span>
        <button
          type="button"
          class="rounded-sm border border-zinc-700 bg-zinc-900 px-4 py-2 font-display-condensed text-xs tracking-widest text-zinc-300 uppercase hover:text-green-400 disabled:opacity-30"
          :disabled="!canGoNext || isLoading"
          @click="changePage(1)"
        >
          Next
        </button>
      </div>
    </div>

    <p
      v-if="errorMessage"
      role="alert"
      class="px-4 py-2 font-display-condensed text-xs text-red-900"
    >
      {{ errorMessage }}
    </p>
  </section>
</template>

<style scoped>
.browser-shell {
  background-image:
    linear-gradient(to bottom, rgba(255, 255, 255, 0.3), rgba(0, 0, 0, 0.08)),
    url("/assets/images/texture_bg_2.webp");
  background-size:
    100% 100%,
    120%;
  box-shadow:
    0 5px 9px rgba(0, 0, 0, 0.32),
    0 1px 0 rgba(255, 255, 255, 0.75),
    inset 0 1px 1px rgba(255, 255, 255, 0.75);
}

.browser-recess {
  box-shadow:
    inset 0 3px 7px rgba(0, 0, 0, 0.95),
    0 1px 0 rgba(255, 255, 255, 0.55);
}

.browser-panel {
  overscroll-behavior: contain;
  background: rgb(9 12 10);
  scrollbar-color: rgb(63 63 70) rgb(9 9 11);
}

.search-field {
  box-shadow:
    inset 0 2px 4px rgba(0, 0, 0, 0.9),
    0 1px 0 rgba(255, 255, 255, 0.55);
}

.album-artwork {
  border: 1px solid rgb(39 39 42);
  background: rgb(24 24 27);
}

.album-card:focus-visible {
  outline: none;
}

.album-card:focus-visible .album-artwork {
  box-shadow:
    0 3px 7px rgba(0, 0, 0, 0.65),
    0 0 0 2px rgb(34 197 94);
}
</style>
