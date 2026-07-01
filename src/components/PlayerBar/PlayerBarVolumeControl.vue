<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, ref, watch } from "vue";

const { volume } = defineProps<{ volume: number }>();

const currentVolume = ref(Math.round(volume));
const volumePercent = computed(() => Math.min(100, Math.max(0, currentVolume.value)));
const volumeTrackStyle = computed(() => ({
  background: `linear-gradient(to right, var(--color-zinc-200) 0%, var(--color-zinc-200) ${volumePercent.value}%, var(--color-zinc-600) ${volumePercent.value}%, var(--color-zinc-600) 100%)`,
}));
const volumeLabelStyle = computed(() => ({
  left: `calc(0.5rem + ${volumePercent.value} * (100% - 1rem) / 100)`,
}));

watch(
  () => volume,
  (newVolume) => {
    currentVolume.value = Math.round(newVolume);
  },
);

const onChange = async (event: Event) => {
  const target = event.target as HTMLInputElement;
  const newVolume = parseInt(target.value);

  await invokeSetNewVolume(newVolume);
};

const onScrollWheel = async (event: Event) => {
  event.preventDefault();
  const delta = (event as WheelEvent).deltaY;
  const newVolume = Math.min(100, Math.max(0, currentVolume.value - Math.sign(delta) * 5));

  await invokeSetNewVolume(newVolume);
};

const invokeSetNewVolume = async (newVolume: number) => {
  currentVolume.value = newVolume;
  try {
    await invoke("player_set_volume", { volume: newVolume });
  } catch (error) {
    console.error("Failed to set player volume:", error);
    currentVolume.value = Math.round(volume);
  }
};
</script>

<template>
  <div class="volume-slider-wrapper">
    <span class="volume-slider-label" :style="volumeLabelStyle">{{ currentVolume }}</span>
    <input
      type="range"
      min="0"
      max="100"
      :value="currentVolume"
      class="volume-slider"
      :style="volumeTrackStyle"
      @input="onChange"
      @wheel="onScrollWheel"
    />
  </div>
</template>

<style scoped>
@reference "@/style/glob.css";

.volume-slider-wrapper {
  @apply relative flex h-4 w-48 items-center;
}

.volume-slider-label {
  @apply pointer-events-none absolute bottom-full mb-2 -translate-x-1/2 rounded bg-zinc-800 px-1.5 py-0.5 text-xs leading-none font-medium text-zinc-100 tabular-nums opacity-0 shadow-md transition-opacity;
}

.volume-slider-wrapper:hover .volume-slider-label,
.volume-slider-wrapper:focus-within .volume-slider-label {
  @apply opacity-100;
}

.volume-slider {
  @apply h-1.5 w-full cursor-pointer appearance-none rounded-full;
}

.volume-slider::-webkit-slider-thumb {
  @apply size-4 appearance-none rounded-full bg-accent shadow-[0_0_12px_#00000060];
}

.volume-slider::-moz-range-thumb {
  @apply size-4 rounded-full border-0 bg-accent shadow-[0_0_12px_#00000060];
}
</style>
