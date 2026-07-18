<script setup lang="ts">
import { computed, useAttrs } from "vue";

defineOptions({
  inheritAttrs: false,
});

defineProps<{
  fieldLabel?: string;
  type?: string;
  placeholder?: string;
}>();

const model = defineModel<string>({ required: true });
const attrs = useAttrs();
const inputAttrs = computed(() => {
  const inputAttributes = { ...attrs };
  delete inputAttributes.class;
  return inputAttributes;
});
</script>

<template>
  <label class="space-y-2" :class="$attrs.class">
    <span v-if="fieldLabel" class="block font-sans text-sm font-medium text-zinc-300">{{ fieldLabel }}</span>
    <input
      v-model="model"
      v-bind="inputAttrs"
      :type="type ?? 'text'"
      :placeholder="placeholder"
      class="w-full rounded border border-zinc-700 bg-zinc-950 p-2 text-zinc-100 focus:border-zinc-500 focus:outline-none"
    />
  </label>
</template>
