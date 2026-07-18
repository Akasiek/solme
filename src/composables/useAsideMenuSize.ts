import { computed, onMounted, onUnmounted, ref } from "vue";

const defaultWidth = 224;
const minWidth = 208;
const maxWidth = 360;
const collapsedWidth = 72;
const collapseThreshold = 136;
const storageKey = "solme-aside-menu";

export function useAsideMenuSize() {
  const width = ref(defaultWidth);
  const isCollapsed = ref(false);
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
    saveState();
  };

  const resize = (event: PointerEvent) => {
    if (!isResizing.value) {
      return;
    }

    width.value = clampResizeWidth(startWidth + event.clientX - startX);
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
