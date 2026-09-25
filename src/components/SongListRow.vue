<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { Play } from "@lucide/vue";
import dayjs from "dayjs";
import { computed, ref } from "vue";
import MissingCoverImage from "@/components/Album/MissingCoverImage.vue";
import ContextMenu from "@/components/ContextMenu";
import { LibraryItemAnnotations } from "@/components/LibraryItemAnnotations";
import type { CachedSong, SongPageItem } from "@/types";
import { artworkSource } from "@/utils/artwork";
import { formatTime } from "@/utils/format";

const props = withDefaults(
  defineProps<{
    song: CachedSong & Partial<Pick<SongPageItem, "releaseDate" | "originalReleaseDate" | "year">>;
    showAlbum?: boolean;
    showArtist?: boolean;
    showCover?: boolean;
    trackNumber?: number;
    isCurrent?: boolean;
  }>(),
  { showAlbum: true, showArtist: true, showCover: true, isCurrent: false },
);
const emit = defineEmits<{ select: [] }>();
const isPlaying = ref(false);
const contextMenu = ref<{ x: number; y: number }>();

const releaseDate = computed(() => {
  const date = props.song.originalReleaseDate || props.song.releaseDate;
  if (!date) return props.song.year?.toString();
  if (/^\d{4}$/.test(date)) return date;
  if (/^\d{4}-\d{2}$/.test(date)) return dayjs(`${date}-01`).format("MMM YYYY");
  return dayjs(date).format("MMM D, YYYY");
});

async function playSong() {
  if (isPlaying.value) return;
  isPlaying.value = true;
  try {
    await invoke("player_play_album", { albumId: props.song.albumId, startSongId: props.song.remoteId });
  } catch (error) {
    console.error("Failed to play song", error);
  } finally {
    isPlaying.value = false;
  }
}

function playFromRow(event: MouseEvent) {
  if (event.target instanceof Element && !event.target.closest("button, a")) {
    void playSong();
  }
}
</script>

<template>
  <div
    class="group relative flex min-w-0 cursor-pointer items-center gap-3 px-3 py-2.5 font-sans transition-colors focus-within:bg-zinc-800/70 hover:bg-zinc-800/70"
    :class="
      isCurrent
        ? 'bg-zinc-800/50 before:absolute before:top-2 before:bottom-2 before:left-0 before:w-1 before:rounded-r before:bg-accent'
        : ''
    "
    :aria-current="isCurrent ? 'true' : undefined"
    @contextmenu.prevent="contextMenu = { x: $event.clientX, y: $event.clientY }"
    @click="playFromRow"
  >
    <button
      type="button"
      class="relative size-12 shrink-0 cursor-pointer overflow-hidden rounded border border-zinc-700 bg-zinc-950 disabled:cursor-wait disabled:opacity-50"
      :disabled="isPlaying"
      :aria-label="`Play ${song.title}`"
      @click="playSong"
    >
      <img
        v-if="showCover && song.artworkPath"
        :src="artworkSource(song.artworkPath)"
        :alt="`${song.albumName} artwork`"
        class="size-full object-cover"
      />
      <MissingCoverImage v-else-if="showCover" />
      <span v-else class="grid size-full place-items-center text-sm text-zinc-400 tabular-nums">{{
        trackNumber ?? song.trackNumber ?? "–"
      }}</span>
      <span
        class="absolute inset-0 grid place-items-center bg-black/65 opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100"
      >
        <Play class="size-5 fill-current text-white" aria-hidden="true" />
      </span>
    </button>
    <div class="min-w-0 flex-1">
      <button
        type="button"
        class="block max-w-full cursor-pointer truncate text-left font-serif font-semibold hover:underline focus-visible:underline"
        :class="isCurrent ? 'text-accent' : 'text-zinc-100'"
        :disabled="isPlaying"
        :title="`Play ${song.title}`"
        @click="playSong"
      >
        {{ song.title }}
      </button>
      <div v-if="showArtist || showAlbum" class="mt-0.5 flex min-w-0 items-center gap-1 text-xs text-zinc-400">
        <RouterLink
          v-if="showArtist && song.artistId"
          :to="{ name: 'artist', params: { artistId: song.artistId } }"
          class="truncate hover:text-white hover:underline"
          @click="emit('select')"
          >{{ song.artistName }}</RouterLink
        >
        <span v-else-if="showArtist" class="truncate">{{ song.artistName }}</span>
        <span v-if="showArtist && showAlbum" aria-hidden="true">·</span>
        <RouterLink
          v-if="showAlbum"
          :to="{ name: 'album', params: { albumId: song.albumId } }"
          class="truncate hover:text-white hover:underline"
          @click="emit('select')"
          >{{ song.albumName }}</RouterLink
        >
      </div>
    </div>
    <div class="hidden @min-[40rem]:block"><LibraryItemAnnotations :item="song" /></div>
    <div class="w-24 shrink-0 pl-2 text-right text-xs text-zinc-500 @min-[40rem]:w-32">
      <span class="block tabular-nums">{{ formatTime(song.durationSeconds) }}</span>
      <span v-if="releaseDate" class="mt-0.5 block" :title="`Released ${releaseDate}`">{{ releaseDate }}</span>
    </div>
    <button
      type="button"
      class="shrink-0 cursor-pointer rounded px-1 text-zinc-400 hover:bg-zinc-700 hover:text-white @min-[40rem]:hidden"
      :aria-label="`${song.title} actions`"
      @click="contextMenu = { x: $event.clientX, y: $event.clientY }"
    >
      ···
    </button>
    <ContextMenu
      :open="Boolean(contextMenu)"
      :x="contextMenu?.x ?? 0"
      :y="contextMenu?.y ?? 0"
      :aria-label="`${song.title} actions`"
      @close="contextMenu = undefined"
    >
      <div class="w-52 p-2"><LibraryItemAnnotations :item="song" /></div>
    </ContextMenu>
  </div>
</template>
