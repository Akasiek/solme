<script setup lang="ts">
import { ref } from "vue";

import AlbumBrowser from "@/components/browser/AlbumBrowser.vue";
import Player from "@/components/player/Player.vue";

type ActiveView = "browser" | "player";

const activeView = ref<ActiveView>("browser");
</script>

<template>
  <main class="relative flex h-screen w-screen flex-col overflow-hidden">
    <div class="player-background absolute inset-0 z-0 bg-center" />

    <nav class="relative z-20 flex shrink-0 justify-center border-b border-black/70 px-4 pt-4">
      <div class="view-tabs flex w-full max-w-5xl p-1">
        <button
          type="button"
          class="view-tab flex-1 py-3 font-display-condensed text-sm tracking-widest uppercase"
          :class="{ 'is-active': activeView === 'browser' }"
          @click="activeView = 'browser'"
        >
          Browse
        </button>
        <button
          type="button"
          class="view-tab flex-1 py-3 font-display-condensed text-sm tracking-widest uppercase"
          :class="{ 'is-active': activeView === 'player' }"
          @click="activeView = 'player'"
        >
          Now playing
        </button>
      </div>
    </nav>

    <div class="relative z-10 flex min-h-0 flex-1 justify-center p-4">
      <AlbumBrowser v-show="activeView === 'browser'" @play="activeView = 'player'" />
      <Player v-show="activeView === 'player'" />
    </div>
  </main>
</template>

<style scoped>
.player-background {
  background-image: url("/assets/images/texture_bg_2.webp");
  background-size: 50%;
}

.view-tabs {
  background-image:
    linear-gradient(to bottom, rgba(255, 255, 255, 0.42), rgba(0, 0, 0, 0.08)),
    url("/assets/images/texture_bg_2.webp");
  background-size:
    100% 100%,
    120%;
  box-shadow:
    0 3px 6px rgba(0, 0, 0, 0.35),
    inset 0 1px 1px rgba(255, 255, 255, 0.85);
}

.view-tab {
  color: rgb(39 39 42);
  border-right: 1px solid rgba(0, 0, 0, 0.3);
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.65);
}

.view-tab:last-child {
  border-right: 0;
}

.view-tab.is-active {
  color: rgb(74 222 128);
  background: rgb(10 12 11);
  box-shadow:
    inset 0 2px 5px rgba(0, 0, 0, 0.95),
    0 1px 0 rgba(255, 255, 255, 0.45);
  text-shadow: 0 0 5px rgba(34, 197, 94, 0.4);
}
</style>
