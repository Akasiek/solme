<script setup lang="ts">
import { ArrowLeft, ArrowRight } from "@lucide/vue";
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";

defineProps<{
  isCollapsed: boolean;
}>();

interface NavigationHistoryState {
  back?: string | null;
  forward?: string | null;
}

const route = useRoute();
const router = useRouter();

const historyState = () => window.history.state as NavigationHistoryState | null;

const canGoBack = computed(() => route.fullPath.length > 0 && historyState()?.back != null);
const canGoForward = computed(() => route.fullPath.length > 0 && historyState()?.forward != null);
</script>

<template>
  <nav class="mb-4 flex gap-2" :class="isCollapsed ? 'flex-col' : 'flex-row'" aria-label="Page history">
    <button
      type="button"
      class="nav-button"
      :class="isCollapsed ? 'w-full justify-center px-4' : 'min-w-0 flex-1 justify-center gap-2 px-2 text-sm'"
      :disabled="!canGoBack"
      title="Go back"
      aria-label="Go back"
      @click="router.back()"
    >
      <ArrowLeft class="size-5 shrink-0" aria-hidden="true" />
    </button>
    <button
      type="button"
      class="nav-button"
      :class="isCollapsed ? 'w-full justify-center px-4' : 'min-w-0 flex-1 justify-center gap-2 px-2 text-sm'"
      :disabled="!canGoForward"
      title="Go forward"
      aria-label="Go forward"
      @click="router.forward()"
    >
      <ArrowRight class="size-5 shrink-0" aria-hidden="true" />
    </button>
  </nav>
</template>

<style scoped>
@reference "@/style/glob.css";

.nav-button {
  @apply flex cursor-pointer items-center rounded border border-zinc-800/60 bg-zinc-800/20 py-2 font-medium text-zinc-100 transition-colors hover:bg-zinc-800 disabled:cursor-not-allowed disabled:border-zinc-800/40 disabled:bg-transparent disabled:text-zinc-600 disabled:hover:bg-transparent;
}
</style>
