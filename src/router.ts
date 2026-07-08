import { createRouter, createWebHashHistory } from "vue-router";

import { installServerConnectionGuard } from "@/router/serverConnectionGuard";
import HomeView from "@/views/HomeView.vue";
import AlbumView from "@/views/AlbumView.vue";
import ArtistView from "@/views/ArtistView.vue";
import SearchView from "@/views/SearchView.vue";
import SettingsView from "@/views/settings/SettingsView.vue";
import SettingsAccountView from "@/views/settings/SettingsAccountView.vue";

const serverAuthorizationRoute = "settings-account";

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

installServerConnectionGuard(router);
