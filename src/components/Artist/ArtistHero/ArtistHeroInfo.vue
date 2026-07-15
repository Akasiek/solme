<script setup lang="ts">
import type { CachedArtist } from "@/types";

defineProps<{
  artist: CachedArtist;
}>();
</script>

<template>
  <div
    class="z-0 -mt-8 rounded-lg border border-zinc-800 bg-zinc-950/70 p-7 pt-12 shadow-2xl shadow-black/25 backdrop-blur md:mt-0 md:-ml-16 md:w-104 md:p-9 md:pl-20 lg:-ml-20 lg:w-lg lg:p-12 lg:pl-24 xl:w-xl"
  >
    <div class="min-w-0 space-y-3">
      <p class="font-sans text-sm font-semibold tracking-wide text-accent uppercase">Artist</p>
      <h1
        class="line-clamp-3 text-4xl leading-tight font-bold wrap-break-word text-white md:text-5xl"
        :title="artist.name"
      >
        {{ artist.name }}
      </h1>
      <p class="font-sans text-base text-zinc-300">{{ artist.albumCount }} albums</p>
    </div>
  </div>
</template>
