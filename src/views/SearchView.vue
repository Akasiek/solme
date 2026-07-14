<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import AlbumCard from "@/components/Album/AlbumCard/AlbumCard.vue";
import Button from "@/components/Button.vue";

import { CachedAlbum, CachedSong } from "@/types.ts";

const query = ref("");
const albums = ref<CachedAlbum[]>([]);
const songs = ref<CachedSong[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const hasSearched = ref(false);

async function search() {
  const trimmedQuery = query.value.trim();

  if (!trimmedQuery) {
    albums.value = [];
    songs.value = [];
    hasSearched.value = false;
    error.value = null;
    return;
  }

  isLoading.value = true;
  error.value = null;
  hasSearched.value = true;

  try {
    const [albumResults, songResults] = await Promise.all([
      invoke<CachedAlbum[]>("search_cached_albums", {
        query: trimmedQuery,
        limit: 24,
      }),
      invoke<CachedSong[]>("search_cached_songs", {
        query: trimmedQuery,
        limit: 24,
      }),
    ]);

    albums.value = albumResults;
    songs.value = songResults;
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : "Search failed.";
    albums.value = [];
    songs.value = [];
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <section class="space-y-6 p-6">
    <div class="space-y-2">
      <h1 class="font-serif text-4xl font-bold">Search</h1>
      <hr class="border-zinc-800" />
    </div>

    <form @submit.prevent="search" class="flex w-full flex-col gap-2 md:flex-row">
      <input
        v-model="query"
        type="search"
        placeholder="Search albums and songs"
        class="w-full rounded border border-zinc-700 bg-zinc-950 p-2 text-zinc-100 focus:border-zinc-500 focus:outline-none"
      />
      <Button type="submit">Search</Button>
    </form>

    <p v-if="isLoading">Searching...</p>
    <p v-else-if="error">{{ error }}</p>

    <div v-if="hasSearched && !isLoading && !error" class="space-y-8">
      <section class="space-y-3">
        <h2 class="font-serif text-2xl font-bold">Albums</h2>
        <p v-if="albums.length === 0" class="text-zinc-400">No albums found.</p>
        <div
          v-else
          class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-4 md:grid-cols-[repeat(auto-fill,minmax(200px,1fr))] lg:grid-cols-[repeat(auto-fill,minmax(220px,1fr))]"
        >
          <div v-for="album in albums" :key="album.remoteId" class="min-w-0">
            <AlbumCard :album="album" />
          </div>
        </div>
      </section>

      <section class="space-y-3">
        <h2 class="font-serif text-2xl font-bold">Songs</h2>
        <p v-if="songs.length === 0" class="text-zinc-400">No songs found.</p>
        <ul v-else>
          <li v-for="song in songs" :key="song.remoteId">
            <RouterLink :to="{ name: 'album', params: { albumId: song.albumId } }">
              {{ song.artistName }} - {{ song.title }} ({{ song.albumName }})
            </RouterLink>
          </li>
        </ul>
      </section>
    </div>
  </section>
</template>
