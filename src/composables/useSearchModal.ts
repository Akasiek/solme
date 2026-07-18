import { ref } from "vue";

const isSearchModalOpen = ref(false);

export function useSearchModal() {
  const openSearchModal = () => {
    isSearchModalOpen.value = true;
  };

  const closeSearchModal = () => {
    isSearchModalOpen.value = false;
  };

  return {
    closeSearchModal,
    isSearchModalOpen,
    openSearchModal,
  };
}
