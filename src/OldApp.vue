<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";
</script>

<template>
  <header>
    <h1>Solme</h1>
    <nav>
      <RouterLink :to="{ name: 'home' }">Home</RouterLink>
      |
      <RouterLink :to="{ name: 'search' }">Search</RouterLink>
      |
      <RouterLink :to="{ name: 'player' }">Player</RouterLink>
    </nav>
  </header>

  <main>
    <RouterView />
  </main>
</template>
