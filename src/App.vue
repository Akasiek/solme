<script setup lang="ts">
import Player from "@/components/player/Player.vue";
</script>

<template>
  <main class="relative h-screen w-screen">
    <div class="player-background absolute inset-0 z-0 bg-center" />

    <Player/>
  </main>
</template>

<style scoped>
.player-background {
  background-image: url("/assets/images/texture_bg_2.webp");
  background-size: 50%;
}
</style>
