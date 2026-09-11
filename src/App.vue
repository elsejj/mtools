<script setup lang="ts">
import { onMounted, onUnmounted, watch, ref } from 'vue';
import { usePayloadStore } from '@/stores/payload';
import { useToolStore } from '@/stores/tools';
import { useSettingsStore } from '@/stores/settings';
import { useHistoryStore } from '@/stores/history';
import { useWindowState } from '@/composables/useWindowState';
import { useShortcuts } from '@/composables/useShortcuts';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  IconSearch,
  IconClock,
  IconSettings,
  IconCopy,
  IconArrowRight,
  IconSparkles,
  IconCode,
  IconTerminal2,
  IconFolder,
  IconCheck,
} from '@tabler/icons-vue';
import { tauriApi } from '@/lib/tauri';

// Stores
const payloadStore = usePayloadStore();
const toolStore = useToolStore();
const settingsStore = useSettingsStore();
const historyStore = useHistoryStore();

// Window state tracking
useWindowState();

// UI States
const activeTab = ref<'workspace' | 'history' | 'settings'>('workspace');
const manualTextInput = ref('');
const copiedFeedback = ref(false);

// Auto-select best tool when payload changes
watch(
  () => payloadStore.currentPayload,
  (newPayload) => {
    if (newPayload) {
      toolStore.autoSelectBestTool(
        newPayload.recommendedToolId,
        newPayload.candidateToolScores
      );
      // Auto-execute or prepare output
      toolStore.executeTool(newPayload);
    }
  }
);

// Global shortcuts
useShortcuts({
  onSearch: () => {
    activeTab.value = 'workspace';
    const el = document.getElementById('search-or-input');
    el?.focus();
  },
  onHistory: () => {
    activeTab.value = activeTab.value === 'history' ? 'workspace' : 'history';
    if (activeTab.value === 'history') {
      historyStore.fetchRecords(true);
    }
  },
  onSettings: () => {
    activeTab.value = activeTab.value === 'settings' ? 'workspace' : 'settings';
  },
  onSelectCandidate: (index) => {
    const recommended = toolStore.getRecommendedTools(payloadStore.candidateScores);
    if (recommended[index]) {
      toolStore.setActiveTool(recommended[index].id);
      if (payloadStore.currentPayload) {
        toolStore.executeTool(payloadStore.currentPayload);
      }
    }
  },
  onExecute: () => {
    if (payloadStore.currentPayload) {
      toolStore.executeTool(payloadStore.currentPayload);
    }
  },
  onEscape: () => {
    if (activeTab.value !== 'workspace') {
      activeTab.value = 'workspace';
    }
  },
});

async function handleManualProcess() {
  if (!manualTextInput.value.trim()) return;
  await payloadStore.processText(manualTextInput.value);
}

async function handleFetchClipboard() {
  await payloadStore.fetchFromClipboard();
}

async function copyResult() {
  if (toolStore.executionOutput) {
    try {
      await navigator.clipboard.writeText(toolStore.executionOutput);
      copiedFeedback.value = true;
      setTimeout(() => {
        copiedFeedback.value = false;
      }, 1500);
    } catch (e) {
      console.error('Failed to copy:', e);
    }
  }
}

function openSavedFolder(path: string) {
  tauriApi.showInFolder(path);
}

onMounted(async () => {
  await settingsStore.loadSettings();
  await toolStore.loadTools();
  await payloadStore.startListening();
});

onUnmounted(() => {
  payloadStore.stopListening();
});
</script>

