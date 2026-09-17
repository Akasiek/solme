<script setup lang="ts">
import { ChevronDown, ChevronUp } from "@lucide/vue";
import { NumberFieldDecrement, NumberFieldIncrement, NumberFieldInput, NumberFieldRoot } from "reka-ui";
import { useId } from "vue";

withDefaults(
  defineProps<{
    label: string;
    min?: number;
    max?: number;
    step?: number;
    placeholder?: string;
    disabled?: boolean;
  }>(),
  {
    min: undefined,
    max: undefined,
    step: 1,
    placeholder: undefined,
    disabled: false,
  },
);

const model = defineModel<number | null>({ required: true });
const inputId = useId();
</script>

<template>
  <NumberFieldRoot
    :id="inputId"
    v-model="model"
    :min="min"
    :max="max"
    :step="step"
    :disabled="disabled"
    :format-options="{ useGrouping: false }"
    disable-wheel-change
    class="block w-full max-w-full min-w-0 space-y-1 font-sans text-xs text-zinc-400"
    :class="disabled ? 'cursor-not-allowed opacity-50' : ''"
  >
    <label :for="inputId">{{ label }}</label>
    <div
      class="flex h-8 w-full max-w-full min-w-0 overflow-hidden rounded border border-zinc-700 bg-zinc-950 transition-colors focus-within:border-zinc-500"
    >
      <NumberFieldInput
        :placeholder="placeholder"
        class="w-0 min-w-0 flex-1 bg-transparent px-2 text-sm text-zinc-100 tabular-nums outline-none"
      />
      <div class="flex w-6 shrink-0 flex-col border-l border-zinc-700">
        <NumberFieldIncrement
          class="flex min-h-0 flex-1 cursor-pointer items-center justify-center text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-100 disabled:cursor-not-allowed"
          aria-label="Increase value"
        >
          <ChevronUp class="size-3" aria-hidden="true" />
        </NumberFieldIncrement>
        <NumberFieldDecrement
          class="flex min-h-0 flex-1 cursor-pointer items-center justify-center border-t border-zinc-700 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-100 disabled:cursor-not-allowed"
          aria-label="Decrease value"
        >
          <ChevronDown class="size-3" aria-hidden="true" />
        </NumberFieldDecrement>
      </div>
    </div>
  </NumberFieldRoot>
</template>
