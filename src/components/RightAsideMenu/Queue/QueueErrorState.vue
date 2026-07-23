<script setup lang="ts">
defineEmits<{
  retry: [];
}>();
</script>

<template>
  <div class="grid min-h-0 flex-1 place-items-center px-6 text-center">
    <div class="space-y-3">
      <p class="font-sans text-sm text-red-400">Could not load the queue.</p>
      <button
        type="button"
        class="rounded-md border border-zinc-700 px-3 py-1.5 font-sans text-sm text-zinc-200 hover:bg-zinc-800"
        @click="$emit('retry')"
      >
        Try again
      </button>
    </div>
  </div>
</template>
