import { createRouter, createWebHashHistory } from "vue-router";
import { invoke } from "@tauri-apps/api/core";

import HomeView from "@/views/HomeView.vue";
import AlbumView from "@/views/AlbumView.vue";
import ArtistView from "@/views/ArtistView.vue";
import SearchView from "@/views/SearchView.vue";
import SettingsView from "@/views/settings/SettingsView.vue";
import SettingsAccountView from "@/views/settings/SettingsAccountView.vue";
import { showToast } from "@/composables/useToast";

const serverAuthorizationRoute = "settings-account";

async function ensureServerConnection() {
  try {
    await invoke("ping_music_server");
    return true;
  } catch {
    try {
      await invoke("connect_saved_music_server", { profileId: null });
      return true;
    } catch {
      return false;
    }
  }
}

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomeView,
      meta: { requiresServer: true },
    },
    {
      path: "/album/:albumId",
      name: "album",
      component: AlbumView,
      props: true,
      meta: { requiresServer: true },
    },
    {
      path: "/search",
      name: "search",
      component: SearchView,
      meta: { requiresServer: true },
    },
    {
      path: "/artist/:artistId",
      name: "artist",
      component: ArtistView,
      props: true,
      meta: { requiresServer: true },
    },
    {
      path: "/settings",
      component: SettingsView,
      meta: { requiresServer: true },
      children: [
        {
          path: "",
          redirect: { name: serverAuthorizationRoute },
        },
        {
          path: "account",
          name: serverAuthorizationRoute,
          component: SettingsAccountView,
        },
      ],
    },
  ],
});

router.beforeEach(async (to) => {
  if (!to.meta.requiresServer) {
    return true;
  }

  if (await ensureServerConnection()) {
    return true;
  }

  if (to.name === serverAuthorizationRoute) {
    return true;
  }

  showToast("Connect a music server before using the rest of Solme.");
  return { name: serverAuthorizationRoute, replace: true };
});
