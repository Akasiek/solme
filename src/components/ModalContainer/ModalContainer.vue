<script setup lang="ts">
import { nextTick, onBeforeUnmount, useTemplateRef, watch } from "vue";

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

const dialog = useTemplateRef<HTMLDialogElement>("dialog");

const closeDialog = () => {
  if (!props.show && dialog.value?.open) {
    dialog.value.close();
  }
};

watch(
  () => props.show,
  async (isOpen) => {
    if (!isOpen) {
      return;
    }

    await nextTick();

    if (dialog.value && !dialog.value.open) {
      dialog.value.showModal();
    }
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  if (dialog.value?.open) {
    dialog.value.close();
  }
});
</script>

<template>
  <Teleport to="body">
    <dialog
      ref="dialog"
      class="fixed inset-0 m-0 h-full max-h-none w-full max-w-none border-0 bg-transparent p-0 backdrop:bg-transparent"
      :aria-label="props.label"
      @cancel.prevent="emit('close')"
      @keydown.esc.prevent.stop="emit('close')"
    >
      <Transition name="fade" appear @after-leave="closeDialog">
        <div
          v-if="props.show"
          class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm"
          @click.self="emit('close')"
        >
          <slot />
        </div>
      </Transition>
    </dialog>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
