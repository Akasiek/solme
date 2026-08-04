<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted } from "vue";
import { usePlayerStore } from "@/stores/player.ts";
import { useHotkey } from "@/composables/useHotkey";
import PlayerBarTrackInfo from "@/components/PlayerBar/PlayerBarTrackInfo.vue";
import PlayerBarVolumeControl from "@/components/PlayerBar/PlayerBarVolumeControl.vue";
import PlayerBarPlaybackControl from "@/components/PlayerBar/PlayerBarPlaybackControl.vue";
import PlayerBarSeekBar from "@/components/PlayerBar/PlayerBarSeekBar.vue";
import PlayerBarRightAsideMenuToggleButton from "@/components/PlayerBar/PlayerBarRightAsideMenuToggleButton.vue";

const playerStore = usePlayerStore();
const playerStatus = computed(() => playerStore.status);
const currentSong = computed(() => playerStore.currentSong);

const togglePlayback = () => {
  const status = playerStatus.value;
  if (!status?.currentSong) {
    return;
  }
  if (status.state === "loading") {
    return;
  }

  const cmd = status.state === "playing" ? "player_pause" : "player_resume";
  invoke(cmd).catch((error) => {
    console.error(`Failed to ${status.state === "playing" ? "pause" : "resume"} player:`, error);
  });
};

useHotkey(" ", togglePlayback);

onMounted(async () => {
  await playerStore.startListening();
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
      class="grid h-24 shrink-0 grid-cols-[minmax(0,1fr)_28rem_minmax(0,1fr)] items-center gap-4 border-t border-zinc-800 bg-zinc-950 p-4 text-zinc-100"
    >
      <div class="h-16 w-96 max-w-full">
        <PlayerBarTrackInfo :currentSong="currentSong" />
      </div>
      <div class="grid justify-items-center gap-2">
        <PlayerBarPlaybackControl :playerStatus="playerStatus" />
        <PlayerBarSeekBar :playerStatus="playerStatus" />
      </div>
      <div class="flex w-96 max-w-full items-center justify-end gap-4 justify-self-end">
        <PlayerBarRightAsideMenuToggleButton />
        <PlayerBarVolumeControl :volume="playerStatus.volume" />
      </div>
    </nav>
  </Transition>
</template>
