<script setup lang="ts">
import { computed } from "vue";
import { CachedAlbum } from "@/types.ts";
import AlbumHeroCoverArt from "@/components/Album/AlbumHero/AlbumHeroCoverArt.vue";
import dayjs from "dayjs";

const { album, genres } = defineProps<{
  album: CachedAlbum;
  genres: string[];
}>();

const albumDateFormat = (album: CachedAlbum) => {
  return dayjs(album.originalReleaseDate || album.releaseDate).format("MMMM D, YYYY");
};

const albumGenres = computed(() => genres.filter(Boolean));
</script>

<template>
  <section class="my-6 px-8">
    <div class="container mx-auto flex gap-10 rounded-lg border-2 border-zinc-800 bg-zinc-900 p-10 shadow-xl">
      <AlbumHeroCoverArt :album="album" />

      <div class="my-auto space-y-4">
        <div class="rounded-md border-2 border-zinc-800 p-5">
          <h1 class="font-serif text-3xl font-bold">{{ album.name }}</h1>

          <p class="my-2 font-sans text-zinc-300">
            <RouterLink
              :to="{ name: 'artist', params: { artistId: album.artistId } }"
              :title="album.artistName"
              class="line-clamp-1 font-sans text-sm text-zinc-300 hover:underline"
            >
              {{ album.artistName }}
            </RouterLink>
          </p>

          <p class="font-sans text-xs font-semibold tracking-wide text-zinc-500 uppercase">
            Released on {{ albumDateFormat(album) }}
          </p>
        </div>

        <div v-if="albumGenres.length > 0" class="space-y-2 rounded-md border-2 border-zinc-800 p-5">
          <h2 class="font-sans text-xs font-semibold tracking-wide text-zinc-500 uppercase">Genres</h2>
          <ul class="flex flex-wrap gap-2">
            <li
              v-for="genre in albumGenres"
              :key="genre"
              class="rounded-md border border-zinc-700 px-2.5 py-1 text-sm text-zinc-200"
            >
              {{ genre }}
            </li>
          </ul>
        </div>
      </div>
    </div>
  </section>
</template>
