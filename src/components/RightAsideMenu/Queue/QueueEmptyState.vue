<script setup lang="ts">
import { ListMusic } from "@lucide/vue";
</script>

<template>
  <div class="grid min-h-0 flex-1 place-items-center px-6 text-center">
    <div class="space-y-2">
      <ListMusic class="mx-auto size-8 text-zinc-600" aria-hidden="true" />
      <p class="font-serif text-zinc-300">Your queue is empty</p>
      <p class="font-sans text-sm text-zinc-500">Play an album or add one to the queue.</p>
    </div>
  </div>
</template>
