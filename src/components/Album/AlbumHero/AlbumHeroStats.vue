<script setup lang="ts">
import { computed } from "vue";
import dayjs from "dayjs";
import { CachedAlbum } from "@/types.ts";
import { formatDuration } from "@/utils/format.ts";

const { album, discCount } = defineProps<{
  album: CachedAlbum;
  discCount: number;
}>();

const albumDateFormat = (album: CachedAlbum) => {
  const date = album.originalReleaseDate || album.releaseDate;
  if (!date) {
    return album.year?.toString();
  }

  if (/^\d{4}$/.test(date)) {
    return date;
  }

  if (/^\d{4}-\d{2}$/.test(date)) {
    return dayjs(`${date}-01`).format("MMMM YYYY");
  }

  return dayjs(date).format("MMMM D, YYYY");
};

const albumDuration = computed(() => formatDuration(album.durationSeconds));
const discLabel = computed(() => {
  if (discCount <= 1) {
    return undefined;
  }

  return `${discCount} Discs`;
});
const releaseDateLabel = computed(() => albumDateFormat(album));
</script>

<template>
  <div class="space-y-1 font-sans">
    <p class="text-xs font-semibold text-zinc-300 @min-[64rem]:text-sm">
      <template v-if="discLabel">{{ discLabel }} ·</template>
      {{ album.songCount }} Tracks · {{ albumDuration }}
    </p>
    <p
      v-if="releaseDateLabel"
      class="text-xxs font-semibold tracking-wide text-zinc-500 uppercase @min-[64rem]:text-xs"
    >
      Released on {{ releaseDateLabel }}
    </p>
  </div>
</template>
