import { createRouter, createWebHashHistory } from "vue-router";

import { installLibraryProfileGuard } from "@/router/libraryProfileGuard";
import HomeView from "@/views/HomeView.vue";
import AlbumsView from "@/views/AlbumsView.vue";
import AlbumView from "@/views/AlbumView.vue";
import ArtistView from "@/views/ArtistView.vue";
import ArtistsView from "@/views/ArtistsView.vue";
import SongsView from "@/views/SongsView.vue";
import GenresView from "@/views/GenresView.vue";
import SearchView from "@/views/SearchView.vue";
import SettingsView from "@/views/settings/SettingsView.vue";
import SettingsAccountView from "@/views/settings/SettingsAccountView.vue";
import SettingsLyricsView from "@/views/settings/SettingsLyricsView.vue";
import LyricsView from "@/views/LyricsView.vue";

const serverAuthorizationRoute = "settings-account";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomeView,
      meta: { requiresLibraryProfile: true },
    },
    {
      path: "/album/:albumId",
      name: "album",
      component: AlbumView,
      props: true,
      meta: { requiresLibraryProfile: true },
    },
    {
      path: "/albums",
      name: "albums",
      component: AlbumsView,
      meta: { requiresLibraryProfile: true },
    },
    {
      path: "/search",
      name: "search",
      component: SearchView,
      meta: { requiresLibraryProfile: true },
    },
    {
      path: "/artists",
      name: "artists",
      component: ArtistsView,
      meta: { requiresLibraryProfile: true },
    },
    {
      path: "/songs",
      name: "songs",
      component: SongsView,
      meta: { requiresLibraryProfile: true },
    },
    {
      path: "/genres",
      name: "genres",
      component: GenresView,
      meta: { requiresLibraryProfile: true },
    },
    {
      path: "/artist/:artistId",
      name: "artist",
      component: ArtistView,
      props: true,
      meta: { requiresLibraryProfile: true },
    },
    {
      path: "/lyrics",
      name: "lyrics",
      component: LyricsView,
      meta: { requiresLibraryProfile: true },
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
        {
          path: "lyrics",
          name: "settings-lyrics",
          component: SettingsLyricsView,
        },
      ],
    },
  ],
});

installLibraryProfileGuard(router);
