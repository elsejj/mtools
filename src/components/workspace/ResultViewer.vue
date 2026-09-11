<script setup lang="ts">
import { ref, computed } from 'vue';
import { useToolStore } from '@/stores/tools';
import { usePayloadStore } from '@/stores/payload';
import CodeHighlight from './CodeHighlight.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  IconCopy,
  IconCheck,
  IconAlertTriangle,
  IconFolder,
  IconLoader2,
  IconSparkles,
} from '@tabler/icons-vue';
import { tauriApi } from '@/lib/tauri';

const toolStore = useToolStore();
const payloadStore = usePayloadStore();

const copied = ref(false);

const outputLanguage = computed(() => {
  if (toolStore.activeTool?.type === 'cli') return 'bash';
  if (payloadStore.currentPayload?.preprocessedResult?.suggestedOutputType) {
    return payloadStore.currentPayload.preprocessedResult.suggestedOutputType;
  }
  if (toolStore.activeTool?.id === 'json-formatter' || toolStore.activeTool?.id === 'jwt-inspector') {
    return 'json';
  }
  return 'markdown';
});

async function copyOutput() {
  if (!toolStore.executionOutput) return;
  try {
    await navigator.clipboard.writeText(toolStore.executionOutput);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 1600);
  } catch (err) {
    console.error('Failed to copy output:', err);
  }
}

function openFolder(path: string) {
  tauriApi.showInFolder(path);
}
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden select-none bg-background">
    <!-- Header -->
    <div class="flex h-9 shrink-0 items-center justify-between border-b border-border px-3 bg-muted/10 text-xs">
      <div class="flex items-center space-x-2">
        <span class="font-medium text-foreground">
          {{ toolStore.activeTool?.name || '处理结果' }}
        </span>
        <Badge variant="outline" class="text-[10px] px-1 py-0 uppercase">
          {{ toolStore.activeTool?.type || 'code' }}
        </Badge>
        <Badge v-if="outputLanguage" variant="secondary" class="text-[10px] px-1 py-0 uppercase font-mono">
          {{ outputLanguage }}
        </Badge>
      </div>

      <div class="flex items-center space-x-1.5">
        <Button
          variant="ghost"
          size="sm"
          class="h-6 px-2 text-xs cursor-pointer hover:bg-muted"
          :disabled="!toolStore.executionOutput"
          @click="copyOutput"
        >
          <IconCheck v-if="copied" class="h-3.5 w-3.5 mr-1 text-green-500" />
          <IconCopy v-else class="h-3.5 w-3.5 mr-1" />
          <span>{{ copied ? '已复制' : '复制结果' }}</span>
        </Button>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 overflow-hidden relative">
      <!-- Loading overlay -->
      <div
        v-if="toolStore.isExecuting"
        class="absolute inset-0 z-10 flex flex-col items-center justify-center bg-background/70 backdrop-blur-2xs text-xs text-muted-foreground"
      >
        <IconLoader2 class="h-6 w-6 animate-spin text-primary mb-2" />
        <span>正在运行工具处理...</span>
      </div>

      <!-- Error State -->
      <div
        v-else-if="toolStore.executionError"
        class="p-4 flex flex-col space-y-2 text-xs text-destructive bg-destructive/5 h-full overflow-auto"
      >
        <div class="flex items-center space-x-1.5 font-semibold">
          <IconAlertTriangle class="h-4 w-4" />
          <span>执行错误</span>
        </div>
        <pre class="font-mono whitespace-pre-wrap break-all rounded border border-destructive/20 bg-destructive/10 p-3">{{ toolStore.executionError }}</pre>
      </div>

      <!-- Output Display -->
      <div v-else-if="toolStore.executionOutput" class="h-full">
        <CodeHighlight
          :code="toolStore.executionOutput"
          :language="outputLanguage"
        />
      </div>

      <!-- Empty State -->
      <div
        v-else
        class="flex h-full flex-col items-center justify-center text-center text-xs text-muted-foreground select-none"
      >
        <IconSparkles class="h-8 w-8 mb-2 opacity-30 text-primary" />
        <p class="font-medium">等待执行</p>
        <p class="text-[11px] opacity-70 mt-1">选择工具或按下 Ctrl+Enter 运行</p>
      </div>
    </div>

    <!-- Post-Action Saved File Notification -->
    <div
      v-if="toolStore.lastSavedFilePath"
      class="flex h-8 shrink-0 items-center justify-between border-t border-border bg-muted/40 px-3 text-xs text-muted-foreground"
    >
      <div class="flex items-center space-x-1.5 truncate pr-2">
        <span class="truncate">已自动落盘：<code class="font-mono text-[11px]">{{ toolStore.lastSavedFilePath }}</code></span>
      </div>
      <Button
        variant="ghost"
        size="sm"
        class="h-6 px-1.5 text-[11px] shrink-0 cursor-pointer"
        @click="openFolder(toolStore.lastSavedFilePath)"
      >
        <IconFolder class="h-3 w-3 mr-1" />
        打开文件夹
      </Button>
    </div>
  </div>
</template>

