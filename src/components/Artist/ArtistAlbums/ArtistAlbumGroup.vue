<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { LoaderCircle, Play, SlidersHorizontal } from "@lucide/vue";
import { computed, ref } from "vue";
import FormSelect from "@/components/FormSelect.vue";
import type { AlbumPageSort, CachedAlbum } from "@/types";
import ArtistAlbumsGrid from "@/components/Artist/ArtistAlbums/ArtistAlbumsGrid.vue";

const props = defineProps<{
  title: string;
  albums: CachedAlbum[];
}>();

type ArtistAlbumSort = Exclude<AlbumPageSort, "artist">;
const sort = ref<ArtistAlbumSort>("newest");
const isPlaying = ref(false);
const playError = ref<string | null>(null);
const sortOptions = [
  { value: "newest", label: "Newest release" },
  { value: "oldest", label: "Oldest release" },
  { value: "title", label: "Album title" },
  { value: "recently-added", label: "Recently added" },
  { value: "recently-played", label: "Recently played" },
  { value: "most-played", label: "Most played" },
] as const;

const compareText = (first: string, second: string) => first.localeCompare(second, undefined, { sensitivity: "base" });

const compareMissingLast = (first?: string | null, second?: string | null, descending = false) => {
  if (!first) return second ? 1 : 0;
  if (!second) return -1;
  return (first < second ? -1 : first > second ? 1 : 0) * (descending ? -1 : 1);
};

const releaseDate = (album: CachedAlbum) =>
  album.originalReleaseDate ??
  album.releaseDate ??
  (album.year == null ? undefined : `${album.year.toString().padStart(4, "0")}-12-31`);

const compareAlbums = (first: CachedAlbum, second: CachedAlbum) => {
  let order = 0;
  switch (sort.value) {
    case "newest":
      order = compareMissingLast(releaseDate(first), releaseDate(second), true);
      break;
    case "oldest":
      order = compareMissingLast(releaseDate(first), releaseDate(second));
      break;
    case "recently-added":
      order = compareMissingLast(first.serverAddedAt, second.serverAddedAt, true);
      break;
    case "recently-played":
      order = compareMissingLast(first.lastPlayedAt, second.lastPlayedAt, true);
      break;
    case "most-played":
      order = second.playCount - first.playCount;
      break;
    case "title":
      break;
  }
  return order || compareText(first.name, second.name) || compareText(first.remoteId, second.remoteId);
};

const sortedAlbums = computed(() => [...props.albums].sort(compareAlbums));

const playAll = async () => {
  if (isPlaying.value) return;
  isPlaying.value = true;
  playError.value = null;
  try {
    await invoke("player_play_albums", { albumIds: sortedAlbums.value.map((album) => album.remoteId) });
  } catch (error) {
    playError.value = String(error);
  } finally {
    isPlaying.value = false;
  }
};
</script>

<template>
  <div class="space-y-3">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex items-center gap-3">
        <h3 class="mb-1 font-serif text-xl font-bold text-white">{{ title }}</h3>
        <p class="border-lg rounded bg-accent px-2 py-0.5 font-serif text-sm font-extrabold text-white">
          {{ albums.length }}
        </p>
      </div>
      <div class="flex items-center gap-2">
        <FormSelect v-model="sort" label="Sort by" :options="sortOptions" compact>
          <template #icon><SlidersHorizontal class="size-3.5 shrink-0 text-zinc-400" aria-hidden="true" /></template>
        </FormSelect>
        <button
          type="button"
          class="inline-flex h-7 shrink-0 cursor-pointer items-center justify-center gap-1.5 rounded-md bg-accent px-2 font-serif text-xs font-semibold text-white transition-colors hover:bg-accent/85 focus-visible:ring-2 focus-visible:ring-zinc-500 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="isPlaying"
          :title="`Play all in ${title}`"
          :aria-label="`Play all in ${title}`"
          @click="playAll"
        >
          <LoaderCircle v-if="isPlaying" class="size-3.5 animate-spin" aria-hidden="true" />
          <Play v-else class="size-3.5 fill-current" aria-hidden="true" />
          {{ isPlaying ? "Loading…" : "Play all" }}
        </button>
      </div>
    </div>

    <p v-if="playError" role="alert" class="font-sans text-sm text-red-400">{{ playError }}</p>
    <ArtistAlbumsGrid :albums="sortedAlbums" />
  </div>
</template>
