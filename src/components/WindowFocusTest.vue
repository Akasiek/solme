<script setup lang="ts">
import { usePageLeave, useIdle, useWindowFocus } from "@vueuse/core";
import {computed} from "vue";

const hasLeftThePage = usePageLeave();
const isIdle = useIdle(5 * 1000);
const isFocused = useWindowFocus();

const showPage = computed(() => {
  const conditions = [hasLeftThePage.value, isIdle.idle.value, !isFocused.value];
  return conditions.filter(Boolean).length >= 2;
});
</script>

<template>
  <div v-if="showPage" class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-80 text-white">
    <div class="text-center">
      <h1 class="text-2xl font-bold mb-4">You have left the page</h1>
      <p class="mb-4">Please return to the page to continue using the application.</p>
    </div>
  </div>
</template>
