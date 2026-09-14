<script setup lang="ts">
import { onMounted, onUnmounted, watch, ref } from "vue";
import { usePayloadStore } from "@/stores/payload";
import { useToolStore } from "@/stores/tools";
import { useSettingsStore } from "@/stores/settings";
import { useHistoryStore } from "@/stores/history";
import { useWindowState } from "@/composables/useWindowState";
import { useShortcuts } from "@/composables/useShortcuts";

// Layout Components
import HeaderBar from "@/components/layout/HeaderBar.vue";
import ToolSidebar from "@/components/layout/ToolSidebar.vue";
import SourceViewer from "@/components/workspace/SourceViewer.vue";
import ResultViewer from "@/components/workspace/ResultViewer.vue";
import BottomActionBar from "@/components/workspace/BottomActionBar.vue";

// Modal & Drawer Components
import HistoryDrawer from "@/components/history/HistoryDrawer.vue";
import SettingsModal from "@/components/settings/SettingsModal.vue";
import ToolEditDialog from "@/components/tools/ToolEditDialog.vue";
import ToolManagerModal from "@/components/tools/ToolManagerModal.vue";

import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

// Stores
const payloadStore = usePayloadStore();
const toolStore = useToolStore();
const settingsStore = useSettingsStore();
const historyStore = useHistoryStore();

// Track window movement and resizing
useWindowState();

// UI Overlays state
const showHistory = ref(false);
const showSettings = ref(false);
const showToolEditor = ref(false);
const showToolManager = ref(false);
const manualInputText = ref("");

// Auto-select best tool when new payload is emitted
watch(
  () => payloadStore.currentPayload,
  (newPayload) => {
    if (newPayload) {
      toolStore.autoSelectBestTool(newPayload.recommendedToolId, newPayload.candidateToolScores);
      toolStore.executeTool(newPayload);
    }
  },
);

// Register Global Hotkeys
useShortcuts({
  onSearch: () => {
    const inputEl = document.getElementById("manual-input-box");
    inputEl?.focus();
  },
  onHistory: () => {
    showHistory.value = !showHistory.value;
    if (showHistory.value) {
      historyStore.fetchRecords(true);
    }
  },
  onSettings: () => {
    showSettings.value = !showSettings.value;
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
    if (showToolEditor.value) showToolEditor.value = false;
    else if (showToolManager.value) showToolManager.value = false;
    else if (showHistory.value) showHistory.value = false;
    else if (showSettings.value) showSettings.value = false;
  },
});

async function handleManualProcess() {
  if (!manualInputText.value.trim()) return;
  await payloadStore.processText(manualInputText.value);
}

onMounted(async () => {
  await settingsStore.loadSettings();
  await toolStore.loadTools();
  await payloadStore.startListening();
});

onUnmounted(() => {
  payloadStore.stopListening();
});

function focusSearch() {
  document.getElementById("manual-input-box")?.focus();
}
</script>

<template>
  <div
    class="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground font-sans"
  >
    <!-- 1. Header Bar -->
    <HeaderBar
      :active-tab="showHistory ? 'history' : showSettings ? 'settings' : 'workspace'"
      @toggle-history="showHistory = !showHistory"
      @toggle-settings="showSettings = !showSettings"
      @focus-search="focusSearch"
    />

    <!-- 2. Main Body: Sidebar + Dual Workspace -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Left Tool Sidebar -->
      <ToolSidebar
        @open-tool-editor="
          () => {
            toolStore.setEditingTool(null);
            showToolEditor = true;
          }
        "
        @open-tool-manager="showToolManager = true"
      />

      <!-- Center Dual-Column Workspace -->
      <main class="flex-1 flex flex-col overflow-hidden">
        <!-- Manual Input / Quick Sniff Bar -->
        <div
          class="flex items-center space-x-2 border-b border-border bg-background px-3 py-2 shrink-0"
        >
          <Input
            id="manual-input-box"
            v-model="manualInputText"
            placeholder="粘贴或输入文本、JSON、Base64 或时间戳进行嗅探 (Ctrl+K)..."
            class="h-7 text-xs"
            @keydown.enter="handleManualProcess"
          />
          <Button
            size="sm"
            class="h-7 text-xs px-3 shrink-0 cursor-pointer"
            @click="handleManualProcess"
          >
            嗅探并处理
          </Button>
        </div>

        <!-- Two Columns: Source (Left) vs Result (Right) -->
        <div class="flex-1 grid grid-cols-2 divide-x divide-border overflow-hidden">
          <SourceViewer />
          <ResultViewer />
        </div>
      </main>
    </div>

    <!-- 3. Bottom Action Bar -->
    <BottomActionBar />

    <!-- 4. Drawers & Modals -->
    <HistoryDrawer v-if="showHistory" @close="showHistory = false" />
    <SettingsModal v-if="showSettings" @close="showSettings = false" />
    <ToolEditDialog v-if="showToolEditor" @close="showToolEditor = false" />
    <ToolManagerModal
      v-if="showToolManager"
      @close="showToolManager = false"
      @open-create="
        () => {
          showToolManager = false;
          showToolEditor = true;
        }
      "
    />
  </div>
</template>
