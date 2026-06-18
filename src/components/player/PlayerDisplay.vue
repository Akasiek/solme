<script setup lang="ts">
import PlayerDisplayMarquee from "@/components/player/PlayerDisplayMarquee.vue";

withDefaults(
  defineProps<{
    track?: number;
    elapsed?: string;
    title?: string;
    artist?: string;
    album?: string;
    state?: "playing" | "paused" | "stopped";
  }>(),
  {
    track: 1,
    elapsed: "0:00",
    title: "NO DISC",
    artist: "",
    album: "",
    state: "stopped",
  },
);
</script>

<template>
  <section
    class="display-bezel w-full max-w-lg rounded-sm border border-black/90 p-2"
    aria-label="CD player display"
  >
    <div class="display-recess rounded-xs bg-zinc-950 p-1">
      <div
        class="display-glass relative isolate flex h-24 items-center overflow-hidden rounded-xs border border-black px-3 py-2 text-green-400 sm:px-4"
      >
        <div
          class="flex shrink-0 self-stretch flex-col items-center justify-center border-r border-green-900/30 pr-3"
        >
          <span
            class="display-label font-display-condensed text-xs leading-none tracking-wider text-green-700 uppercase"
          >
            TRK
          </span>
          <span
            class="track-number mt-2 font-led-italic text-xl leading-none tracking-wider text-green-400"
          >
            {{ String(track).padStart(2, "0") }}
          </span>
        </div>

        <div
          class="flex min-w-0 flex-1 flex-col justify-center gap-2 overflow-hidden px-3 whitespace-nowrap sm:px-4"
        >
          <PlayerDisplayMarquee
            :text="title"
            class="metadata-title font-segment text-base leading-tight tracking-wider"
          />
          <PlayerDisplayMarquee
            v-if="album"
            :text="album"
            class="metadata-album font-segment-italic text-xs leading-tight tracking-wider text-green-600/85"
          />
          <PlayerDisplayMarquee
            v-if="artist"
            :text="artist"
            class="metadata-artist font-segment-italic text-xs leading-tight tracking-wider text-green-600/70"
          />
        </div>

        <div
          class="state-marker flex w-7 shrink-0 items-center justify-center text-green-700"
          :class="`is-${state}`"
          :aria-label="state"
        >
          <span class="play-marker" aria-hidden="true" />
        </div>

        <div class="flex min-w-24 shrink-0 items-center justify-end sm:min-w-28">
          <span
            class="segment font-led text-3xl leading-none tracking-wide tabular-nums sm:text-4xl"
          >
            {{ elapsed }}
          </span>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.display-bezel {
  background-image:
    linear-gradient(to bottom, rgba(255, 255, 255, 0.3), rgba(0, 0, 0, 0.08)),
    url("/assets/images/texture_bg_2.webp");
  background-size:
    100% 100%,
    190%;
  box-shadow:
    0 4px 7px rgba(0, 0, 0, 0.28),
    0 1px 2px rgba(0, 0, 0, 0.55),
    0 1px 0 rgba(255, 255, 255, 0.75),
    inset 0 1px 1px rgba(255, 255, 255, 0.8),
    inset 0 -1px 2px rgba(0, 0, 0, 0.4);
}

.display-recess {
  box-shadow:
    inset 0 3px 5px rgba(0, 0, 0, 0.95),
    inset 0 -1px 1px rgba(255, 255, 255, 0.2),
    0 1px 0 rgba(255, 255, 255, 0.55);
}

.display-glass {
  background:
    radial-gradient(circle at 50% 20%, rgba(22, 101, 52, 0.12), transparent 55%),
    linear-gradient(105deg, rgb(3 8 5), rgb(1 4 2) 55%, rgb(3 7 4));
  box-shadow:
    inset 0 0 18px rgba(0, 0, 0, 0.95),
    inset 0 1px 1px rgba(74, 222, 128, 0.12);
}

.display-glass::before {
  content: "";
  position: absolute;
  z-index: 2;
  inset: 0;
  pointer-events: none;
  background:
    linear-gradient(112deg, rgba(255, 255, 255, 0.055), transparent 28%),
    repeating-linear-gradient(to bottom, transparent 0, transparent 2px, rgba(0, 0, 0, 0.15) 3px);
  mix-blend-mode: screen;
}

.display-glass::after {
  content: "";
  position: absolute;
  z-index: 3;
  inset: 0;
  pointer-events: none;
  box-shadow:
    inset 0 0 1px rgba(134, 239, 172, 0.24),
    inset 0 0 12px rgba(0, 0, 0, 0.9);
}

.track-number {
  text-shadow:
    0 0 2px rgba(74, 222, 128, 0.75),
    0 0 6px rgba(34, 197, 94, 0.35);
}

.display-label {
  text-shadow: 0 0 5px rgba(34, 197, 94, 0.35);
}

.metadata-title {
  text-shadow: 0 0 5px rgba(34, 197, 94, 0.38);
}

.metadata-artist {
  text-shadow: 0 0 4px rgba(22, 163, 74, 0.3);
}

.metadata-album {
  text-shadow: 0 0 4px rgba(22, 163, 74, 0.3);
}

.play-marker {
  width: 0;
  height: 0;
  border-top: 0.34rem solid transparent;
  border-bottom: 0.34rem solid transparent;
  border-left: 0.52rem solid currentColor;
}

.state-marker:not(.is-playing) .play-marker {
  opacity: 0.22;
}

.state-marker.is-playing {
  color: rgb(74 222 128);
  text-shadow: 0 0 6px rgba(34, 197, 94, 0.5);
}

.state-marker.is-paused .play-marker {
  width: 0.48rem;
  height: 0.65rem;
  border: 0;
  border-right: 0.15rem solid currentColor;
  border-left: 0.15rem solid currentColor;
}

.segment {
  color: rgb(74 222 128);
  text-shadow:
    0 0 2px rgba(74, 222, 128, 0.9),
    0 0 7px rgba(34, 197, 94, 0.45);
}
</style>
