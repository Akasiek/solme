<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";

const props = withDefaults(
  defineProps<{
    text?: string;
  }>(),
  {
    text: "",
  },
);

const viewport = ref<HTMLElement | null>(null);
const probe = ref<HTMLElement | null>(null);
const isOverflowing = ref(false);
const duration = ref(12);
const movementEnd = ref("80%");
let resizeObserver: ResizeObserver | undefined;

const SCROLL_SPEED_PX_PER_SECOND = 24;
const LOOP_PAUSE_SECONDS = 2;

const marqueeStyle = computed(() => ({
  "--marquee-duration": `${duration.value}s`,
  "--marquee-movement-end": movementEnd.value,
}));

function measure() {
  if (!viewport.value || !probe.value) {
    return;
  }

  const travelDistance = probe.value.scrollWidth + 32;
  const movementDuration = travelDistance / SCROLL_SPEED_PX_PER_SECOND;
  const totalDuration = movementDuration + LOOP_PAUSE_SECONDS;

  isOverflowing.value = probe.value.scrollWidth > viewport.value.clientWidth + 1;
  duration.value = totalDuration;
  movementEnd.value = `${(movementDuration / totalDuration) * 100}%`;
}

watch(
  () => props.text,
  async () => {
    await nextTick();
    measure();
  },
);

onMounted(async () => {
  resizeObserver = new ResizeObserver(measure);

  if (viewport.value) {
    resizeObserver.observe(viewport.value);
  }

  await document.fonts.ready;
  measure();
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
});
</script>

<template>
  <span ref="viewport" class="relative block min-w-0 overflow-hidden">
    <span
      v-if="isOverflowing"
      class="marquee-track flex w-max items-center gap-8"
      :style="marqueeStyle"
    >
      <span class="whitespace-nowrap">{{ text }}</span>
      <span class="whitespace-nowrap" aria-hidden="true">{{ text }}</span>
    </span>
    <span v-else class="block overflow-hidden text-ellipsis whitespace-nowrap">
      {{ text }}
    </span>

    <span
      ref="probe"
      class="pointer-events-none invisible absolute w-max whitespace-nowrap"
      aria-hidden="true"
    >
      {{ text }}
    </span>
  </span>
</template>

<style scoped>
.marquee-track {
  animation: cd-marquee var(--marquee-duration) 1s infinite;
  animation-timing-function: linear(0, 1 var(--marquee-movement-end), 1);
}

@keyframes cd-marquee {
  from {
    transform: translateX(0);
  }

  to {
    transform: translateX(calc(-50% - 1rem));
  }
}

@media (prefers-reduced-motion: reduce) {
  .marquee-track {
    animation: none;
  }
}
</style>
