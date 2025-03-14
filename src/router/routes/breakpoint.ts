export default [
  {
    path: "/breakpoint",
    component: () => import("@/window/breakpoint/index.vue")
  },
  {
    path: "/breakpoint/edit",
    component: () =>
      import("@/window/breakpoint/edit/index.tsx").then(
        (module) => module.default
      )
  },
  {
    path: "/breakpoint/pause",
    component: () => import("@/window/breakpoint/pause/index.vue")
  }
];
