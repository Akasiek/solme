import { defineStore } from "pinia";
import { ref } from "vue";

export interface ToastMessage {
  id: number;
  message: string;
}

export const useToastStore = defineStore("toast", () => {
  const toasts = ref<ToastMessage[]>([]);
  let nextToastId = 1;

  const dismiss = (id: number) => {
    toasts.value = toasts.value.filter((toast) => toast.id !== id);
  };

  const show = (message: string) => {
    const id = nextToastId++;
    toasts.value.push({ id, message });

    window.setTimeout(() => {
      dismiss(id);
    }, 4200);
  };

  return {
    toasts,
    dismiss,
    show,
  };
});
