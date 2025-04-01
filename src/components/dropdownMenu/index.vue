<template>
  <ElDropdown :trigger="['click']">
    <div class="but f-c cp" type="text">
      <span class="dropdown-title">{{ props.title }}▾</span>
    </div>
    <template #dropdown>
      <ElDropdownMenu>
        <ElDropdownItem
          v-for="item in menuItems"
          :key="item.label + `1`"
          @click="item.click"
        >
          <!-- <Menu.Item > -->
          <ElLink :underline="false">{{ item.label }}</ElLink>
          <!-- </Menu.Item> -->
        </ElDropdownItem>
      </ElDropdownMenu>
    </template>
  </ElDropdown>
</template>

<script setup lang="ts">
import {
  ElDropdown,
  ElDropdownMenu,
  ElDropdownItem,
  ElLink
} from "element-plus";
import type { MenuItem } from "./model";

const props = defineProps<{
  title: string;
  menuItems: MenuItem[];
  dropdownOffset?: { left: string };
}>();
</script>

<style scoped>
.dropdown-title {
  user-select: none;
  -webkit-user-select: none;
}
</style>
