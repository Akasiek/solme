<script setup lang="ts">
import { Check, ChevronDown } from "@lucide/vue";
import {
  SelectContent,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
} from "reka-ui";
import { computed, useId } from "vue";

const props = defineProps<{
  label: string;
  options: ReadonlyArray<{ value: string; label: string }>;
  id?: string;
}>();

const model = defineModel<string>({ required: true });
const generatedId = useId();
const selectId = computed(() => props.id ?? generatedId);
const selectedLabel = computed(() => props.options.find((option) => option.value === model.value)?.label ?? "");
</script>

<template>
  <div class="min-w-48 space-y-2">
    <label :for="selectId" class="flex items-center gap-1.5 font-serif text-sm font-bold text-zinc-300">
      <slot name="icon" />{{ label }}
    </label>
    <SelectRoot v-model="model">
      <SelectTrigger
        :id="selectId"
        class="flex w-full cursor-pointer items-center justify-between rounded-md border border-zinc-700 bg-zinc-950 py-2 pr-3 pl-3 text-left text-zinc-100 transition-colors outline-none hover:border-zinc-600 focus:border-zinc-500 focus:ring-2 focus:ring-zinc-700"
      >
        <SelectValue>{{ selectedLabel }}</SelectValue>
        <ChevronDown aria-hidden="true" class="size-4 shrink-0 text-zinc-400" />
      </SelectTrigger>
      <SelectPortal>
        <SelectContent
          position="popper"
          :side-offset="4"
          class="z-[110] min-w-[var(--reka-select-trigger-width)] overflow-hidden rounded-md border border-zinc-700 bg-zinc-950 p-1 text-zinc-100 shadow-xl"
        >
          <SelectViewport>
            <SelectItem
              v-for="option in options"
              :key="option.value"
              :value="option.value"
              class="relative flex cursor-pointer items-center rounded px-8 py-1.5 font-sans text-sm outline-none select-none data-[highlighted]:bg-zinc-800 data-[highlighted]:text-white"
            >
              <SelectItemIndicator class="absolute left-2 inline-flex items-center">
                <Check class="size-4" aria-hidden="true" />
              </SelectItemIndicator>
              <SelectItemText>{{ option.label }}</SelectItemText>
            </SelectItem>
          </SelectViewport>
        </SelectContent>
      </SelectPortal>
    </SelectRoot>
  </div>
</template>
