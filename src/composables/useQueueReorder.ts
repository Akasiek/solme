import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch, type Ref } from "vue";

interface QueueReorderOptions {
  list: Ref<HTMLElement | null>;
  revision: Ref<string | null>;
  currentIndex: Ref<number>;
  disabled: Ref<boolean>;
  move: (from: number, to: number, revision: string, currentIndex: number) => Promise<void>;
}

interface Drag {
  pointerId: number;
  from: number;
  boundary: number;
  startY: number;
  x: number;
  y: number;
  active: boolean;
  revision: string;
  currentIndex: number;
}

export function useQueueReorder(options: QueueReorderOptions) {
  const drag = ref<Drag | null>(null);
  const isSaving = ref(false);
  const error = ref<string | null>(null);
  const announcement = ref("");
  let frame: number | null = null;
  let previousFrame = 0;
  let captureTarget: HTMLElement | null = null;
  const isReordering = computed(() => drag.value !== null || isSaving.value);
  const draggedIndex = computed(() => (drag.value?.active ? drag.value.from : null));
  const dropBoundary = computed(() => (drag.value?.active ? drag.value.boundary : null));

  const cancel = () => {
    const pointerId = drag.value?.pointerId;
    drag.value = null;
    if (frame !== null) {
      cancelAnimationFrame(frame);
    }
    frame = null;

    if (pointerId !== undefined && captureTarget?.hasPointerCapture(pointerId)) {
      captureTarget.releasePointerCapture(pointerId);
    }
    captureTarget = null;
  };

  const updateBoundary = () => {
    const current = drag.value;
    const list = options.list.value;
    if (!current || !list) {
      return;
    }

    const rows = Array.from(list.querySelectorAll<HTMLElement>("[data-queue-index]"));
    const boundary = rows.findIndex((row) => {
      const rect = row.getBoundingClientRect();
      return current.y < rect.top + rect.height / 2;
    });

    current.boundary = boundary < 0 ? rows.length : boundary;
  };

  const scroll = (now: number) => {
    const current = drag.value;
    const list = options.list.value;
    if (!current?.active || !list) {
      return;
    }

    const rect = list.getBoundingClientRect();
    const edge = Math.min(48, rect.height / 3);
    const elapsed = Math.min(now - previousFrame, 32);
    previousFrame = now;

    if (current.x >= rect.left && current.x <= rect.right) {
      let speed = 0;
      if (current.y < rect.top + edge) {
        speed = -Math.min(1, (rect.top + edge - current.y) / edge);
      } else if (current.y > rect.bottom - edge) {
        speed = Math.min(1, (current.y - rect.bottom + edge) / edge);
      }
      list.scrollTop += speed * elapsed * 0.5;
    }

    updateBoundary();
    frame = requestAnimationFrame(scroll);
  };

  const start = (event: PointerEvent, from: number) => {
    if (event.button !== 0 || !event.isPrimary || options.disabled.value || isSaving.value) {
      return;
    }

    const revision = options.revision.value;
    if (revision === null) {
      return;
    }

    cancel();
    error.value = null;
    captureTarget = event.currentTarget as HTMLElement;
    captureTarget.setPointerCapture(event.pointerId);

    drag.value = {
      pointerId: event.pointerId,
      from,
      boundary: from,
      startY: event.clientY,
      x: event.clientX,
      y: event.clientY,
      active: false,
      revision,
      currentIndex: options.currentIndex.value,
    };
  };

  const pointerMove = (event: PointerEvent) => {
    const current = drag.value;
    if (!current || event.pointerId !== current.pointerId) {
      return;
    }

    current.x = event.clientX;
    current.y = event.clientY;

    if (!current.active && Math.abs(current.y - current.startY) >= 5) {
      current.active = true;
      previousFrame = performance.now();
      frame = requestAnimationFrame(scroll);
    }

    if (current.active) {
      event.preventDefault();
      updateBoundary();
    }
  };

  const save = async (from: number, to: number, revision: string, currentIndex: number, focusHandle = false) => {
    if (from === to || isSaving.value) {
      return;
    }

    isSaving.value = true;
    error.value = null;

    try {
      await options.move(from, to, revision, currentIndex);
      announcement.value = `Song moved to position ${to + 1}.`;

      if (focusHandle) {
        isSaving.value = false;
        await nextTick();
        const handle = options.list.value?.querySelector<HTMLButtonElement>(`[data-queue-index="${to}"] button`);
        handle?.focus({ preventScroll: true });
        handle?.scrollIntoView({ block: "nearest" });
      }
    } catch (cause) {
      error.value = `Could not reorder the queue. ${String(cause)}`;
    } finally {
      isSaving.value = false;
    }
  };

  const pointerUp = (event: PointerEvent) => {
    const current = drag.value;
    if (!current || event.pointerId !== current.pointerId) {
      return;
    }

    current.x = event.clientX;
    current.y = event.clientY;
    updateBoundary();

    const rect = options.list.value?.getBoundingClientRect();
    const inside =
      rect && current.x >= rect.left && current.x <= rect.right && current.y >= rect.top && current.y <= rect.bottom;
    const to = current.boundary > current.from ? current.boundary - 1 : current.boundary;
    // Set saving before cancelling the gesture to keep automatic centering suspended.
    if (current.active && inside) {
      void save(current.from, to, current.revision, current.currentIndex);
    }

    cancel();
  };

  const keyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && drag.value) {
      event.preventDefault();
      cancel();
    }
  };

  const moveWithKeyboard = (from: number, to: number) => {
    const revision = options.revision.value;
    if (options.disabled.value || isReordering.value || revision === null) {
      return;
    }

    void save(from, to, revision, options.currentIndex.value, true);
  };

  watch(
    [options.revision, options.currentIndex, options.disabled],
    () => {
      if (!drag.value) return;
      cancel();
      announcement.value = "Reordering cancelled because the queue or current song changed.";
    },
    { flush: "sync" },
  );

  onMounted(() => {
    window.addEventListener("pointermove", pointerMove, { passive: false });
    window.addEventListener("pointerup", pointerUp);
    window.addEventListener("pointercancel", cancel);
    window.addEventListener("blur", cancel);
    window.addEventListener("keydown", keyDown);
  });

  onBeforeUnmount(() => {
    cancel();
    window.removeEventListener("pointermove", pointerMove);
    window.removeEventListener("pointerup", pointerUp);
    window.removeEventListener("pointercancel", cancel);
    window.removeEventListener("blur", cancel);
    window.removeEventListener("keydown", keyDown);
  });

  return { start, cancel, draggedIndex, dropBoundary, isReordering, isSaving, error, announcement, moveWithKeyboard };
}
