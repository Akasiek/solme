<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref, watch } from "vue";
import { RouterLink, useRoute, useRouter } from "vue-router";

import type { CachedAlbum } from "@/types";

const route = useRoute();
const router = useRouter();
const query = ref("");
const results = ref<CachedAlbum[]>([]);
const message = ref("");

async function submitSearch() {
  const value = query.value.trim();
  await router.push({
    name: "search",
    query: value ? { q: value } : {},
  });
}

async function loadResults(value: string) {
  if (!value) {
    results.value = [];
    message.value = "";
    return;
  }

  try {
    results.value = await invoke<CachedAlbum[]>("search_cached_albums", {
      query: value,
      limit: 100,
    });
    message.value = results.value.length === 0 ? "No albums found" : "";
  } catch (error) {
    message.value = String(error);
  }
}

watch(
  () => route.query.q,
  (routeQuery) => {
    const value = typeof routeQuery === "string" ? routeQuery : "";
    query.value = value;
    void loadResults(value);
  },
  { immediate: true },
);
</script>

<template>
  <h2>Search albums</h2>
  <form @submit.prevent="submitSearch">
    <label>
      Album or artist:
      <input v-model="query" type="search" autofocus />
    </label>
    <button type="submit">Search</button>
  </form>

  <p>{{ message }}</p>
  <ul>
    <li v-for="album in results" :key="album.remoteId">
      <RouterLink :to="{ name: 'album', params: { albumId: album.remoteId } }">
        {{ album.artistName }} - {{ album.name }}
        <span v-if="album.year">({{ album.year }})</span>
      </RouterLink>
    </li>
  </ul>
</template>
