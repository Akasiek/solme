import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { Ref } from "vue";

interface Options {
  defaultWidth?: number;
  minWidth?: number;
  maxWidth?: number;
  collapsedWidth?: number;
  collapseThreshold?: number;
  isLeft?: boolean;
  isCollapsed?: Ref<boolean>;
}

const defaultOptions: Required<Omit<Options, "isCollapsed">> = {
  defaultWidth: 224,
  minWidth: 208,
  maxWidth: 360,
  collapsedWidth: 72,
  collapseThreshold: 136,
  isLeft: true,
};

export function useAsideMenuSize(storageKey: string, providedOptions: Options = {}) {
  const options = {
    ...defaultOptions,
    ...providedOptions,
  };
  const { defaultWidth, minWidth, maxWidth, collapsedWidth, collapseThreshold, isLeft } = options;

  const width = ref(defaultWidth);
  const isCollapsed = providedOptions.isCollapsed ?? ref(false);
  const isResizing = ref(false);
  const asideWidth = computed(() => (isCollapsed.value ? collapsedWidth : width.value));

  let startX = 0;
  let startWidth = defaultWidth;
  let resizeHandle: HTMLElement | null = null;
  let activePointerId: number | null = null;

  const clampWidth = (value: number) => Math.min(maxWidth, Math.max(minWidth, value));
  const clampResizeWidth = (value: number) => Math.min(maxWidth, Math.max(collapsedWidth, value));

  const saveState = () => {
    localStorage.setItem(
      storageKey,
      JSON.stringify({
        width: width.value,
        isCollapsed: isCollapsed.value,
      }),
    );
  };

  const toggleCollapsed = () => {
    isCollapsed.value = !isCollapsed.value;
  };

  const resize = (event: PointerEvent) => {
    if (!isResizing.value) {
      return;
    }

    if (isLeft) {
      width.value = clampResizeWidth(startWidth + event.clientX - startX);
    } else {
      width.value = clampResizeWidth(startWidth - event.clientX + startX);
    }
  };

  const stopResize = () => {
    if (!isResizing.value) {
      return;
    }

    if (resizeHandle && activePointerId !== null) {
      resizeHandle.releasePointerCapture(activePointerId);
    }

    isResizing.value = false;
    resizeHandle = null;
    activePointerId = null;
    document.body.classList.remove("cursor-col-resize", "select-none");
    window.removeEventListener("pointermove", resize);
    window.removeEventListener("pointerup", stopResize);
    window.removeEventListener("pointercancel", stopResize);

    if (width.value <= collapseThreshold) {
      isCollapsed.value = true;
      width.value = startWidth;
    } else {
      width.value = clampWidth(width.value);
    }

    saveState();
  };

  const startResize = (event: PointerEvent) => {
    if (isCollapsed.value) {
      return;
    }

    event.preventDefault();
    startX = event.clientX;
    startWidth = width.value;
    isResizing.value = true;
    activePointerId = event.pointerId;
    resizeHandle = event.currentTarget as HTMLElement;
    resizeHandle.setPointerCapture(activePointerId);

    document.body.classList.add("cursor-col-resize", "select-none");
    window.addEventListener("pointermove", resize);
    window.addEventListener("pointerup", stopResize);
    window.addEventListener("pointercancel", stopResize);
  };

  const resetWidth = () => {
    width.value = defaultWidth;
    saveState();
  };

  onMounted(() => {
    const savedState = localStorage.getItem(storageKey);

    if (!savedState) {
      return;
    }

    try {
      const parsedState = JSON.parse(savedState) as { width?: number; isCollapsed?: boolean };
      if (typeof parsedState.width === "number") {
        width.value = clampWidth(parsedState.width);
      }
      isCollapsed.value = parsedState.isCollapsed === true;
    } catch {
      localStorage.removeItem(storageKey);
    }
  });

  watch(isCollapsed, saveState);

  onUnmounted(stopResize);

  return {
    asideWidth,
    isCollapsed,
    isResizing,
    resetWidth,
    startResize,
    toggleCollapsed,
  };
}
