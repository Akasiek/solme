<script setup lang="ts">
import CdPlayerButton from "@/components/CdPlayerButton.vue";
import PlayIcon from "@/components/icons/PlayIcon.vue";
import StopIcon from "@/components/icons/StopIcon.vue";
import PrevIcon from "@/components/icons/PrevIcon.vue";
import NextIcon from "@/components/icons/NextIcon.vue";
import PauseIcon from "@/components/icons/PauseIcon.vue";
</script>

<template>
  <main class="relative h-screen w-screen">
    <div
      class="absolute inset-0 z-0 bg-center bg-[url('/assets/images/texture_bg_2.webp')] bg-size-[50%]"
    />

    <div class="relative z-10 flex h-full w-full items-center justify-center space-x-1">
      <CdPlayerButton label="Stop">
        <StopIcon />
      </CdPlayerButton>
      <CdPlayerButton label="Previous track">
        <PrevIcon />
      </CdPlayerButton>
      <CdPlayerButton label="Play">
        <PlayIcon />
      </CdPlayerButton>
      <CdPlayerButton label="Next track">
        <NextIcon />
      </CdPlayerButton>
      <CdPlayerButton label="Pause">
        <PauseIcon />
      </CdPlayerButton>
    </div>
  </main>
</template>
