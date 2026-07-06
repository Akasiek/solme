<script setup lang="ts">
import { CachedAlbumDetails } from "@/types.ts";
import AlbumHeroCoverArt from "@/components/Album/AlbumHero/AlbumHeroCoverArt.vue";
import AlbumHeroGenres from "@/components/Album/AlbumHero/AlbumHeroGenres.vue";
import AlbumHeroPlayerButtons from "@/components/Album/AlbumHero/AlbumHeroPlayerButtons.vue";
import AlbumHeroQualityBadge from "@/components/Album/AlbumHero/AlbumHeroQualityBadge.vue";
import AlbumHeroStats from "@/components/Album/AlbumHero/AlbumHeroStats.vue";

const { albumDetails } = defineProps<{
  albumDetails: CachedAlbumDetails;
}>();

const { album, genres, discCount, audioFormats } = albumDetails;
</script>

<template>
  <section class="my-8 px-8">
    <div
      class="container mx-auto grid items-center gap-10 rounded-lg border border-zinc-800 bg-zinc-900/70 p-8 shadow-2xl shadow-black/20 lg:grid-cols-[20rem_minmax(0,1fr)] lg:gap-12 lg:p-10"
    >
      <AlbumHeroCoverArt :album="album" />

      <div class="min-w-0 space-y-6">
        <div class="space-y-3">
          <AlbumHeroQualityBadge :audio-formats="audioFormats" />
          <h1 class="min-w-0 font-serif text-4xl leading-tight font-bold text-white lg:text-5xl">{{ album.name }}</h1>
          <p class="font-sans text-lg text-zinc-300">
            <RouterLink
              :to="{ name: 'artist', params: { artistId: album.artistId } }"
              :title="album.artistName"
              class="line-clamp-1 hover:text-white hover:underline"
            >
              {{ album.artistName }}
            </RouterLink>
          </p>
          <AlbumHeroStats :album="album" :disc-count="discCount" />
        </div>

        <AlbumHeroGenres :genres="genres" />

        <AlbumHeroPlayerButtons :album="album" />
      </div>
    </div>
  </section>
</template>
