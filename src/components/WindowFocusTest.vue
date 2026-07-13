<script setup lang="ts">
import { usePageLeave, useIdle, useWindowFocus } from "@vueuse/core";
import { computed } from "vue";

const hasLeftThePage = usePageLeave();
const isIdle = useIdle(5 * 1000);
const isFocused = useWindowFocus();

const showPage = computed(() => {
  const conditions = [hasLeftThePage.value, isIdle.idle.value, !isFocused.value];
  return conditions.filter(Boolean).length >= 2;
});
</script>

<template>
  <div
    v-if="showPage"
    class="bg-opacity-80 fixed inset-x-0 top-0 z-50 flex h-12 items-center justify-center bg-black text-white"
  >
    <div class="text-center">
      <h1 class="mb-4 text-2xl font-bold">You have left the page</h1>
      <p class="mb-4">Please return to the page to continue using the application.</p>
    </div>
  </div>
</template>
