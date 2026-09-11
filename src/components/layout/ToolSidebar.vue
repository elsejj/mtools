<script setup lang="ts">
import { ref, computed } from 'vue';
import { useToolStore } from '@/stores/tools';
import { usePayloadStore } from '@/stores/payload';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import {
  IconCode,
  IconSparkles,
  IconTerminal2,
  IconFileText,
  IconPlus,
  IconClipboard,
  IconSearch,
  IconCategory,
} from '@tabler/icons-vue';

const emit = defineEmits<{
  (e: 'openToolEditor'): void;
  (e: 'openToolManager'): void;
}>();

const toolStore = useToolStore();
const payloadStore = usePayloadStore();

const searchQuery = ref('');
const selectedCategory = ref<string>('all');

const categories = [
  { id: 'all', name: '全部' },
  { id: 'developer', name: '开发' },
  { id: 'text', name: '文本' },
  { id: 'ai', name: '智能' },
  { id: 'utilities', name: '通用' },
];

const filteredTools = computed(() => {
  return toolStore.tools.filter((t) => {
    if (!t.enabled) return false;
    const matchesCategory =
      selectedCategory.value === 'all' || t.category === selectedCategory.value;
    const matchesQuery =
      searchQuery.value === '' ||
      t.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      t.description.toLowerCase().includes(searchQuery.value.toLowerCase());
    return matchesCategory && matchesQuery;
  });
});

function handleSelectTool(toolId: string) {
  toolStore.setActiveTool(toolId);
  if (payloadStore.currentPayload) {
    toolStore.executeTool(payloadStore.currentPayload);
  }
}

async function handleReadClipboard() {
  await payloadStore.fetchFromClipboard();
}
</script>

<template>
  <aside class="w-56 shrink-0 border-r border-border bg-muted/10 flex flex-col justify-between select-none overflow-hidden">
    <!-- Top Part: Search & Category Pills -->
    <div class="p-2 space-y-2 border-b border-border">
      <div class="relative">
        <IconSearch class="absolute left-2.5 top-2 h-3.5 w-3.5 text-muted-foreground" />
        <Input
          v-model="searchQuery"
          placeholder="过滤工具..."
          class="h-7 pl-8 text-xs bg-background"
        />
      </div>

      <!-- Categories Pills -->
      <div class="flex items-center space-x-1 overflow-x-auto pb-1">
        <button
          v-for="cat in categories"
          :key="cat.id"
          type="button"
          @click="selectedCategory = cat.id"
          :class="[
            'px-2 py-0.5 rounded text-[11px] whitespace-nowrap transition-colors cursor-pointer',
            selectedCategory === cat.id
              ? 'bg-primary text-primary-foreground font-medium'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground'
          ]"
        >
          {{ cat.name }}
        </button>
      </div>
    </div>

    <!-- Middle: Tool List -->
    <div class="flex-1 overflow-y-auto p-2 space-y-0.5">
      <button
        v-for="tool in filteredTools"
        :key="tool.id"
        type="button"
        @click="handleSelectTool(tool.id)"
        :class="[
          'flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-xs transition-colors cursor-pointer',
          toolStore.activeToolId === tool.id
            ? 'bg-accent text-accent-foreground font-medium shadow-2xs'
            : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'
        ]"
      >
        <div class="flex items-center space-x-2 truncate pr-1">
          <IconCode v-if="tool.type === 'code'" class="h-3.5 w-3.5 shrink-0 text-sky-500" />
          <IconSparkles v-else-if="tool.type === 'llm'" class="h-3.5 w-3.5 shrink-0 text-violet-500" />
          <IconTerminal2 v-else-if="tool.type === 'cli'" class="h-3.5 w-3.5 shrink-0 text-emerald-500" />
          <IconFileText v-else class="h-3.5 w-3.5 shrink-0" />
          <span class="truncate">{{ tool.name }}</span>
        </div>

        <Badge
          v-if="tool.id === payloadStore.recommendedToolId"
          variant="secondary"
          class="text-[9px] px-1 py-0 bg-primary/10 text-primary border-primary/20 shrink-0"
        >
          推荐
        </Badge>
      </button>

      <div v-if="filteredTools.length === 0" class="py-6 text-center text-xs text-muted-foreground">
        未找到相关工具
      </div>
    </div>

    <!-- Bottom Actions: Add Custom Tool, Manage Tools, Read Clipboard -->
    <div class="p-2 border-t border-border space-y-1.5 bg-background/50">
      <div class="grid grid-cols-2 gap-1.5">
        <Button
          variant="outline"
          size="sm"
          class="text-xs h-7 justify-center px-1"
          @click="emit('openToolEditor')"
        >
          <IconPlus class="h-3 w-3 mr-1" />
          新建工具
        </Button>
        <Button
          variant="outline"
          size="sm"
          class="text-xs h-7 justify-center px-1"
          @click="emit('openToolManager')"
        >
          <IconCategory class="h-3 w-3 mr-1" />
          管理工具
        </Button>
      </div>
      <Button
        variant="ghost"
        size="sm"
        class="w-full text-xs h-7 justify-center text-muted-foreground hover:text-foreground"
        @click="handleReadClipboard"
      >
        <IconClipboard class="h-3.5 w-3.5 mr-1" />
        读取剪贴板
      </Button>
    </div>
  </aside>
</template>

