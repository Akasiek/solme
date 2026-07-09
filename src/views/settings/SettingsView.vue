<script setup lang="ts">
const sections = [{ name: "Servers", route: { name: "settings-account" } }];
</script>

<template>
  <section class="space-y-6 p-6">
    <div class="space-y-2">
      <h1 class="font-serif text-4xl font-bold">Settings</h1>
      <hr class="border-zinc-800" />
    </div>

    <div class="grid gap-6 lg:grid-cols-[12rem_1fr]">
      <nav class="flex gap-2 lg:flex-col">
        <RouterLink
          v-for="section in sections"
          :key="section.name"
          :to="section.route"
          class="rounded px-3 py-2 text-sm font-medium text-zinc-300 hover:bg-zinc-800 hover:text-zinc-100"
          active-class="bg-zinc-800 text-zinc-100"
        >
          {{ section.name }}
        </RouterLink>
      </nav>

      <RouterView />
    </div>
  </section>
</template>
