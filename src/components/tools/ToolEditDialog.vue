<script setup lang="ts">
import { ref } from 'vue';
import { useToolStore } from '@/stores/tools';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { IconX } from '@tabler/icons-vue';
import type { ToolDefinition, ToolType, PostActionType } from '@/types';

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const toolStore = useToolStore();

const toolForm = ref<ToolDefinition>({
  id: `custom-tool-${Date.now()}`,
  name: '新自定义工具',
  icon: 'IconCode',
  description: '执行自定义处理与后置逻辑',
  category: 'developer',
  isCustom: true,
  enabled: true,
  sortOrder: 99,
  matcher: {
    acceptedTypes: ['text'],
    basePriority: 40,
  },
  type: 'code',
  postAction: {
    type: 'none',
    saveConfig: {
      directory: 'custom_output',
      extension: 'txt',
    },
  },
  cliConfig: {
    command: 'cat',
    args: [],
    stdinMode: 'pipe',
    timeoutMs: 5000,
  },
  llmConfig: {
    useSystemProvider: true,
    systemPrompt: '请处理输入的文本并返回清晰的结果。',
    userPromptTemplate: '{{input}}',
    stream: true,
  },
});

async function handleSave() {
  await toolStore.saveTool(toolForm.value);
  toolStore.setActiveTool(toolForm.value.id);
  emit('close');
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-2xs select-none p-4 animate-in fade-in duration-150">
    <div class="flex h-[580px] w-full max-w-lg flex-col rounded-xl border border-border bg-background shadow-2xl overflow-hidden">
      <!-- Header -->
      <div class="flex h-12 shrink-0 items-center justify-between border-b border-border px-5 bg-muted/20">
        <h2 class="text-sm font-semibold text-foreground">新增 / 编辑工具</h2>
        <Button
          variant="ghost"
          size="sm"
          class="h-7 w-7 p-0 cursor-pointer text-muted-foreground hover:text-foreground"
          @click="emit('close')"
        >
          <IconX class="h-4 w-4" />
        </Button>
      </div>

      <!-- Form Body -->
      <div class="flex-1 overflow-y-auto p-5 space-y-4 text-xs">
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1">
            <label class="font-medium text-foreground">工具名称</label>
            <Input v-model="toolForm.name" class="h-8 text-xs" />
          </div>
          <div class="space-y-1">
            <label class="font-medium text-foreground">分类</label>
            <select
              v-model="toolForm.category"
              class="flex h-8 w-full rounded-md border border-input bg-background px-2 text-xs"
            >
              <option value="developer">开发工具 (developer)</option>
              <option value="text">文本处理 (text)</option>
              <option value="ai">智能助理 (ai)</option>
              <option value="utilities">通用工具 (utilities)</option>
            </select>
          </div>
        </div>

        <div class="space-y-1">
          <label class="font-medium text-foreground">功能描述</label>
          <Input v-model="toolForm.description" class="h-8 text-xs" />
        </div>

        <!-- Tool Type -->
        <div class="space-y-1.5 pt-2 border-t border-border">
          <label class="font-medium text-foreground">引擎执行类型</label>
          <div class="grid grid-cols-3 gap-2">
            <button
              v-for="t in (['code', 'llm', 'cli'] as ToolType[])"
              :key="t"
              type="button"
              @click="toolForm.type = t"
              :class="[
                'p-2 rounded border text-xs capitalize cursor-pointer transition-colors',
                toolForm.type === t
                  ? 'border-primary bg-primary/10 text-primary font-medium'
                  : 'border-border bg-card text-muted-foreground hover:bg-muted'
              ]"
            >
              {{ t }} 引擎
            </button>
          </div>
        </div>

        <!-- Type Specific Config -->
        <div v-if="toolForm.type === 'cli' && toolForm.cliConfig" class="space-y-3 rounded-lg border border-border bg-muted/10 p-3">
          <div class="space-y-1">
            <label class="text-[11px] text-muted-foreground">命令可执行文件 (如 jq, prettier, curl)</label>
            <Input v-model="toolForm.cliConfig.command" class="h-8 text-xs font-mono" />
          </div>
          <div class="space-y-1">
            <label class="text-[11px] text-muted-foreground">输入传递模式</label>
            <select
              v-model="toolForm.cliConfig.stdinMode"
              class="flex h-8 w-full rounded-md border border-input bg-background px-2 text-xs font-mono"
            >
              <option value="pipe">管道 (stdin)</option>
              <option value="none">无 (仅执行命令)</option>
            </select>
          </div>
        </div>

        <div v-else-if="toolForm.type === 'llm' && toolForm.llmConfig" class="space-y-3 rounded-lg border border-border bg-muted/10 p-3">
          <div class="space-y-1">
            <label class="text-[11px] text-muted-foreground">System Prompt (系统提示词)</label>
            <textarea
              v-model="toolForm.llmConfig.systemPrompt"
              rows="3"
              class="w-full rounded border border-border bg-background p-2 text-xs font-mono"
            />
          </div>
        </div>

        <!-- Post Action -->
        <div class="space-y-2 pt-2 border-t border-border">
          <label class="font-medium text-foreground">后置执行动作 (Post-Action)</label>
          <div class="grid grid-cols-3 gap-2">
            <button
              v-for="pa in (['none', 'copy_to_clipboard', 'save_to_file'] as PostActionType[])"
              :key="pa"
              type="button"
              @click="toolForm.postAction.type = pa"
              :class="[
                'p-2 rounded border text-xs cursor-pointer transition-colors',
                toolForm.postAction.type === pa
                  ? 'border-primary bg-primary/10 text-primary font-medium'
                  : 'border-border bg-card text-muted-foreground hover:bg-muted'
              ]"
            >
              {{ pa === 'none' ? '无动作' : pa === 'copy_to_clipboard' ? '自动复制' : '落盘文件' }}
            </button>
          </div>

          <div
            v-if="toolForm.postAction.type === 'save_to_file' && toolForm.postAction.saveConfig"
            class="space-y-2 rounded-lg border border-border bg-muted/10 p-3 mt-2"
          >
            <div class="space-y-1">
              <label class="text-[11px] text-muted-foreground">专属子目录</label>
              <Input v-model="toolForm.postAction.saveConfig.directory" class="h-8 text-xs font-mono" />
            </div>
            <div class="space-y-1">
              <label class="text-[11px] text-muted-foreground">文件扩展名 (如 json, md, txt)</label>
              <Input v-model="toolForm.postAction.saveConfig.extension" class="h-8 text-xs font-mono" />
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex h-12 shrink-0 items-center justify-end space-x-2 border-t border-border px-5 bg-muted/20">
        <Button variant="ghost" size="sm" class="h-7 text-xs cursor-pointer" @click="emit('close')">
          取消
        </Button>
        <Button size="sm" class="h-7 text-xs cursor-pointer" @click="handleSave">
          保存工具
        </Button>
      </div>
    </div>
  </div>
</template>

