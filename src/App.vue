<script setup lang="ts">
import PlayerBar from "@/components/PlayerBar/PlayerBar.vue";
</script>

<template>
  <main class="bg-zinc-900 font-sans text-zinc-100">
    <RouterView />
  </main>
  <PlayerBar />
</template>
