<script setup lang="ts">
import SearchSection from "@/components/Search/SearchSection";
</script>

<template>
  <section class="container mx-auto space-y-8 p-6">
    <div class="space-y-2">
      <h1 class="font-serif text-4xl font-bold">Search</h1>
      <hr class="border-zinc-800" />
    </div>

    <SearchSection variant="view" />
  </section>
</template>
