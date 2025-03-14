import { createWebHistory, createRouter } from "vue-router";
import Home from "@/window/index.vue";
import { breakpointRoute, settingRoute, externalProxyRoute } from "./routes";

const routes = [
  { path: "/", component: Home },
  ...breakpointRoute,
  ...settingRoute,
  ...externalProxyRoute
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;
