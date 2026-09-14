<script setup lang="ts">
import { usePayloadStore } from "@/stores/payload";
import { useToolStore } from "@/stores/tools";
import { Button } from "@/components/ui/button";
import { IconClock, IconSettings, IconSearch, IconSparkles } from "@tabler/icons-vue";

const props = defineProps<{
  activeTab: "workspace" | "history" | "settings";
}>();

const emit = defineEmits<{
  (e: "toggleHistory"): void;
  (e: "toggleSettings"): void;
  (e: "focusSearch"): void;
}>();

const payloadStore = usePayloadStore();
const toolStore = useToolStore();

function selectCandidate(toolId: string) {
  toolStore.setActiveTool(toolId);
  if (payloadStore.currentPayload) {
    toolStore.executeTool(payloadStore.currentPayload);
  }
}
</script>

<template>
  <header
    class="flex h-12 shrink-0 items-center justify-between border-b border-border px-4 bg-muted/20 select-none"
  >
    <!-- Left: Brand + Candidates Bar -->
    <div class="flex items-center space-x-3 overflow-x-auto py-1">
      <div
        class="flex items-center space-x-1.5 font-bold tracking-tight text-primary mr-1 shrink-0"
      >
        <span
          class="flex items-center justify-center h-6 w-6 bg-primary text-primary-foreground text-xs font-mono font-bold rounded shadow-xs"
        >
          M
        </span>
        <span class="text-sm tracking-tight font-semibold">mTools</span>
      </div>

      <!-- Candidate Recommended Tools with Alt+N Badges -->
      <div class="flex items-center space-x-1.5">
        <button
          v-for="(candidate, idx) in toolStore
            .getRecommendedTools(payloadStore.candidateScores)
            .slice(0, 5)"
          :key="candidate.id"
          type="button"
          @click="selectCandidate(candidate.id)"
          :class="[
            'group flex items-center space-x-1.5 rounded-md px-2.5 py-1 text-xs transition-all cursor-pointer',
            toolStore.activeToolId === candidate.id
              ? 'bg-primary text-primary-foreground font-medium shadow-xs'
              : 'bg-secondary/70 text-secondary-foreground hover:bg-secondary hover:text-foreground',
          ]"
        >
          <span class="truncate max-w-[120px]">{{ candidate.name }}</span>
          <kbd
            :class="[
              'rounded px-1 py-0.2 text-[10px] font-mono transition-opacity',
              toolStore.activeToolId === candidate.id
                ? 'bg-white/20 text-primary-foreground'
                : 'bg-black/10 dark:bg-white/10 text-muted-foreground group-hover:text-foreground',
            ]"
          >
            Alt+{{ idx + 1 }}
          </kbd>
        </button>

        <span
          v-if="toolStore.getRecommendedTools(payloadStore.candidateScores).length === 0"
          class="text-xs text-muted-foreground flex items-center"
        >
          <IconSparkles class="h-3 w-3 mr-1 text-primary/70" />
          就绪中...
        </span>
      </div>
    </div>

    <!-- Right: Quick Search trigger, History & Settings -->
    <div class="flex items-center space-x-1.5 shrink-0">
      <button
        type="button"
        @click="emit('focusSearch')"
        class="hidden sm:flex items-center space-x-1.5 rounded-md border border-border bg-background/50 px-2.5 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground cursor-pointer"
        title="搜索工具与输入 (Ctrl+K)"
      >
        <IconSearch class="h-3.5 w-3.5" />
        <span>搜索或输入...</span>
        <kbd class="ml-1 rounded bg-muted px-1 py-0.2 text-[10px] font-mono">⌘K</kbd>
      </button>

      <Button
        variant="ghost"
        size="sm"
        :class="props.activeTab === 'history' ? 'bg-accent text-accent-foreground' : ''"
        @click="emit('toggleHistory')"
        title="历史记录 (Ctrl+H)"
      >
        <IconClock class="h-4 w-4" />
      </Button>

      <Button
        variant="ghost"
        size="sm"
        :class="props.activeTab === 'settings' ? 'bg-accent text-accent-foreground' : ''"
        @click="emit('toggleSettings')"
        title="系统设置 (Ctrl+,)"
      >
        <IconSettings class="h-4 w-4" />
      </Button>
    </div>
  </header>
</template>
