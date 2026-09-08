<script setup lang="ts">
import { ChevronLeft, ChevronRight } from "@lucide/vue";
import Button from "@/components/Button.vue";

defineProps<{ page: number; pageCount: number; label: string }>();
const emit = defineEmits<{ change: [page: number] }>();
</script>

<template>
  <nav v-if="pageCount > 1" :aria-label="label" class="flex items-center justify-center gap-3 pt-2">
    <Button variant="outline" :disabled="page === 1" aria-label="Previous page" @click="emit('change', page - 1)">
      <ChevronLeft class="size-4" /> Previous
    </Button>
    <span class="min-w-24 text-center font-sans text-sm text-zinc-400">{{ page }} / {{ pageCount }}</span>
    <Button variant="outline" :disabled="page === pageCount" aria-label="Next page" @click="emit('change', page + 1)">
      Next <ChevronRight class="size-4" />
    </Button>
  </nav>
</template>
