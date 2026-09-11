import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { HistoryRecordItem, HistoryQuery } from '@/types';
import { tauriApi } from '@/lib/tauri';

export const useHistoryStore = defineStore('history', () => {
  const records = ref<HistoryRecordItem[]>([]);
  const isLoading = ref<boolean>(false);
  const keyword = ref<string>('');
  const selectedToolId = ref<string | undefined>(undefined);
  const pageSize = ref<number>(30);
  const offset = ref<number>(0);
  const hasMore = ref<boolean>(true);

  // Getters
  const filteredRecords = computed(() => {
    return records.value;
  });

  // Actions
  async function fetchRecords(reset = false) {
    if (reset) {
      offset.value = 0;
      hasMore.value = true;
    }
    if (!hasMore.value && !reset) return;

    isLoading.value = true;
    try {
      const query: HistoryQuery = {
        keyword: keyword.value.trim() || undefined,
        toolId: selectedToolId.value,
        limit: pageSize.value,
        offset: offset.value,
      };
      const data = await tauriApi.getHistoryRecords(query);
      if (reset) {
        records.value = data;
      } else {
        records.value.push(...data);
      }
      if (data.length < pageSize.value) {
        hasMore.value = false;
      } else {
        offset.value += data.length;
      }
    } catch (err) {
      console.error('Failed to fetch history records:', err);
    } finally {
      isLoading.value = false;
    }
  }

  async function deleteRecord(id: string) {
    try {
      await tauriApi.deleteHistoryRecord(id);
      records.value = records.value.filter((r) => r.id !== id);
    } catch (err) {
      console.error('Failed to delete history record:', err);
    }
  }

  async function clearAll() {
    try {
      await tauriApi.clearAllHistory();
      records.value = [];
      hasMore.value = false;
    } catch (err) {
      console.error('Failed to clear all history:', err);
    }
  }

  function setKeyword(val: string) {
    keyword.value = val;
    fetchRecords(true);
  }

  function setToolFilter(toolId?: string) {
    selectedToolId.value = toolId;
    fetchRecords(true);
  }

  return {
    // State
    records,
    isLoading,
    keyword,
    selectedToolId,
    hasMore,

    // Getters
    filteredRecords,

    // Actions
    fetchRecords,
    deleteRecord,
    clearAll,
    setKeyword,
    setToolFilter,
  };
});

