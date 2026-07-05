<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from "vue";
import type { CachedAlbum } from "@/types.js";
import AlbumCard from "@/components/Album/AlbumCard/AlbumCard.vue";
import CarouselControlButton from "@/components/Home/HomeAlbumCarousel/CarouselControlButton.vue";

const props = defineProps<{
  title: string;
  albums: CachedAlbum[];
}>();

const viewportRef = useTemplateRef<HTMLElement>("viewport");
const pageIndex = ref(0);
const visibleCount = ref(1);
let resizeObserver: ResizeObserver | undefined;

const maxPageIndex = computed(() => Math.max(0, Math.ceil(props.albums.length / visibleCount.value) - 1));
const startIndex = computed(() => Math.min(pageIndex.value * visibleCount.value, maxStartIndex.value));
const maxStartIndex = computed(() => Math.max(0, props.albums.length - visibleCount.value));
const visibleAlbums = computed(() => props.albums.slice(startIndex.value, startIndex.value + visibleCount.value));
const visibleColumnCount = computed(() => Math.max(1, visibleAlbums.value.length));
const carouselGridStyle = computed(() => ({
  gridTemplateColumns: `repeat(${visibleColumnCount.value}, minmax(0, 1fr))`,
}));
const hasControls = computed(() => props.albums.length > visibleCount.value);
const canGoPrevious = computed(() => startIndex.value > 0);
const canGoNext = computed(() => startIndex.value < maxStartIndex.value);

function updateVisibleCount() {
  const viewportWidth = viewportRef.value?.clientWidth ?? 0;

  if (viewportWidth === 0) {
    visibleCount.value = 1;
    return;
  }

  const gapWidth = 16;
  const minCardWidth = viewportWidth >= 1024 ? 220 : viewportWidth >= 768 ? 200 : 180;
  visibleCount.value = Math.max(1, Math.floor((viewportWidth + gapWidth) / (minCardWidth + gapWidth)));
}

function goPrevious() {
  pageIndex.value = Math.max(0, pageIndex.value - 1);
}

function goNext() {
  pageIndex.value = Math.min(maxPageIndex.value, pageIndex.value + 1);
}

watch(
  () => props.albums.length,
  () => {
    pageIndex.value = 0;
  },
);

watch([visibleCount, maxPageIndex], () => {
  pageIndex.value = Math.min(pageIndex.value, maxPageIndex.value);
});

onMounted(async () => {
  await nextTick();
  updateVisibleCount();

  if (viewportRef.value) {
    resizeObserver = new ResizeObserver(updateVisibleCount);
    resizeObserver.observe(viewportRef.value);
  }
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
});
</script>

<template>
  <section class="space-y-3">
    <h2 class="font-serif text-2xl font-bold">{{ title }}</h2>
    <p v-if="albums.length === 0" class="text-zinc-400">No albums to show.</p>
    <div v-else ref="viewport" class="relative overflow-hidden">
      <CarouselControlButton
        v-if="hasControls"
        :is-disabled="!canGoPrevious"
        :handle-click="goPrevious"
        variant="previous"
      />
      <div class="grid gap-4" :style="carouselGridStyle">
        <div v-for="album in visibleAlbums" :key="album.remoteId" class="min-w-0">
          <AlbumCard :album="album" />
        </div>
      </div>
      <CarouselControlButton v-if="hasControls" :is-disabled="!canGoNext" :handle-click="goNext" variant="next" />
    </div>
  </section>
</template>
