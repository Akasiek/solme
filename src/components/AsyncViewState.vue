<script setup lang="ts">
defineProps<{
  isLoading: boolean;
  error?: string | null;
}>();
</script>

<template>
  <slot v-if="isLoading" name="loading">
    <p class="text-zinc-400">Loading...</p>
  </slot>

  <slot v-else-if="error" name="error" :error="error">
    <p class="text-red-400">{{ error }}</p>
  </slot>

  <slot v-else />
</template>
