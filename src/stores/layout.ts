import { defineStore } from "pinia";
import { ref } from "vue";

export const useLayoutStore = defineStore("layout", () => {
  const isRightAsideCollapsed = ref(false);
  const isBigArtworkShown = ref(false);
  const isSearchModalOpen = ref(false);

  const toggleRightAside = () => {
    isRightAsideCollapsed.value = !isRightAsideCollapsed.value;
  };

  const toggleBigArtwork = () => {
    isBigArtworkShown.value = !isBigArtworkShown.value;
  };

  const openSearchModal = () => {
    isSearchModalOpen.value = true;
  };

  const closeSearchModal = () => {
    isSearchModalOpen.value = false;
  };

  return {
    isRightAsideCollapsed,
    isBigArtworkShown,
    isSearchModalOpen,
    toggleRightAside,
    toggleBigArtwork,
    openSearchModal,
    closeSearchModal,
  };
});
