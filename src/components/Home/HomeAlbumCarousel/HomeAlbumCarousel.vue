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
const navigationDirection = ref<"next" | "previous">("next");
const shouldAnimatePage = ref(false);
const hasMeasuredViewport = ref(false);
let resizeObserver: ResizeObserver | undefined;

const maxPageIndex = computed(() => Math.max(0, Math.ceil(props.albums.length / visibleCount.value) - 1));
const startIndex = computed(() => Math.min(pageIndex.value * visibleCount.value, maxStartIndex.value));
const maxStartIndex = computed(() => Math.max(0, props.albums.length - visibleCount.value));
const visibleAlbums = computed(() => props.albums.slice(startIndex.value, startIndex.value + visibleCount.value));
const visibleColumnCount = computed(() => Math.max(1, visibleAlbums.value.length));
const carouselGridStyle = computed(() => ({
  gridTemplateColumns: `repeat(${visibleColumnCount.value}, minmax(0, 1fr))`,
}));
const carouselPageKey = computed(() => visibleAlbums.value.map((album) => album.remoteId).join(":"));
const carouselTransitionName = computed(() =>
  shouldAnimatePage.value ? `album-carousel-${navigationDirection.value}` : "album-carousel-none",
);
const hasControls = computed(() => props.albums.length > visibleCount.value);
const canGoPrevious = computed(() => startIndex.value > 0);
const canGoNext = computed(() => startIndex.value < maxStartIndex.value);

function updateVisibleCount() {
  const viewportWidth = viewportRef.value?.clientWidth ?? 0;

  if (viewportWidth === 0) {
    return false;
  }

  const gapWidth = 16;
  const minCardWidth = viewportWidth >= 1024 ? 220 : viewportWidth >= 768 ? 200 : 180;
  visibleCount.value = Math.max(1, Math.floor((viewportWidth + gapWidth) / (minCardWidth + gapWidth)));
  return true;
}

async function measureViewport() {
  await nextTick();

  if (!viewportRef.value) {
    return;
  }

  if (updateVisibleCount()) {
    hasMeasuredViewport.value = true;
  }

  if (!resizeObserver) {
    resizeObserver = new ResizeObserver(() => {
      shouldAnimatePage.value = false;
      if (updateVisibleCount()) {
        hasMeasuredViewport.value = true;
      }
    });
    resizeObserver.observe(viewportRef.value);
  }
}

function goPrevious() {
  navigationDirection.value = "previous";
  shouldAnimatePage.value = true;
  pageIndex.value = Math.max(0, pageIndex.value - 1);
}

function goNext() {
  navigationDirection.value = "next";
  shouldAnimatePage.value = true;
  pageIndex.value = Math.min(maxPageIndex.value, pageIndex.value + 1);
}

watch(
  () => props.albums.length,
  async (albumCount) => {
    shouldAnimatePage.value = false;
    hasMeasuredViewport.value = false;
    pageIndex.value = 0;

    if (albumCount > 0) {
      await measureViewport();
    }
  },
);

watch([visibleCount, maxPageIndex], () => {
  shouldAnimatePage.value = false;
  pageIndex.value = Math.min(pageIndex.value, maxPageIndex.value);
});

onMounted(() => {
  if (props.albums.length > 0) {
    void measureViewport();
  }
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
});
</script>

<template>
  <section class="space-y-3">
    <div class="flex items-center justify-between gap-4">
      <h2 class="font-serif text-2xl font-bold">{{ title }}</h2>
      <div v-if="hasMeasuredViewport && hasControls" class="flex shrink-0 items-center gap-2">
        <CarouselControlButton :is-disabled="!canGoPrevious" :handle-click="goPrevious" variant="previous" />
        <CarouselControlButton :is-disabled="!canGoNext" :handle-click="goNext" variant="next" />
      </div>
    </div>
    <p v-if="albums.length === 0" class="text-zinc-400">No albums to show.</p>
    <div v-else ref="viewport" class="relative overflow-hidden">
      <div class="relative overflow-hidden">
        <Transition v-if="hasMeasuredViewport" :name="carouselTransitionName">
          <div :key="carouselPageKey" class="grid gap-4" :style="carouselGridStyle">
            <div v-for="album in visibleAlbums" :key="album.remoteId" class="min-w-0">
              <AlbumCard :album="album" />
            </div>
          </div>
        </Transition>
      </div>
    </div>
  </section>
</template>

<style scoped>
.album-carousel-next-enter-active,
.album-carousel-next-leave-active,
.album-carousel-previous-enter-active,
.album-carousel-previous-leave-active {
  transition:
    opacity 120ms ease-out,
    transform 200ms ease-out;
}

.album-carousel-next-leave-active,
.album-carousel-previous-leave-active {
  position: absolute;
  inset: 0;
  width: 100%;
}

.album-carousel-next-enter-from {
  opacity: 0;
  transform: translateX(32px);
}

.album-carousel-next-leave-to {
  opacity: 0;
  transform: translateX(-32px);
}

.album-carousel-previous-enter-from {
  opacity: 0;
  transform: translateX(-32px);
}

.album-carousel-previous-leave-to {
  opacity: 0;
  transform: translateX(32px);
}
</style>
