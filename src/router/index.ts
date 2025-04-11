import { createWebHistory, createRouter } from "vue-router";
import Home from "@/window/index.vue";
import {
  breakpointRoute,
  settingRoute,
  externalProxyRoute,
  searchRoute,
  mapLocal
} from "./routes";

const routes = [
  { path: "/", component: Home },
  ...breakpointRoute,
  ...settingRoute,
  ...externalProxyRoute,
  ...searchRoute,
  ...mapLocal
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;
