import { createRouter, createWebHashHistory } from "vue-router";
import { invoke } from "@tauri-apps/api/core";

import HomeView from "@/views/HomeView.vue";
import AlbumView from "@/views/AlbumView.vue";
import ArtistView from "@/views/ArtistView.vue";
import LoginView from "@/views/LoginView.vue";

async function ensureServerConnection() {
  try {
    await invoke("ping_music_server");
    return true;
  } catch {
    try {
      await invoke("connect_saved_music_server");
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
      path: "/artist/:artistId",
      name: "artist",
      component: ArtistView,
      props: true,
      meta: { requiresServer: true },
    },
    {
      path: "/login",
      name: "login",
      component: LoginView,
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

  return { name: "login" };
});
