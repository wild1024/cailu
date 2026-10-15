<template>
  <t-submenu v-if="hasChildren" :value="menuValue">
    <template #icon><t-icon :name="icon" /></template>
    <template #title>{{ item.title }}</template>
    <side-menu-item v-for="child in children" :key="child.id" :item="child" />
  </t-submenu>
  <t-menu-item v-else :value="menuValue">
    <template #icon><t-icon :name="icon" /></template>
    {{ item.title }}
  </t-menu-item>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { MenuNode } from '@/api/system/auth-api';
import { normalizeMenuPath } from '@/utils/menu';

const props = defineProps<{
  /** 菜单节点（可含多级子节点） */
  item: MenuNode;
}>();

const children = computed(() => props.item.children ?? []);
const hasChildren = computed(() => children.value.length > 0);
const icon = computed(() => props.item.icon || (hasChildren.value ? 'folder' : 'app'));

/** 菜单项取值：有路径时用路由路径，否则用 id 占位（目录点击不跳转） */
const menuValue = computed(() => normalizeMenuPath(props.item.path) || `menu:${props.item.id}`);
</script>