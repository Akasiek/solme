import { readonly, ref } from "vue";

export interface ToastMessage {
  id: number;
  message: string;
}

const toasts = ref<ToastMessage[]>([]);
let nextToastId = 1;

export function showToast(message: string) {
  const id = nextToastId++;
  toasts.value.push({ id, message });

  window.setTimeout(() => {
    dismissToast(id);
  }, 4200);
}

export function dismissToast(id: number) {
  toasts.value = toasts.value.filter((toast) => toast.id !== id);
}

export function useToast() {
  return {
    toasts: readonly(toasts),
    dismissToast,
    showToast,
  };
}
