<script setup lang="ts">
import { nextTick, onUnmounted, ref, watch } from "vue";

import { preventDefaultAndStopPropagation } from "@/composables/useHotkey";

const props = withDefaults(
  defineProps<{
    show: boolean;
    label?: string;
  }>(),
  {
    label: "Dialog",
  },
);

const emit = defineEmits<{
  close: [];
}>();

const dialog = ref<HTMLElement | null>(null);
let previouslyFocusedElement: HTMLElement | null = null;

const focusableSelector =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

const getFocusableElements = () => {
  return dialog.value ? Array.from(dialog.value.querySelectorAll<HTMLElement>(focusableSelector)) : [];
};

const focusDialog = () => {
  const [firstFocusableElement] = getFocusableElements();
  (firstFocusableElement ?? dialog.value)?.focus();
};

const close = () => {
  emit("close");
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === "Escape") {
    preventDefaultAndStopPropagation(event);
    close();
    return;
  }

  if (event.key !== "Tab") {
    return;
  }

  const focusableElements = getFocusableElements();
  const firstFocusableElement = focusableElements[0];
  const lastFocusableElement = focusableElements[focusableElements.length - 1];

  if (!firstFocusableElement || !lastFocusableElement) {
    event.preventDefault();
    dialog.value?.focus();
    return;
  }

  if (event.shiftKey && document.activeElement === firstFocusableElement) {
    event.preventDefault();
    lastFocusableElement.focus();
  } else if (!event.shiftKey && document.activeElement === lastFocusableElement) {
    event.preventDefault();
    firstFocusableElement.focus();
  }
};

const handleBackdropClick = (event: MouseEvent) => {
  if (event.target === event.currentTarget) {
    close();
  }
};

watch(
  () => props.show,
  async (isOpen) => {
    if (isOpen) {
      previouslyFocusedElement = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      window.addEventListener("keydown", handleKeydown);
      await nextTick();
      focusDialog();
      return;
    }

    window.removeEventListener("keydown", handleKeydown);
    previouslyFocusedElement?.focus();
    previouslyFocusedElement = null;
  },
  { immediate: true },
);

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="props.show"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm"
      @click="handleBackdropClick"
    >
      <div ref="dialog" role="dialog" aria-modal="true" :aria-label="props.label" tabindex="-1">
        <slot />
      </div>
    </div>
  </Teleport>
</template>
