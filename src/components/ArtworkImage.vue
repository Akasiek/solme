<script setup lang="ts">
import { artworkSource } from "@/utils/artwork";

defineProps<{
  path?: string;
  alt: string;
}>();
</script>

<template>
  <img crossorigin="anonymous" :src="artworkSource(path)" :alt="alt" />
</template>
