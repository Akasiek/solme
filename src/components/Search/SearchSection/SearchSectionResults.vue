<script setup lang="ts">
import AlbumCard from "@/components/Album/AlbumCard";
import ArtistCard from "@/components/Artist/ArtistCard";
import SearchTrackRow from "@/components/Search/SearchTrackRow";
import type { CachedAlbum, CachedArtist, CachedSong } from "@/types";

const props = defineProps<{
  albums: CachedAlbum[];
  artists: CachedArtist[];
  songs: CachedSong[];
  compact: boolean;
}>();

defineOptions({ name: "SearchSectionResults" });

defineEmits<{
  select: [];
}>();

const compactGridClass =
  "grid w-full justify-start gap-4 pb-1 grid-flow-col auto-cols-[9rem] md:auto-cols-[11rem] lg:auto-cols-[13rem]";
const fullGridClass =
  "grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-4 md:grid-cols-[repeat(auto-fill,minmax(11rem,1fr))] lg:grid-cols-[repeat(auto-fill,minmax(13rem,1fr))]";
</script>

<template>
  <div class="space-y-10">
    <section v-if="albums.length" class="space-y-3">
      <h2 class="font-serif text-2xl font-bold">Albums</h2>
      <div v-if="compact" :class="compactGridClass">
        <AlbumCard v-for="album in albums" :key="album.remoteId" :album="album" @click="$emit('select')" />
      </div>
      <div v-else :class="fullGridClass">
        <AlbumCard v-for="album in albums" :key="album.remoteId" :album="album" />
      </div>
    </section>

    <section v-if="artists.length" class="space-y-3">
      <h2 class="font-serif text-2xl font-bold">Artists</h2>
      <div v-if="compact" :class="compactGridClass">
        <ArtistCard v-for="artist in artists" :key="artist.remoteId" :artist="artist" @select="$emit('select')" />
      </div>
      <div v-else :class="fullGridClass">
        <ArtistCard v-for="artist in artists" :key="artist.remoteId" :artist="artist" />
      </div>
    </section>

    <section v-if="songs.length" class="space-y-3">
      <h2 class="font-serif text-2xl font-bold">Tracks</h2>
      <div class="divide-y divide-zinc-800">
        <SearchTrackRow v-for="song in songs" :key="song.remoteId" :song="song" @select="compact && $emit('select')" />
      </div>
    </section>
  </div>
</template>
