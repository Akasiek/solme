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
  <section class="@container my-6 px-6">
    <div
      class="container mx-auto grid items-center gap-6 rounded-lg border border-zinc-800 bg-zinc-900/70 p-5 shadow-2xl shadow-black/20 @min-[36rem]:gap-8 @min-[36rem]:p-8 @min-[48rem]:grid-cols-[minmax(14rem,1fr)_minmax(0,2fr)] @min-[48rem]:gap-10 @min-[64rem]:gap-12 @min-[64rem]:p-10"
    >
      <AlbumHeroCoverArt :album="album" />

      <div class="min-w-0 space-y-4 @min-[48rem]:space-y-5 @min-[64rem]:space-y-6">
        <div class="space-y-2 @min-[64rem]:space-y-3">
          <AlbumHeroQualityBadge :audio-formats="audioFormats" />
          <h1
            class="min-w-0 font-serif text-3xl leading-tight font-bold text-white @min-[36rem]:text-4xl @min-[64rem]:text-5xl"
          >
            {{ album.name }}
          </h1>
          <p class="font-sans text-base text-zinc-300 @min-[64rem]:text-lg">
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
