<script setup lang="ts">
import { Play } from "@lucide/vue";
import { ref } from "vue";
import ContextMenu from "@/components/ContextMenu";
import { FavoriteButton, RatingStars } from "@/components/LibraryItemAnnotations";
import { CachedSong } from "@/types.ts";
import { formatTime } from "@/utils/format.ts";

const { song, trackNumber, showArtist, isCurrent, isLoading } = defineProps<{
  song: CachedSong;
  trackNumber: number;
  showArtist: boolean;
  isCurrent: boolean;
  isLoading: boolean;
}>();

defineEmits<{
  play: [song: CachedSong];
}>();

const contextMenu = ref<{ x: number; y: number }>();

const openContextMenu = (event: MouseEvent) => {
  contextMenu.value = { x: event.clientX, y: event.clientY };
};
</script>

<template>
  <div
    class="relative grid w-full grid-cols-[3rem_minmax(0,1fr)_4rem] items-center gap-4 rounded-md px-4 py-2.5 font-sans focus-within:bg-zinc-800 hover:bg-zinc-800 @min-[48rem]:grid-cols-[3rem_minmax(0,1fr)_11rem_4rem]"
    :class="
      isCurrent
        ? 'bg-zinc-800/50 shadow-inner shadow-black/20 before:absolute before:top-2 before:bottom-2 before:left-0 before:w-1 before:rounded-r before:bg-accent'
        : ''
    "
    @contextmenu.prevent="openContextMenu"
  >
    <button
      type="button"
      class="group contents text-left focus-visible:outline-none disabled:cursor-wait disabled:opacity-70"
      :disabled="isLoading"
      :title="`Play ${song.title}`"
      @click="$emit('play', song)"
    >
      <span class="grid size-6 place-items-center text-sm text-zinc-500 tabular-nums">
        <Play class="hidden size-4 text-zinc-100 group-hover:block group-focus-visible:block" aria-hidden="true" />
        <span class="group-hover:hidden group-focus-visible:hidden">
          {{ trackNumber }}
        </span>
      </span>

      <span class="mb-0.5 flex min-w-0 items-center gap-2">
        <span class="truncate font-serif text-sm font-medium" :class="isCurrent ? 'text-accent' : 'text-zinc-100'">
          {{ song.title }}
        </span>
        <span v-if="showArtist" class="truncate text-xs text-zinc-500" :title="song.artistName">
          {{ song.artistName }}
        </span>
      </span>

      <span class="col-start-3 text-right text-sm text-zinc-500 tabular-nums @min-[48rem]:col-start-4">
        {{ formatTime(song.durationSeconds) }}
      </span>
    </button>

    <div class="col-start-3 row-start-1 hidden items-center gap-2 @min-[48rem]:flex">
      <RatingStars :item="song" />
      <FavoriteButton :item="song" />
    </div>

    <ContextMenu
      :open="Boolean(contextMenu)"
      :x="contextMenu?.x ?? 0"
      :y="contextMenu?.y ?? 0"
      :aria-label="`${song.title} actions`"
      @close="contextMenu = undefined"
    >
      <div class="w-52 p-1">
        <p class="mb-2 truncate text-xs font-medium text-zinc-400" :title="song.title">{{ song.title }}</p>
        <div class="flex items-center gap-2">
          <RatingStars :item="song" />
          <FavoriteButton :item="song" />
        </div>
      </div>
    </ContextMenu>
  </div>
</template>