<template>
  <div class="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground select-none font-sans">
    <!-- Top Bar -->
    <header class="flex h-12 shrink-0 items-center justify-between border-b border-border px-4 bg-muted/20">
      <!-- Left: Logo & Candidate Quick Switchers -->
      <div class="flex items-center space-x-2 overflow-x-auto py-1">
        <div class="flex items-center space-x-1.5 font-bold tracking-tight text-primary mr-2 shrink-0">
          <span class="bg-primary text-primary-foreground text-xs font-mono px-1.5 py-0.5 rounded">M</span>
          <span class="text-sm">mTools</span>
        </div>

        <!-- Candidate Tools -->
        <div class="flex items-center space-x-1">
          <button
            v-for="(candidate, idx) in toolStore.getRecommendedTools(payloadStore.candidateScores).slice(0, 5)"
            :key="candidate.id"
            @click="() => { toolStore.setActiveTool(candidate.id); if (payloadStore.currentPayload) toolStore.executeTool(payloadStore.currentPayload); }"
            :class="[
              'flex items-center space-x-1.5 rounded-md px-2 py-1 text-xs transition-colors',
              toolStore.activeToolId === candidate.id
                ? 'bg-primary text-primary-foreground font-medium shadow-xs'
                : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'
            ]"
          >
            <span>{{ candidate.name }}</span>
            <kbd class="ml-1 rounded bg-black/10 dark:bg-white/10 px-1 py-0.2 text-[10px] font-mono">
              Alt+{{ idx + 1 }}
            </kbd>
          </button>
        </div>
      </div>

      <!-- Right: Action Buttons (History, Settings) -->
      <div class="flex items-center space-x-1 shrink-0">
        <Button
          variant="ghost"
          size="sm"
          :class="activeTab === 'history' ? 'bg-accent text-accent-foreground' : ''"
          @click="() => { activeTab = activeTab === 'history' ? 'workspace' : 'history'; if (activeTab === 'history') historyStore.fetchRecords(true); }"
          title="历史记录 (Ctrl+H)"
        >
          <IconClock class="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :class="activeTab === 'settings' ? 'bg-accent text-accent-foreground' : ''"
          @click="activeTab = activeTab === 'settings' ? 'workspace' : 'settings'"
          title="设置 (Ctrl+,)"
        >
          <IconSettings class="h-4 w-4" />
        </Button>
      </div>
    </header>

    <!-- Decoding Trace & Notification Bar -->
    <div
      v-if="payloadStore.hasDecodingTrace"
      class="flex items-center justify-between border-b border-amber-500/20 bg-amber-500/10 px-4 py-1.5 text-xs text-amber-700 dark:text-amber-300"
    >
      <div class="flex items-center space-x-2">
        <IconSparkles class="h-3.5 w-3.5" />
        <span>自动多层解码溯源：</span>
        <div class="flex items-center space-x-1">
          <Badge
            v-for="(trace, i) in payloadStore.decodingTrace"
            :key="i"
            variant="outline"
            class="text-[10px] bg-background/80 border-amber-500/30"
          >
            {{ trace }}
          </Badge>
        </div>
      </div>
      <button
        @click="payloadStore.toggleOriginalDecoded"
        class="underline cursor-pointer hover:opacity-80 text-[11px]"
      >
        {{ payloadStore.showOriginalDecoded ? '显示解码后' : '查看解码前原文' }}
      </button>
    </div>

    <!-- Main Workspace -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Left Sidebar: Tools Directory -->
      <aside class="w-48 shrink-0 border-r border-border bg-muted/10 p-2 flex flex-col justify-between">
        <div class="space-y-3 overflow-y-auto">
          <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider px-2">
            常用工具
          </div>
          <div class="space-y-0.5">
            <button
              v-for="tool in toolStore.tools"
              :key="tool.id"
              @click="() => { toolStore.setActiveTool(tool.id); if (payloadStore.currentPayload) toolStore.executeTool(payloadStore.currentPayload); }"
              :class="[
                'flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-xs transition-colors',
                toolStore.activeToolId === tool.id
                  ? 'bg-accent text-accent-foreground font-medium'
                  : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'
              ]"
            >
              <div class="flex items-center space-x-2 truncate">
                <IconCode v-if="tool.type === 'code'" class="h-3.5 w-3.5 shrink-0" />
                <IconSparkles v-else-if="tool.type === 'llm'" class="h-3.5 w-3.5 shrink-0" />
                <IconTerminal2 v-else class="h-3.5 w-3.5 shrink-0" />
                <span class="truncate">{{ tool.name }}</span>
              </div>
              <Badge v-if="tool.id === payloadStore.recommendedToolId" variant="secondary" class="text-[9px] px-1 py-0">
                推荐
              </Badge>
            </button>
          </div>
        </div>

        <!-- Quick Clipboard Test Button -->
        <div class="pt-2 border-t border-border">
          <Button variant="outline" size="sm" class="w-full text-xs" @click="handleFetchClipboard">
            <IconCopy class="h-3.5 w-3.5 mr-1" />
            读取剪贴板
          </Button>
        </div>
      </aside>

      <!-- Center / Content Pane -->
      <main class="flex-1 flex flex-col overflow-hidden">
        <!-- Tab: Workspace -->
        <div v-if="activeTab === 'workspace'" class="flex-1 flex flex-col overflow-hidden">
          <!-- Input Bar if no payload or manual override -->
          <div class="p-3 border-b border-border flex items-center space-x-2 bg-background">
            <Input
              id="search-or-input"
              v-model="manualTextInput"
              placeholder="输入文本、JSON、Base64 或时间戳进行手动嗅探处理 (Ctrl+K)..."
              class="text-xs h-8"
              @keydown.enter="handleManualProcess"
            />
            <Button size="sm" class="h-8 text-xs" @click="handleManualProcess">
              处理
            </Button>
          </div>

          <!-- Dual Column: Source vs Result -->
          <div class="flex-1 grid grid-cols-2 divide-x divide-border overflow-hidden">
            <!-- Left: Source Payload -->
            <div class="flex flex-col overflow-hidden p-3 bg-muted/5">
              <div class="flex items-center justify-between pb-2 text-xs font-medium text-muted-foreground">
                <span>输入原稿 ({{ payloadStore.payloadType || '未载入' }})</span>
                <span v-if="payloadStore.currentPayload" class="text-[10px]">
                  {{ payloadStore.currentPayload.actualContent.length }} 字符
                </span>
              </div>
              <div class="flex-1 overflow-auto rounded border border-border bg-background p-2.5 font-mono text-xs select-text">
                <pre v-if="payloadStore.showOriginalDecoded" class="whitespace-pre-wrap break-all text-muted-foreground">{{ payloadStore.currentPayload?.rawOriginal }}</pre>
                <pre v-else class="whitespace-pre-wrap break-all">{{ payloadStore.currentPayload?.actualContent || '暂无内容，请在系统任意处复制或点击下方读取剪贴板。' }}</pre>
              </div>
            </div>

            <!-- Right: Result Output -->
            <div class="flex flex-col overflow-hidden p-3">
              <div class="flex items-center justify-between pb-2 text-xs font-medium text-muted-foreground">
                <div class="flex items-center space-x-1.5">
                  <span>{{ toolStore.activeTool?.name || '处理结果' }}</span>
                  <Badge variant="outline" class="text-[10px]">{{ toolStore.activeTool?.type }}</Badge>
                </div>
                <div class="flex items-center space-x-1">
                  <Button size="sm" variant="ghost" class="h-6 text-xs px-2" @click="copyResult">
                    <IconCheck v-if="copiedFeedback" class="h-3 w-3 mr-1 text-green-500" />
                    <IconCopy v-else class="h-3 w-3 mr-1" />
                    {{ copiedFeedback ? '已复制' : '复制结果' }}
                  </Button>
                </div>
              </div>
              <div class="flex-1 overflow-auto rounded border border-border bg-background p-2.5 font-mono text-xs select-text">
                <div v-if="toolStore.executionError" class="text-destructive">
                  {{ toolStore.executionError }}
                </div>
                <pre v-else class="whitespace-pre-wrap break-all">{{ toolStore.executionOutput || '等待处理结果...' }}</pre>
              </div>

              <!-- Post-action File Saved Banner -->
              <div
                v-if="toolStore.lastSavedFilePath"
                class="mt-2 flex items-center justify-between rounded bg-muted/40 px-2.5 py-1.5 text-xs text-muted-foreground"
              >
                <span class="truncate">已自动保存: {{ toolStore.lastSavedFilePath }}</span>
                <Button
                  variant="ghost"
                  size="sm"
                  class="h-5 px-1 text-[11px]"
                  @click="openSavedFolder(toolStore.lastSavedFilePath!)"
                >
                  <IconFolder class="h-3 w-3 mr-1" /> 打开目录
                </Button>
              </div>
            </div>
          </div>
        </div>

        <!-- Tab: History -->
        <div v-else-if="activeTab === 'history'" class="flex-1 flex flex-col p-4 overflow-hidden">
          <div class="flex items-center justify-between pb-3">
            <h2 class="text-sm font-semibold">历史记录</h2>
            <div class="flex items-center space-x-2">
              <Button size="sm" variant="outline" class="h-7 text-xs" @click="historyStore.clearAll">
                清空历史
              </Button>
              <Button size="sm" class="h-7 text-xs" @click="activeTab = 'workspace'">
                返回工作区
              </Button>
            </div>
          </div>

          <div class="flex-1 overflow-auto space-y-2">
            <div
              v-for="item in historyStore.records"
              :key="item.id"
              class="flex items-center justify-between p-2.5 rounded-lg border border-border bg-card hover:bg-accent/40 text-xs transition-colors"
            >
              <div class="space-y-1 truncate pr-2">
                <div class="flex items-center space-x-2">
                  <span class="font-medium text-foreground">{{ item.toolName }}</span>
                  <Badge variant="secondary" class="text-[10px]">{{ item.payloadType }}</Badge>
                  <span class="text-muted-foreground text-[10px]">{{ new Date(item.createdAt).toLocaleTimeString() }}</span>
                </div>
                <div class="truncate text-muted-foreground font-mono">{{ item.inputSummary }}</div>
              </div>
              <div class="flex items-center space-x-1 shrink-0">
                <Button
                  size="sm"
                  variant="ghost"
                  class="h-6 text-xs"
                  @click="() => {
                    if (item.inputText) {
                      payloadStore.processText(item.inputText);
                      activeTab = 'workspace';
                    }
                  }"
                >
                  载入
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  class="h-6 text-xs text-destructive hover:text-destructive"
                  @click="historyStore.deleteRecord(item.id)"
                >
                  删除
                </Button>
              </div>
            </div>

            <div v-if="historyStore.records.length === 0" class="text-center py-12 text-xs text-muted-foreground">
              暂无历史记录
            </div>
          </div>
        </div>

        <!-- Tab: Settings -->
        <div v-else-if="activeTab === 'settings'" class="flex-1 p-6 overflow-auto space-y-6">
          <div class="flex items-center justify-between border-b border-border pb-3">
            <h2 class="text-sm font-semibold">系统配置</h2>
            <Button size="sm" class="h-7 text-xs" @click="activeTab = 'workspace'">
              返回工作区
            </Button>
          </div>

          <div class="space-y-4 max-w-xl text-xs">
            <div class="space-y-2">
              <label class="font-medium text-foreground">外观主题</label>
              <div class="flex space-x-2">
                <Button
                  v-for="theme in (['system', 'light', 'dark'] as const)"
                  :key="theme"
                  size="sm"
                  :variant="settingsStore.settings.theme === theme ? 'default' : 'outline'"
                  class="text-xs h-7 capitalize"
                  @click="settingsStore.updateSettings({ theme })"
                >
                  {{ theme }}
                </Button>
              </div>
            </div>

            <div class="space-y-2 pt-2 border-t border-border">
              <label class="font-medium text-foreground">图片缓存管控</label>
              <div class="flex items-center space-x-3">
                <Button size="sm" variant="outline" class="h-7 text-xs" @click="settingsStore.fetchCacheStats">
                  查看缓存统计
                </Button>
                <Button size="sm" variant="destructive" class="h-7 text-xs" @click="settingsStore.cleanupCache(0, true)">
                  清理全部图片缓存
                </Button>
              </div>
              <div v-if="settingsStore.imageCacheStats" class="mt-2 text-muted-foreground">
                文件数: {{ settingsStore.imageCacheStats.fileCount }}, 总占用: {{ (settingsStore.imageCacheStats.totalBytes / 1024).toFixed(1) }} KB
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>

    <!-- Bottom Status Bar -->
    <footer class="flex h-7 shrink-0 items-center justify-between border-t border-border px-3 text-[11px] text-muted-foreground bg-muted/20">
      <div class="flex items-center space-x-3">
        <span>当前工具: <strong class="text-foreground">{{ toolStore.activeTool?.name }}</strong></span>
        <span v-if="toolStore.isExecuting" class="text-primary animate-pulse">执行中...</span>
      </div>
      <div class="flex items-center space-x-3">
        <span>Ctrl+K 聚焦输入</span>
        <span>Ctrl+H 历史</span>
        <span>Ctrl+Enter 运行</span>
      </div>
    </footer>
  </div>
</template>
