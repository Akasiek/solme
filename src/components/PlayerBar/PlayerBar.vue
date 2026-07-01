<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { PlayerStatus } from "@/types.js";
import PlayerBarTrackInfo from "@/components/PlayerBar/PlayerBarTrackInfo.vue";
import PlayerBarVolumeControl from "@/components/PlayerBar/PlayerBarVolumeControl.vue";
import PlayerBarPlaybackControl from "@/components/PlayerBar/PlayerBarPlaybackControl.vue";
import PlayerBarSeekBar from "@/components/PlayerBar/PlayerBarSeekBar.vue";

const playerStatus = ref<PlayerStatus | null>(null);
let unlistenPlayerStatusChanged: (() => void) | null = null;
const currentSong = computed(() => playerStatus.value?.currentSong ?? null);

const loadPlayerStatus = async () => {
  playerStatus.value = await invoke<PlayerStatus>("get_player_status");
};

onMounted(async () => {
  await loadPlayerStatus();

  unlistenPlayerStatusChanged = await listen<PlayerStatus>("player-status-changed", (event) => {
    playerStatus.value = event.payload;
  });
});

onUnmounted(() => {
  unlistenPlayerStatusChanged?.();
});
</script>

<template>
  <Transition
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="translate-y-full opacity-0"
    enter-to-class="translate-y-0 opacity-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="translate-y-0 opacity-100"
    leave-to-class="translate-y-full opacity-0"
  >
    <nav
      v-if="playerStatus && currentSong"
      class="sticky inset-x-0 bottom-0 grid h-28 grid-cols-[minmax(0,1fr)_28rem_minmax(0,1fr)] items-center gap-4 bg-zinc-900 p-4 text-zinc-100 shadow-[0_0_24px_0_rgba(0,0,0,0.5)]"
    >
      <div class="h-20 w-96 max-w-full">
        <PlayerBarTrackInfo :currentSong="currentSong" />
      </div>
      <div class="grid justify-items-center gap-3">
        <PlayerBarPlaybackControl :playerStatus="playerStatus" />
        <PlayerBarSeekBar :playerStatus="playerStatus" />
      </div>
      <div class="flex w-96 max-w-full justify-end justify-self-end">
        <PlayerBarVolumeControl :volume="playerStatus.volume" />
      </div>
    </nav>
  </Transition>
</template>
