<template>
  <div class="toolbar f-r f-g-20">
    <DropdownMenu
      v-for="menu in dropdownMenus"
      :key="menu.title"
      :title="menu.title"
      :menuItems="menu.menuItems"
    />
  </div>
</template>

<script setup lang="ts">
import DropdownMenu from "@/components/dropdownMenu/index.vue";
import { importMenuItems } from "./menu/import";
import { createExportMenuItems } from "./menu/export";
import { proxyMenuItems } from "./menu/proxy";
import { settingsMenuItems } from "./menu/settings";
import { helpMenuItems } from "./menu/help";
import { fileMenuItems } from "./menu/file";
import { ref, watch } from "vue";
import { useSessionStore } from "@/stores/session";
import { toolsMenuItems } from "./menu/tools";

const sessionStore = useSessionStore();

const dropdownMenus = ref([
  {
    title: "File",
    menuItems: fileMenuItems
  },
  {
    title: "Import",
    menuItems: importMenuItems
  },
  {
    title: "Export",
    menuItems: createExportMenuItems(sessionStore.currentSession ?? "")
  },
  {
    title: "Proxy",
    menuItems: proxyMenuItems
  },
  {
    title: "Tools",
    menuItems: toolsMenuItems
  },
  {
    title: "Settings",
    menuItems: settingsMenuItems
  },
  {
    title: "Help",
    menuItems: helpMenuItems
  }
]);

watch(
  () => sessionStore.currentSession,
  (newSession) => {
    dropdownMenus.value.forEach((menu) => {
      if (menu.title === "Export") {
        menu.menuItems = createExportMenuItems(newSession ?? "");
      }
    });
  }
);
</script>

<style scoped></style>
