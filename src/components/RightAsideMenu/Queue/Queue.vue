<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";

import QueueEmptyState from "@/components/RightAsideMenu/Queue/QueueEmptyState.vue";
import QueueErrorState from "@/components/RightAsideMenu/Queue/QueueErrorState.vue";
import QueueHeader from "@/components/RightAsideMenu/Queue/QueueHeader.vue";
import QueueItem from "@/components/RightAsideMenu/Queue/QueueItem.vue";
import { useQueueReorder } from "@/composables/useQueueReorder";
import { usePlayerStore } from "@/stores/player";

const playerStore = usePlayerStore();
const queueList = ref<HTMLElement | null>(null);

const queue = computed(() => playerStore.queue);
const isQueueLoading = computed(() => playerStore.isQueueLoading);
const queueError = computed(() => playerStore.queueError);
const currentIndex = computed(() => {
  const position = playerStore.queueStatus?.queuePosition;
  return (position ?? 0) - 1;
});

const reorderDisabled = computed(() => playerStore.queueRevision === null || playerStore.status?.state === "loading");
const {
  start,
  cancel,
  draggedIndex,
  dropBoundary,
  isReordering,
  isSaving,
  error: reorderError,
  announcement,
  moveWithKeyboard,
} = useQueueReorder({
  list: queueList,
  revision: computed(() => playerStore.queueRevision),
  currentIndex,
  disabled: reorderDisabled,
  move: playerStore.moveQueueEntry,
});

const scrollCurrentSongIntoView = async (index: number) => {
  if (index < 0 || isReordering.value) {
    return;
  }

  await nextTick();
  if (isReordering.value) return;
  queueList.value?.querySelector<HTMLElement>(`[data-queue-index="${index}"]`)?.scrollIntoView({
    behavior: "smooth",
    block: "center",
    inline: "nearest",
  });
};

watch(
  () => playerStore.status?.queuePosition,
  () => cancel(),
  { flush: "sync" },
);

watch([currentIndex, queue], ([index]) => scrollCurrentSongIntoView(index), { immediate: true });

onMounted(() => {
  playerStore.startListening().catch((error) => {
    console.error("Failed to start listening for player changes:", error);
  });
});
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col">
    <QueueHeader :songCount="queue.length" />

    <div v-if="isQueueLoading && queue.length === 0" class="grid min-h-0 flex-1 place-items-center px-6">
      <p class="font-sans text-sm text-zinc-400">Loading queue...</p>
    </div>

    <QueueErrorState v-else-if="queueError && queue.length === 0" @retry="playerStore.refreshQueue" />

    <QueueEmptyState v-else-if="queue.length === 0" />

    <p
      v-if="reorderError"
      role="alert"
      class="mx-4 mt-2 rounded bg-red-950/40 px-3 py-2 font-sans text-xs text-red-300"
    >
      {{ reorderError }}
    </p>
    <p class="sr-only" role="status">{{ announcement }}</p>

    <div v-if="queue.length > 0" ref="queueList" class="min-h-0 flex-1 overflow-y-auto px-2 py-3">
      <p v-if="queueError" class="mx-2 mb-2 rounded bg-red-950/40 px-3 py-2 font-sans text-xs text-red-300">
        The queue could not be refreshed.
      </p>

      <QueueItem
        v-for="(song, index) in queue"
        :key="playerStore.queueEntryIds[index]"
        :song="song"
        :index="index"
        :isCurrent="index === currentIndex"
        :songCount="queue.length"
        :isDragged="index === draggedIndex"
        :dropBefore="dropBoundary === index"
        :dropAfter="index === queue.length - 1 && dropBoundary === queue.length"
        :reorderDisabled="reorderDisabled || isSaving"
        @reorderStart="start"
        @reorderCancel="cancel"
        @reorderKeyboard="moveWithKeyboard"
      />
    </div>
  </section>
</template>
