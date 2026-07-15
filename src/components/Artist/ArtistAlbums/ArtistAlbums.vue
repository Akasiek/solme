<script setup lang="ts">
import { computed } from "vue";
import type { CachedAlbum } from "@/types";
import ArtistAlbumsGrid from "@/components/Artist/ArtistAlbums/ArtistAlbumsGrid.vue";

const props = defineProps<{
  albums: CachedAlbum[];
}>();

interface AlbumGroup {
  key: string;
  title: string;
  albums: CachedAlbum[];
}

const albumTypeOrder = new Map([
  ["album", 0],
  ["ep", 1],
  ["single", 2],
  ["live", 3],
  ["compilation", 4],
  ["remix", 5],
  ["soundtrack", 6],
  ["other", 99],
]);

const normalizedAlbumType = (albumType?: string) => {
  const types =
    albumType
      ?.split(/[;,/]/)
      .map((type) => type.trim())
      .filter(Boolean) ?? [];

  return types.find((type) => type.toLowerCase() !== "album") ?? types[0] ?? "";
};

const albumTypeLabel = (albumType?: string) => {
  const trimmed = normalizedAlbumType(albumType);
  const normalized = trimmed.toLowerCase();

  switch (normalized) {
    case "":
    case "album":
      return { key: "album", title: "Albums" };
    case "ep":
      return { key: "ep", title: "EPs" };
    case "single":
      return { key: "single", title: "Singles" };
    case "live":
      return { key: "live", title: "Live albums" };
    case "compilation":
      return { key: "compilation", title: "Compilations" };
    case "remix":
      return { key: "remix", title: "Remixes" };
    case "soundtrack":
      return { key: "soundtrack", title: "Soundtracks" };
    default:
      return { key: normalized, title: trimmed };
  }
};

const albumGroups = computed(() => {
  const groups = new Map<string, AlbumGroup>();

  for (const album of props.albums) {
    const { key, title } = albumTypeLabel(album.albumType);
    const group = groups.get(key) ?? { key, title, albums: [] };
    group.albums.push(album);
    groups.set(key, group);
  }

  return [...groups.values()].sort((first, second) => {
    const firstOrder = albumTypeOrder.get(first.key) ?? albumTypeOrder.get("other")!;
    const secondOrder = albumTypeOrder.get(second.key) ?? albumTypeOrder.get("other")!;
    return firstOrder - secondOrder || first.title.localeCompare(second.title);
  });
});
</script>

<template>
  <section class="px-6 pb-12">
    <div
      class="container mx-auto space-y-6 rounded-lg border border-zinc-800 bg-zinc-900/70 p-6 shadow-2xl shadow-black/20 lg:p-8"
    >
      <div class="min-w-0 space-y-1">
        <p class="font-sans text-sm font-semibold tracking-wide text-accent uppercase">Releases</p>
        <h2 class="font-serif text-2xl font-bold text-white">Discography</h2>
      </div>

      <p v-if="albums.length === 0" class="font-sans text-zinc-400">No albums to show.</p>
      <div v-else class="space-y-8">
        <div v-for="group in albumGroups" :key="group.key" class="space-y-3">
          <div class="flex items-center gap-3">
            <h3 class="mb-1 font-serif text-xl font-bold text-white">{{ group.title }}</h3>
            <p
              class="border-lg flex items-center justify-between rounded bg-accent px-2 py-0.5 font-serif text-sm font-extrabold text-white"
            >
              {{ group.albums.length }}
            </p>
          </div>
          <ArtistAlbumsGrid :albums="group.albums" />
        </div>
      </div>
    </div>
  </section>
</template>
