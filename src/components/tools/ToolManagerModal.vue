<script setup lang="ts">
import { ref } from 'vue';
import { useToolStore } from '@/stores/tools';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  IconX,
  IconPlus,
  IconDownload,
  IconUpload,
  IconArrowUp,
  IconArrowDown,
  IconEdit,
  IconTrash,
  IconCode,
  IconSparkles,
  IconTerminal2,
  IconCheck,
} from '@tabler/icons-vue';
import type { ToolDefinition } from '@/types';

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'openCreate'): void;
}>();

const toolStore = useToolStore();

const noticeMessage = ref<string | null>(null);

function showNotice(msg: string) {
  noticeMessage.value = msg;
  setTimeout(() => {
    noticeMessage.value = null;
  }, 2000);
}

function handleEdit(tool: ToolDefinition) {
  toolStore.setEditingTool(tool);
  emit('openCreate');
}

function handleCreate() {
  toolStore.setEditingTool(null);
  emit('openCreate');
}

function handleExport() {
  const json = toolStore.exportToolsToJson();
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `mtools-tools-export-${Date.now()}.json`;
  a.click();
  URL.revokeObjectURL(url);
  showNotice('已成功导出工具配置 JSON 文件');
}

async function handleImportFile(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = async (e) => {
    const content = e.target?.result as string;
    const res = await toolStore.importToolsFromJson(content);
    if (res.error) {
      showNotice(`导入失败: ${res.error}`);
    } else {
      showNotice(`成功导入并更新了 ${res.count} 个工具`);
    }
  };
  reader.readAsText(file);
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-2xs select-none p-4 animate-in fade-in duration-150">
    <div class="flex h-[600px] w-full max-w-2xl flex-col rounded-xl border border-border bg-background shadow-2xl overflow-hidden">
      <!-- Header -->
      <div class="flex h-12 shrink-0 items-center justify-between border-b border-border px-5 bg-muted/20">
        <div class="flex items-center space-x-2">
          <h2 class="text-sm font-semibold text-foreground">工具中心与管理</h2>
          <Badge variant="secondary" class="text-[10px]">
            共 {{ toolStore.tools.length }} 个工具
          </Badge>
        </div>

        <div class="flex items-center space-x-1.5">
          <Button
            size="sm"
            class="h-7 text-xs px-2.5 cursor-pointer"
            @click="handleCreate"
          >
            <IconPlus class="h-3.5 w-3.5 mr-1" />
            新建工具
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

      <!-- Notice Banner -->
      <div
        v-if="noticeMessage"
        class="flex items-center space-x-1.5 bg-primary/10 border-b border-primary/20 px-4 py-1.5 text-xs text-primary font-medium"
      >
        <IconCheck class="h-3.5 w-3.5" />
        <span>{{ noticeMessage }}</span>
      </div>

      <!-- Tool List -->
      <div class="flex-1 overflow-y-auto p-4 space-y-2">
        <div
          v-for="(tool, index) in toolStore.tools"
          :key="tool.id"
          :class="[
            'flex items-center justify-between rounded-lg border p-2.5 text-xs transition-colors',
            tool.enabled ? 'border-border bg-card' : 'border-border/60 bg-muted/30 opacity-60'
          ]"
        >
          <!-- Left: Icon & Tool Info -->
          <div class="flex items-center space-x-3 truncate pr-2">
            <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-muted/60">
              <IconCode v-if="tool.type === 'code'" class="h-4 w-4 text-sky-500" />
              <IconSparkles v-else-if="tool.type === 'llm'" class="h-4 w-4 text-violet-500" />
              <IconTerminal2 v-else class="h-4 w-4 text-emerald-500" />
            </div>

            <div class="space-y-0.5 truncate">
              <div class="flex items-center space-x-1.5">
                <span class="font-medium text-foreground">{{ tool.name }}</span>
                <Badge variant="outline" class="text-[9px] px-1 py-0 uppercase">
                  {{ tool.type }}
                </Badge>
                <Badge v-if="tool.isCustom" variant="secondary" class="text-[9px] px-1 py-0 text-amber-500">
                  自定义
                </Badge>
              </div>
              <p class="truncate text-[11px] text-muted-foreground font-mono">{{ tool.description }}</p>
            </div>
          </div>

          <!-- Right: Controls -->
          <div class="flex items-center space-x-1 shrink-0">
            <!-- Move Up -->
            <Button
              variant="ghost"
              size="sm"
              class="h-6 w-6 p-0 cursor-pointer text-muted-foreground hover:text-foreground"
              :disabled="index === 0"
              @click="toolStore.reorderTool(tool.id, 'up')"
              title="上移"
            >
              <IconArrowUp class="h-3 w-3" />
            </Button>

            <!-- Move Down -->
            <Button
              variant="ghost"
              size="sm"
              class="h-6 w-6 p-0 cursor-pointer text-muted-foreground hover:text-foreground"
              :disabled="index === toolStore.tools.length - 1"
              @click="toolStore.reorderTool(tool.id, 'down')"
              title="下移"
            >
              <IconArrowDown class="h-3 w-3" />
            </Button>

            <!-- Enable / Disable Switch -->
            <button
              type="button"
              @click="toolStore.toggleToolEnabled(tool.id)"
              :class="[
                'px-2 py-0.5 rounded text-[10px] font-medium transition-colors cursor-pointer',
                tool.enabled ? 'bg-primary/10 text-primary hover:bg-primary/20' : 'bg-muted text-muted-foreground'
              ]"
            >
              {{ tool.enabled ? '已启用' : '已停用' }}
            </button>

            <!-- Edit Button -->
            <Button
              variant="ghost"
              size="sm"
              class="h-6 w-6 p-0 cursor-pointer text-muted-foreground hover:text-foreground"
              @click="handleEdit(tool)"
              title="编辑配置"
            >
              <IconEdit class="h-3 w-3" />
            </Button>

            <!-- Delete (Only Custom) -->
            <Button
              v-if="tool.isCustom"
              variant="ghost"
              size="sm"
              class="h-6 w-6 p-0 cursor-pointer text-muted-foreground hover:text-destructive"
              @click="toolStore.deleteTool(tool.id)"
              title="删除工具"
            >
              <IconTrash class="h-3 w-3" />
            </Button>
          </div>
        </div>
      </div>

      <!-- Footer: Import & Export -->
      <div class="flex h-12 shrink-0 items-center justify-between border-t border-border px-5 bg-muted/20 text-xs">
        <div class="flex items-center space-x-2">
          <Button
            variant="outline"
            size="sm"
            class="h-7 text-xs px-2.5 cursor-pointer"
            @click="handleExport"
          >
            <IconDownload class="h-3.5 w-3.5 mr-1" />
            导出配置 JSON
          </Button>

          <label class="inline-flex items-center">
            <Button
              variant="outline"
              size="sm"
              class="h-7 text-xs px-2.5 cursor-pointer"
              as="span"
            >
              <IconUpload class="h-3.5 w-3.5 mr-1" />
              导入配置 JSON
            </Button>
            <input
              type="file"
              accept=".json"
              class="hidden"
              @change="handleImportFile"
            />
          </label>
        </div>

        <Button
          variant="ghost"
          size="sm"
          class="h-7 text-xs px-3 cursor-pointer"
          @click="emit('close')"
        >
          完成
        </Button>
      </div>
    </div>
  </div>
</template>

