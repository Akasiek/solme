<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { computed, useAttrs, useId } from "vue";

defineProps<{ label: string }>();
const model = defineModel<string>({ required: true });
const attrs = useAttrs();
const generatedId = useId();
const selectId = computed(() => (typeof attrs.id === "string" ? attrs.id : generatedId));
const selectAttrs = computed(() => {
  const attributes = { ...attrs };
  delete attributes.class;
  return attributes;
});

defineOptions({ inheritAttrs: false });
</script>

<template>
  <div class="min-w-48 space-y-2" :class="$attrs.class">
    <label :for="selectId" class="flex items-center gap-1.5 font-sans text-sm font-medium text-zinc-300">
      <slot name="icon" />{{ label }}
    </label>
    <div class="relative">
      <select
        :id="selectId"
        v-model="model"
        v-bind="selectAttrs"
        class="w-full cursor-pointer appearance-none rounded-md border border-zinc-700 bg-zinc-950 py-2 pr-10 pl-3 text-zinc-100 transition-colors outline-none hover:border-zinc-600 focus:border-zinc-500 focus:ring-2 focus:ring-zinc-700"
      >
        <slot />
      </select>
      <ChevronDown
        aria-hidden="true"
        class="pointer-events-none absolute top-1/2 right-3 size-4 -translate-y-1/2 text-zinc-400"
      />
    </div>
  </div>
</template>
