<script setup lang="ts">
defineProps<{
  discNumber: number;
}>();
</script>

<template>
  <div class="px-4 pt-5 pb-2 font-sans text-xs font-semibold tracking-wide text-zinc-500 uppercase">
    Disc {{ discNumber }}
  </div>
</template>
