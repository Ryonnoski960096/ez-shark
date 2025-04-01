import { createWebHistory, createRouter } from "vue-router";
import Home from "@/window/index.vue";
import {
  breakpointRoute,
  settingRoute,
  externalProxyRoute,
  searchRoute
} from "./routes";

const routes = [
  { path: "/", component: Home },
  ...breakpointRoute,
  ...settingRoute,
  ...externalProxyRoute,
  ...searchRoute
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;
