<script setup lang="ts">
import { toRef } from "vue";
import PaginationControls from "@/components/PaginationControls.vue";
import { usePagination } from "@/composables/usePagination";

const props = withDefaults(
  defineProps<{ total: number; itemLabel: string; navigationLabel: string; pageSize?: number }>(),
  { pageSize: 24 },
);
const page = defineModel<number>("page", { required: true });
const { pageCount, resultRange, changePage } = usePagination(
  toRef(props, "total"),
  props.pageSize,
  props.itemLabel,
  page,
);
</script>

<template>
  <div class="@container min-w-0 space-y-5">
    <div class="flex items-center justify-between font-sans text-sm text-zinc-400">
      <p>{{ resultRange }}</p>
      <p v-if="pageCount > 1">Page {{ page }} of {{ pageCount }}</p>
    </div>
    <slot />
    <PaginationControls :page="page" :page-count="pageCount" :label="navigationLabel" @change="changePage" />
  </div>
</template>
