<script setup lang="ts">
defineProps<{
  modelValue: boolean;
  label: string;
  description?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const updateValue = (event: Event) => {
  emit("update:modelValue", (event.target as HTMLInputElement).checked);
};
</script>

<template>
  <label
    class="flex items-start justify-between gap-6 rounded border border-zinc-800 bg-zinc-900/30 p-4"
    :class="disabled ? 'cursor-not-allowed opacity-60' : 'cursor-pointer hover:border-zinc-700'"
  >
    <span class="min-w-0 space-y-1">
      <span class="block font-sans text-sm font-medium text-zinc-100">{{ label }}</span>
      <span v-if="description" class="block font-sans text-sm leading-5 text-zinc-400">{{ description }}</span>
    </span>

    <span class="relative mt-0.5 shrink-0">
      <input type="checkbox" class="peer sr-only" :checked="modelValue" :disabled="disabled" @change="updateValue" />
      <span
        class="block h-6 w-11 rounded-full bg-zinc-700 transition-colors peer-checked:bg-accent peer-focus-visible:ring-2 peer-focus-visible:ring-zinc-400 peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-zinc-950"
      />
      <span
        class="pointer-events-none absolute top-1 left-1 size-4 rounded-full bg-white transition-transform peer-checked:translate-x-5"
      />
    </span>
  </label>
</template>
