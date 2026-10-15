import type { MenuNode } from '@/api/system/auth-api';

/** 把后端菜单路径规整为以 `/` 开头的路由路径 */
export function normalizeMenuPath(path: string): string {
  if (!path) {
    return '';
  }
  return path.startsWith('/') ? path : `/${path}`;
}

/** 收集菜单树中所有叶子节点路径（有子节点的目录不参与跳转） */
export function collectLeafPaths(nodes: MenuNode[], result: Set<string> = new Set()): Set<string> {
  for (const node of nodes) {
    const children = node.children ?? [];
    if (children.length > 0) {
      collectLeafPaths(children, result);
      continue;
    }
    const path = normalizeMenuPath(node.path);
    if (path) {
      result.add(path);
    }
  }
  return result;
}