import { onMounted, ref } from "vue";

export function useAsyncData<T>(loader: () => Promise<T>, initialValue: T) {
  const data = ref<T>(initialValue);
  const isLoading = ref(true);
  const error = ref<string | null>(null);

  async function reload() {
    isLoading.value = true;
    error.value = null;

    try {
      data.value = await loader();
    } catch (cause) {
      error.value = cause instanceof Error ? cause.message : "Unexpected error.";
    } finally {
      isLoading.value = false;
    }
  }

  onMounted(() => {
    void reload();
  });

  return {
    data,
    isLoading,
    error,
    reload,
  };
}
