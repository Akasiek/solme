<script setup lang="ts">
defineProps<{
  isLoading: boolean;
  error?: string | null;
}>();
</script>

<template>
  <slot v-if="isLoading" name="loading">
    <div class="flex h-full w-full items-center justify-center">
      <p class="text-zinc-400">Loading...</p>
    </div>
  </slot>

  <slot v-else-if="error" name="error" :error="error">
    <p class="text-red-400">{{ error }}</p>
  </slot>

  <slot v-else />
</template>
