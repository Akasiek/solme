<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";

const props = defineProps<{
  open: boolean;
  x: number;
  y: number;
  ariaLabel?: string;
}>();

const emit = defineEmits<{ close: [] }>();
const menu = ref<HTMLElement>();
const position = ref({ left: 0, top: 0 });
const viewportPadding = 8;

const close = () => {
  if (props.open) emit("close");
};

const updatePosition = async () => {
  if (!props.open) return;

  await nextTick();
  const width = menu.value?.offsetWidth ?? 0;
  const height = menu.value?.offsetHeight ?? 0;

  position.value = {
    left: Math.max(viewportPadding, Math.min(props.x, window.innerWidth - width - viewportPadding)),
    top: Math.max(viewportPadding, Math.min(props.y, window.innerHeight - height - viewportPadding)),
  };

  menu.value?.focus({ preventScroll: true });
};

const closeOnOutsidePointer = (event: PointerEvent) => {
  if (props.open && !menu.value?.contains(event.target as Node)) close();
};

const closeOnEscape = (event: KeyboardEvent) => {
  if (event.key === "Escape") close();
};

watch(() => [props.open, props.x, props.y], updatePosition);

onMounted(() => {
  document.addEventListener("pointerdown", closeOnOutsidePointer);
  document.addEventListener("keydown", closeOnEscape);
  window.addEventListener("blur", close);
  window.addEventListener("resize", close);
});

onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", closeOnOutsidePointer);
  document.removeEventListener("keydown", closeOnEscape);
  window.removeEventListener("blur", close);
  window.removeEventListener("resize", close);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-show="open"
      ref="menu"
      role="menu"
      tabindex="-1"
      :aria-label="ariaLabel"
      class="fixed z-50 min-w-48 rounded-md border border-zinc-700 bg-zinc-900 p-2 font-sans text-zinc-100 shadow-2xl shadow-black/50 outline-none"
      :style="{ left: `${position.left}px`, top: `${position.top}px` }"
      @contextmenu.prevent
    >
      <slot :close="close" />
    </div>
  </Teleport>
</template>
