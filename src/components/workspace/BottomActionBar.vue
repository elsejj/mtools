<script setup lang="ts">
import { ref, computed } from "vue";
import { useToolStore } from "@/stores/tools";
import { usePayloadStore } from "@/stores/payload";
import { Button } from "@/components/ui/button";
import { IconCopy, IconCheck, IconArrowBackUp, IconPlayerPlay } from "@tabler/icons-vue";
import { tauriApi } from "@/lib/tauri";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { copyContentToClipboard, markdownToRichHtml } from "@/lib/clipboard";

const toolStore = useToolStore();
const payloadStore = usePayloadStore();

const copied = ref(false);
const pasted = ref(false);

const isMarkdownMode = computed(() => {
  if (toolStore.activeTool?.type === "cli") return false;
  if (
    toolStore.activeTool?.id === "json-formatter" ||
    toolStore.activeTool?.id === "jwt-inspector"
  ) {
    return false;
  }
  return true;
});

const isPreviewMode = computed(() => {
  return isMarkdownMode.value && toolStore.isMarkdownPreview;
});

async function handleCopy() {
  if (!toolStore.executionOutput) return;
  try {
    const asHtml = isPreviewMode.value;
    await copyContentToClipboard(toolStore.executionOutput, asHtml);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch (err) {
    console.error("Failed to copy to clipboard:", err);
  }
}

async function handlePasteBack() {
  if (!toolStore.executionOutput) return;
  try {
    pasted.value = true;
    const text = toolStore.executionOutput;
    const html = isPreviewMode.value ? markdownToRichHtml(text) : undefined;

    // Minimize or hide window before simulating paste so target window gains focus
    try {
      const appWindow = getCurrentWindow();
      await appWindow.hide();
    } catch {
      // Not in Tauri webview window
    }

    // Give OS focus shift delay (~100ms)
    setTimeout(async () => {
      await tauriApi.simulatePaste(text, html);
      pasted.value = false;
    }, 120);
  } catch (err) {
    console.error("Failed to simulate paste:", err);
    pasted.value = false;
  }
}

async function handleRerun() {
  if (payloadStore.currentPayload) {
    await toolStore.executeTool(payloadStore.currentPayload);
  }
}
</script>

<template>
  <footer
    class="flex h-10 shrink-0 items-center justify-between border-t border-border bg-muted/20 px-3 text-xs select-none"
  >
    <!-- Left Actions -->
    <div class="flex items-center space-x-2">
      <Button
        size="sm"
        class="h-7 text-xs px-2.5 cursor-pointer shadow-2xs"
        :disabled="!toolStore.executionOutput"
        @click="handleCopy"
        :title="
          isPreviewMode
            ? '复制为 HTML 富文本 (可直接粘贴表格至 Word/Excel/微信/飞书)'
            : '复制为纯文本源码'
        "
      >
        <IconCheck v-if="copied" class="h-3.5 w-3.5 mr-1 text-green-400" />
        <IconCopy v-else class="h-3.5 w-3.5 mr-1" />
        <span>{{ copied ? (isPreviewMode ? "已复制 HTML" : "已复制") : "复制结果" }}</span>
      </Button>

      <Button
        variant="outline"
        size="sm"
        class="h-7 text-xs px-2.5 cursor-pointer hover:bg-muted"
        :disabled="!toolStore.executionOutput"
        @click="handlePasteBack"
        title="将结果直接粘贴回呼出前的活动应用程序"
      >
        <IconArrowBackUp class="h-3.5 w-3.5 mr-1 text-primary" />
        <span>{{ pasted ? "已回贴..." : "回贴源软件" }}</span>
      </Button>

      <Button
        variant="ghost"
        size="sm"
        class="h-7 text-xs px-2 cursor-pointer text-muted-foreground hover:text-foreground"
        :disabled="!payloadStore.hasPayload || toolStore.isExecuting"
        @click="handleRerun"
        title="重新运行当前工具 (Ctrl+Enter)"
      >
        <IconPlayerPlay class="h-3.5 w-3.5 mr-1" />
        <span>重新运行</span>
        <kbd class="ml-1 text-[10px] opacity-70 font-mono">↵</kbd>
      </Button>
    </div>

    <!-- Right Status & Shortcuts Guide -->
    <div class="flex items-center space-x-3 text-[11px] text-muted-foreground">
      <div class="hidden md:flex items-center space-x-2">
        <span><kbd class="font-mono bg-muted px-1 rounded">Ctrl+K</kbd> 聚焦输入</span>
        <span><kbd class="font-mono bg-muted px-1 rounded">Alt+1~5</kbd> 候选工具</span>
        <span><kbd class="font-mono bg-muted px-1 rounded">Ctrl+H</kbd> 历史</span>
      </div>
    </div>
  </footer>
</template>
