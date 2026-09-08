<script setup lang="ts">
import { computed, useAttrs, useId, useSlots } from "vue";

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
const slots = useSlots();
const generatedId = useId();
const inputId = computed(() => (typeof attrs.id === "string" ? attrs.id : generatedId));
const inputAttrs = computed(() => {
  const inputAttributes = { ...attrs };
  delete inputAttributes.class;
  return inputAttributes;
});
</script>

<template>
  <div class="space-y-2" :class="$attrs.class">
    <label v-if="fieldLabel" :for="inputId" class="block font-sans text-sm font-medium text-zinc-300">
      {{ fieldLabel }}
    </label>
    <div class="relative">
      <span v-if="slots.leading" class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-zinc-500">
        <slot name="leading" />
      </span>
      <input
        :id="inputId"
        v-model="model"
        v-bind="inputAttrs"
        :type="type ?? 'text'"
        :placeholder="placeholder"
        class="w-full rounded border border-zinc-700 bg-zinc-950 py-2 pr-2 text-zinc-100 focus:border-zinc-500 focus:outline-none"
        :class="slots.leading ? 'pl-9' : 'pl-2'"
      />
    </div>
  </div>
</template>
