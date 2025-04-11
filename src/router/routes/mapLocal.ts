export default [
  {
    path: "/mapLocal",
    component: () => import("@/window/mapLocal/index.vue")
  },
  {
    path: "/mapLocal/edit",
    component: () => import("@/window/mapLocal/edit/index.vue")
  }
];
