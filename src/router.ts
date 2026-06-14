import { createRouter, createWebHashHistory } from "vue-router";

import AlbumView from "@/views/AlbumView.vue";
import HomeView from "@/views/HomeView.vue";
import PlayerView from "@/views/PlayerView.vue";
import SearchView from "@/views/SearchView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomeView,
    },
    {
      path: "/album/:albumId",
      name: "album",
      component: AlbumView,
      props: true,
    },
    {
      path: "/search",
      name: "search",
      component: SearchView,
    },
    {
      path: "/player",
      name: "player",
      component: PlayerView,
    },
  ],
});
