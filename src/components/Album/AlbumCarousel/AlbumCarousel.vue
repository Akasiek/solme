<script setup lang="ts">
import { useElementSize } from "@vueuse/core";
import { computed, ref, useTemplateRef, watch } from "vue";
import type { CachedAlbum } from "@/types.js";
import AlbumCard from "@/components/Album/AlbumCard/AlbumCard.vue";
import CarouselControlButton from "@/components/Album/AlbumCarousel/CarouselControlButton.vue";

const GAP_WIDTH = 16;

const props = defineProps<{
  title: string;
  albums: CachedAlbum[];
}>();

const viewportRef = useTemplateRef<HTMLElement>("viewport");
const { width: viewportWidth } = useElementSize(viewportRef);
const pageIndex = ref(0);
const navigationDirection = ref<"next" | "previous">("next");
const shouldAnimatePage = ref(false);

const page = computed(() => {
  const minCardWidth = viewportWidth.value >= 1024 ? 220 : viewportWidth.value >= 768 ? 200 : 180;
  const size = Math.max(1, Math.floor((viewportWidth.value + GAP_WIDTH) / (minCardWidth + GAP_WIDTH)));
  const maxIndex = Math.max(0, Math.ceil(props.albums.length / size) - 1);
  const start = Math.min(pageIndex.value * size, Math.max(0, props.albums.length - size));
  const albums = props.albums.slice(start, start + size);

  return {
    albums,
    gridStyle: {
      gridTemplateColumns: `repeat(${Math.max(1, albums.length)}, minmax(0, 1fr))`,
    },
    key: albums.map((album) => album.remoteId).join(":"),
    maxIndex,
    size,
  };
});
const carouselTransitionName = computed(() =>
  shouldAnimatePage.value ? `album-carousel-${navigationDirection.value}` : "album-carousel-none",
);

function changePage(direction: "next" | "previous") {
  navigationDirection.value = direction;
  shouldAnimatePage.value = true;
  const offset = direction === "next" ? 1 : -1;
  pageIndex.value = Math.min(page.value.maxIndex, Math.max(0, pageIndex.value + offset));
}

watch(viewportWidth, () => {
  shouldAnimatePage.value = false;
  pageIndex.value = Math.min(pageIndex.value, page.value.maxIndex);
});

watch(
  () => props.albums.length,
  () => {
    shouldAnimatePage.value = false;
    pageIndex.value = 0;
  },
);
</script>

<template>
  <section class="space-y-3">
    <div class="flex items-center justify-between gap-4">
      <h2 class="font-serif text-2xl font-bold">{{ title }}</h2>
      <div v-if="viewportWidth > 0 && albums.length > page.size" class="flex shrink-0 items-center gap-2">
        <CarouselControlButton
          :is-disabled="pageIndex === 0"
          :handle-click="() => changePage('previous')"
          variant="previous"
        />
        <CarouselControlButton
          :is-disabled="pageIndex === page.maxIndex"
          :handle-click="() => changePage('next')"
          variant="next"
        />
      </div>
    </div>
    <p v-if="albums.length === 0" class="text-zinc-400">No albums to show.</p>
    <div v-else ref="viewport" class="relative overflow-hidden">
      <div class="relative overflow-hidden">
        <Transition v-if="viewportWidth > 0" :name="carouselTransitionName">
          <div :key="page.key" class="grid gap-x-2 gap-y-4" :style="page.gridStyle">
            <div v-for="album in page.albums" :key="album.remoteId" class="min-w-0">
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
