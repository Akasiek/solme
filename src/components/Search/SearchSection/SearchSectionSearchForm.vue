<script setup lang="ts">
import { Search } from "@lucide/vue";

import Button from "@/components/Button.vue";
import TextInput from "@/components/TextInput.vue";

defineProps<{
  placeholder: string;
}>();

defineOptions({ name: "SearchSectionSearchForm" });

const query = defineModel<string>({ required: true });

defineEmits<{
  submit: [];
}>();
</script>

<template>
  <form class="flex w-full items-end gap-2" @submit.prevent="$emit('submit')">
    <TextInput v-model="query" type="search" :placeholder="placeholder" aria-label="Search" class="min-w-0 flex-1" />
    <Button type="submit" title="Show all search results" aria-label="Show all search results" class="shrink-0 py-2.25">
      <Search class="size-5" aria-hidden="true" />
      Search
    </Button>
  </form>
</template>
