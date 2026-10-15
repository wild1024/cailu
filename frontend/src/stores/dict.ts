import { ref } from 'vue';
import { defineStore } from 'pinia';
import { listDictItems, type DictItemRes } from '@/api/support/dict-api';

/** 字典缓存：按字典编码缓存启用项，避免同一字典重复请求 */
export const useDictStore = defineStore('dict', () => {
  /** 字典编码 -> 字典项列表 */
  const items = ref<Record<string, DictItemRes[]>>({});

  /** 读取字典项，优先命中缓存 */
  async function loadItems(dictCode: string): Promise<DictItemRes[]> {
    const cached = items.value[dictCode];
    if (cached) {
      return cached;
    }
    const list = await listDictItems(dictCode);
    items.value[dictCode] = list;
    return list;
  }

  /** 取字典标签，未命中时回退为原始值 */
  function getLabel(dictCode: string, value: string | number | undefined): string {
    if (value === undefined || value === null || value === '') {
      return '';
    }
    const target = String(value);
    const item = items.value[dictCode]?.find((entry) => entry.item_value === target);
    return item ? item.item_label : target;
  }

  /** 清空缓存；传入字典编码时仅清理该字典（字典维护后可调用） */
  function clear(dictCode?: string): void {
    if (dictCode) {
      delete items.value[dictCode];
      return;
    }
    items.value = {};
  }

  return { items, loadItems, getLabel, clear };
});