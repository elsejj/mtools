<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useHistoryStore } from '@/stores/history';
import { usePayloadStore } from '@/stores/payload';
import { useToolStore } from '@/stores/tools';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import {
  IconX,
  IconSearch,
  IconTrash,
  IconClock,
  IconFolder,
  IconArrowUpRight,
  IconAlertCircle,
  IconCheck,
} from '@tabler/icons-vue';
import { tauriApi } from '@/lib/tauri';
import type { HistoryRecordItem } from '@/types';

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const historyStore = useHistoryStore();
const payloadStore = usePayloadStore();
const toolStore = useToolStore();

const searchInput = ref('');

watch(searchInput, (val) => {
  historyStore.setKeyword(val);
});

onMounted(() => {
  historyStore.fetchRecords(true);
});

async function loadIntoWorkspace(item: HistoryRecordItem) {
  if (item.inputText) {
    await payloadStore.processText(item.inputText);
  }
  if (item.toolId) {
    toolStore.setActiveTool(item.toolId);
  }
  emit('close');
}

function openFolder(path?: string) {
  if (path) {
    tauriApi.showInFolder(path);
  }
}

function formatDate(ts: number) {
  const d = new Date(ts);
  return `${d.toLocaleDateString()} ${d.toLocaleTimeString()}`;
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex justify-end bg-black/40 backdrop-blur-2xs select-none animate-in fade-in duration-150">
    <div class="flex h-full w-full max-w-md flex-col border-l border-border bg-background shadow-xl">
      <!-- Header -->
      <div class="flex h-12 shrink-0 items-center justify-between border-b border-border px-4 bg-muted/20">
        <div class="flex items-center space-x-2">
          <IconClock class="h-4 w-4 text-primary" />
          <h2 class="text-sm font-semibold text-foreground">历史记录</h2>
          <Badge variant="secondary" class="text-[10px]">
            {{ historyStore.records.length }}
          </Badge>
        </div>

        <div class="flex items-center space-x-1">
          <Button
            variant="ghost"
            size="sm"
            class="h-7 text-xs text-muted-foreground hover:text-destructive cursor-pointer"
            title="清空全部历史"
            @click="historyStore.clearAll"
          >
            <IconTrash class="h-3.5 w-3.5 mr-1" />
            清空
          </Button>
          <Button
            variant="ghost"
            size="sm"
            class="h-7 w-7 p-0 cursor-pointer text-muted-foreground hover:text-foreground"
            @click="emit('close')"
          >
            <IconX class="h-4 w-4" />
          </Button>
        </div>
      </div>

      <!-- Search & Filter Bar -->
      <div class="p-3 border-b border-border bg-muted/5">
        <div class="relative">
          <IconSearch class="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-muted-foreground" />
          <Input
            v-model="searchInput"
            placeholder="搜索历史记录摘要与内容..."
            class="h-8 pl-8 text-xs bg-background"
          />
        </div>
      </div>

      <!-- Record List -->
      <div class="flex-1 overflow-y-auto p-3 space-y-2.5">
        <div
          v-for="item in historyStore.records"
          :key="item.id"
          class="group rounded-lg border border-border bg-card p-3 shadow-2xs hover:border-primary/40 transition-colors"
        >
          <!-- Record Header -->
          <div class="flex items-center justify-between pb-1.5 border-b border-border/50 text-xs">
            <div class="flex items-center space-x-1.5 font-medium">
              <span>{{ item.toolName }}</span>
              <Badge variant="outline" class="text-[9px] px-1 py-0 uppercase">
                {{ item.payloadType }}
              </Badge>
              <Badge
                v-if="item.status === 'success'"
                variant="secondary"
                class="text-[9px] px-1 py-0 text-green-600 bg-green-500/10 border-green-500/20"
              >
                成功
              </Badge>
              <Badge
                v-else
                variant="destructive"
                class="text-[9px] px-1 py-0"
              >
                失败
              </Badge>
            </div>

            <span class="text-[10px] text-muted-foreground">
              {{ formatDate(item.createdAt) }}
            </span>
          </div>

          <!-- Record Body: Input summary -->
          <div class="py-2 text-xs font-mono text-muted-foreground truncate select-text">
            {{ item.inputSummary || item.inputText || '(无输入文本)' }}
          </div>

          <!-- File saved indicator if any -->
          <div
            v-if="item.outputFilePath"
            class="flex items-center justify-between rounded bg-muted/50 px-2 py-1 text-[11px] text-muted-foreground mb-2"
          >
            <span class="truncate pr-1">文件: {{ item.outputFilePath }}</span>
            <Button
              variant="ghost"
              size="sm"
              class="h-5 px-1 text-[10px] shrink-0"
              @click="openFolder(item.outputFilePath)"
            >
              <IconFolder class="h-3 w-3" />
            </Button>
          </div>

          <!-- Actions -->
          <div class="flex items-center justify-between pt-1 text-xs">
            <span class="text-[10px] text-muted-foreground">
              耗时: {{ item.durationMs }}ms
            </span>
            <div class="flex items-center space-x-1">
              <Button
                variant="secondary"
                size="sm"
                class="h-6 px-2 text-[11px] cursor-pointer"
                @click="loadIntoWorkspace(item)"
              >
                <IconArrowUpRight class="h-3 w-3 mr-1" />
                载入工作区
              </Button>
              <Button
                variant="ghost"
                size="sm"
                class="h-6 w-6 p-0 text-muted-foreground hover:text-destructive cursor-pointer"
                title="删除记录"
                @click="historyStore.deleteRecord(item.id)"
              >
                <IconTrash class="h-3 w-3" />
              </Button>
            </div>
          </div>
        </div>

        <div
          v-if="historyStore.records.length === 0"
          class="flex flex-col items-center justify-center py-16 text-center text-xs text-muted-foreground"
        >
          <IconClock class="h-8 w-8 mb-2 opacity-30" />
          <p class="font-medium">暂无历史记录</p>
          <p class="text-[11px] opacity-70 mt-1">使用工具执行处理后，记录将自动在此归档</p>
        </div>
      </div>
    </div>
  </div>
</template>

