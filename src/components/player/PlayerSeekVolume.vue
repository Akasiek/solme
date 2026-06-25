<script setup lang="ts">
import { computed, ref, watch } from "vue";

import { formatTime } from "@/utils/format";

const props = withDefaults(
  defineProps<{
    position?: number;
    duration?: number;
    volume?: number;
    disabled?: boolean;
    embedded?: boolean;
  }>(),
  {
    position: 0,
    duration: 0,
    volume: 0,
    disabled: false,
    embedded: false,
  },
);

const emit = defineEmits<{
  seek: [position: number];
  volume: [volume: number];
}>();

const seekValue = ref(props.position);
const volumeValue = ref(props.volume);
const isSeeking = ref(false);
const isChangingVolume = ref(false);

const safeDuration = computed(() => Math.max(0, props.duration));

watch(
  () => props.position,
  (position) => {
    if (!isSeeking.value) {
      seekValue.value = position;
    }
  },
);

watch(
  () => props.volume,
  (volume) => {
    if (!isChangingVolume.value) {
      volumeValue.value = volume;
    }
  },
);

function beginSeeking() {
  isSeeking.value = true;
}

function finishSeeking() {
  emit("seek", seekValue.value);
  isSeeking.value = false;
}

function beginChangingVolume() {
  isChangingVolume.value = true;
}

function updateVolume() {
  emit("volume", volumeValue.value);
}

function finishChangingVolume() {
  emit("volume", volumeValue.value);
  isChangingVolume.value = false;
}

function changeVolumeWithWheel(event: WheelEvent) {
  const direction = event.deltaY < 0 ? 1 : -1;
  volumeValue.value = Math.min(100, Math.max(0, volumeValue.value + direction * 2));
  emit("volume", volumeValue.value);
}
</script>

<template>
  <section
    class="control-deck grid w-full grid-cols-1 gap-4 p-3 sm:grid-cols-4"
    :class="{ 'is-embedded': embedded, 'max-w-2xl': !embedded }"
  >
    <div class="min-w-0 p-3 sm:col-span-3">
      <div class="mb-2 flex items-center justify-between">
        <span class="deck-label">Track position</span>
        <span class="time-readout">
          {{ formatTime(seekValue) }} / {{ formatTime(safeDuration) }}
        </span>
      </div>

      <div class="seek-slot relative flex h-8 items-center px-2">
        <input
          v-model.number="seekValue"
          type="range"
          min="0"
          :max="safeDuration"
          step="0.1"
          :disabled="disabled || safeDuration <= 0"
          aria-label="Track position"
          class="seek-input relative z-10 w-full"
          @pointerdown="beginSeeking"
          @change="finishSeeking"
        />
      </div>
    </div>

    <div
      class="volume-panel flex flex-col items-center justify-between p-3"
      @wheel.prevent="changeVolumeWithWheel"
    >
      <span class="deck-label">Volume</span>

      <input
        v-model.number="volumeValue"
        type="range"
        min="0"
        max="100"
        step="1"
        aria-label="Volume"
        class="volume-input w-full"
        @pointerdown="beginChangingVolume"
        @input="updateVolume"
        @change="finishChangingVolume"
      />
      <span class="volume-readout">{{ Math.round(volumeValue) }}</span>
    </div>
  </section>
</template>

<style scoped>
.control-deck {
  background-image:
    linear-gradient(to bottom, rgba(255, 255, 255, 0.3), rgba(0, 0, 0, 0.08)),
    url("/assets/images/texture_bg_2.webp");
  background-size:
    100% 100%,
    150%;
  border: 1px solid rgba(22, 22, 20, 0.95);
  border-radius: 3px;
  box-shadow:
    0 6px 9px rgba(0, 0, 0, 0.3),
    0 1px 0 rgba(255, 255, 255, 0.75),
    inset 0 1px 1px rgba(255, 255, 255, 0.8);
}

.control-deck.is-embedded {
  padding: 0.75rem 0 0;
  border: 0;
  border-top: 1px solid rgba(35, 35, 32, 0.7);
  border-radius: 0;
  background: none;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.5);
}

.volume-panel {
  border: 1px solid rgba(30, 30, 28, 0.75);
  background: rgba(215, 215, 208, 0.22);
  box-shadow:
    inset 0 1px 1px rgba(255, 255, 255, 0.65),
    inset 0 -1px 2px rgba(0, 0, 0, 0.24);
}

.deck-label {
  font-family: var(--font-display-condensed);
  font-size: 0.65rem;
  line-height: 1;
  letter-spacing: 0.12em;
  color: rgb(45 45 42);
  text-transform: uppercase;
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.65);
}

.time-readout,
.volume-readout {
  font-family: var(--font-led);
  color: rgb(22 101 52);
  text-shadow: 0 0 4px rgba(34, 197, 94, 0.28);
}

.time-readout {
  font-size: 0.78rem;
}

.volume-readout {
  margin-top: 0.2rem;
  font-size: 0.68rem;
}

.seek-slot {
  overflow: hidden;
  border: 1px solid rgb(12 13 12);
  border-radius: 2px;
  background: rgb(7 10 8);
  box-shadow:
    inset 0 3px 5px rgba(0, 0, 0, 0.95),
    0 1px 0 rgba(255, 255, 255, 0.5);
}

.seek-input,
.volume-input {
  height: 1rem;
  cursor: pointer;
  appearance: none;
  background: transparent;
}

.seek-input:disabled {
  cursor: default;
  opacity: 0.35;
}

.seek-input::-webkit-slider-runnable-track,
.volume-input::-webkit-slider-runnable-track {
  height: 3px;
  border-radius: 2px;
  background: rgb(44 45 42);
}

.seek-input::-webkit-slider-thumb,
.volume-input::-webkit-slider-thumb {
  width: 12px;
  height: 18px;
  margin-top: -8px;
  appearance: none;
  border: 1px solid rgb(30 30 28);
  border-radius: 2px;
  background: linear-gradient(to right, rgb(175 175 169), rgb(235 235 229), rgb(145 145 140));
  box-shadow:
    0 2px 3px rgba(0, 0, 0, 0.55),
    inset 0 1px 0 rgba(255, 255, 255, 0.75);
}
</style>
