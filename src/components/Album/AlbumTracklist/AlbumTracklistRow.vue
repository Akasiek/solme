<script setup lang="ts">
import { Play } from "@lucide/vue";
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
</script>

<template>
  <button
    type="button"
    class="group relative grid w-full grid-cols-[3rem_minmax(0,1fr)_4rem] items-center gap-4 rounded-md px-4 py-2.5 text-left font-sans hover:bg-zinc-800 focus-visible:bg-zinc-800 focus-visible:outline-none disabled:cursor-wait disabled:opacity-70"
    :class="
      isCurrent
        ? 'bg-zinc-800/50 shadow-inner shadow-black/20 before:absolute before:top-2 before:bottom-2 before:left-0 before:w-1 before:rounded-r before:bg-accent'
        : ''
    "
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

    <span class="text-right text-sm text-zinc-500 tabular-nums">
      {{ formatTime(song.durationSeconds) }}
    </span>
  </button>
</template>
