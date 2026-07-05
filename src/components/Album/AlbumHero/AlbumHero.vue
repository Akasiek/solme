<script setup lang="ts">
import { computed } from "vue";
import { CachedAlbum } from "@/types.ts";
import AlbumHeroCoverArt from "@/components/Album/AlbumHero/AlbumHeroCoverArt.vue";
import dayjs from "dayjs";
import AlbumHeroPlayerButtons from "@/components/Album/AlbumHero/AlbumHeroPlayerButtons.vue";
import AlbumHeroMetadata from "@/components/Album/AlbumHero/AlbumHeroMetadata.vue";

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
    <div
      class="container mx-auto flex flex-col gap-10 rounded-md border-2 border-zinc-800 bg-zinc-900 p-10 shadow-xl lg:flex-row"
    >
      <AlbumHeroCoverArt :album="album" />

      <div class="my-auto min-w-0 flex-1 space-y-4">
        <div class="rounded border border-zinc-700 p-5">
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

        <AlbumHeroMetadata :album-genres="albumGenres" />

        <AlbumHeroPlayerButtons :album="album" />
      </div>
    </div>
  </section>
</template>
