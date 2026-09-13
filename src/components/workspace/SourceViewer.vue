<script setup lang="ts">
import { computed, ref } from "vue";
import { usePayloadStore } from "@/stores/payload";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  IconSparkles,
  IconPhoto,
  IconFileText,
  IconTrash,
  IconEye,
  IconFolder,
  IconMaximize,
  IconMinimize,
} from "@tabler/icons-vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { tauriApi } from "@/lib/tauri";
import type { ImageMetadata } from "@/types";

const payloadStore = usePayloadStore();

const naturalWidth = ref(0);
const naturalHeight = ref(0);
const isOriginalSize = ref(false);

const imageMetadata = computed<ImageMetadata | null>(() => {
  if (payloadStore.currentPayload?.payloadType === "image") {
    return payloadStore.currentPayload.metadata as ImageMetadata;
  }
  return null;
});

const imageSrc = computed<string>(() => {
  // If actualContent is already a data URI, display it directly
  if (payloadStore.currentPayload?.payloadType === "image") {
    if (payloadStore.currentPayload.actualContent?.startsWith("data:image/")) {
      return payloadStore.currentPayload.actualContent;
    }
  }
  if (!imageMetadata.value) return "";
  if (imageMetadata.value.localCachePath) {
    try {
      return convertFileSrc(imageMetadata.value.localCachePath);
    } catch {
      return "";
    }
  }
  return "";
});

const displayWidth = computed(() => {
  return imageMetadata.value?.width || naturalWidth.value || 0;
});

const displayHeight = computed(() => {
  return imageMetadata.value?.height || naturalHeight.value || 0;
});

function onImageLoad(event: Event) {
  const img = event.target as HTMLImageElement;
  if (img) {
    naturalWidth.value = img.naturalWidth;
    naturalHeight.value = img.naturalHeight;
  }
}

const charCount = computed(() => {
  return payloadStore.currentPayload?.actualContent.length ?? 0;
});

const lineCount = computed(() => {
  if (!payloadStore.currentPayload?.actualContent) return 0;
  return payloadStore.currentPayload.actualContent.split("\n").length;
});

function openImageDir() {
  if (imageMetadata.value?.localCachePath) {
    tauriApi.showInFolder(imageMetadata.value.localCachePath);
  }
}
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden bg-muted/5 select-none">
    <!-- Header info bar -->
    <div
      class="flex h-9 shrink-0 items-center justify-between border-b border-border px-3 bg-muted/20 text-xs"
    >
      <div class="flex items-center space-x-2">
        <IconPhoto v-if="payloadStore.payloadType === 'image'" class="h-3.5 w-3.5 text-sky-500" />
        <IconFileText v-else class="h-3.5 w-3.5 text-muted-foreground" />
        <span class="font-medium text-foreground">输入原稿</span>
        <Badge variant="outline" class="text-[10px] px-1 py-0 uppercase">
          {{ payloadStore.payloadType || "未载入" }}
        </Badge>
      </div>

      <div class="flex items-center space-x-2 text-[11px] text-muted-foreground">
        <span v-if="payloadStore.payloadType === 'text' && charCount > 0">
          {{ charCount }} 字符 / {{ lineCount }} 行
        </span>
        <Button
          v-if="payloadStore.hasPayload"
          variant="ghost"
          size="sm"
          class="h-6 w-6 p-0 text-muted-foreground hover:text-destructive cursor-pointer"
          title="清空载荷"
          @click="payloadStore.reset"
        >
          <IconTrash class="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>

    <!-- Decoding Trace Banner -->
    <div
      v-if="payloadStore.hasDecodingTrace"
      class="flex items-center justify-between border-b border-amber-500/20 bg-amber-500/10 px-3 py-1.5 text-xs text-amber-700 dark:text-amber-300 shrink-0"
    >
      <div class="flex items-center space-x-1.5 overflow-hidden">
        <IconSparkles class="h-3.5 w-3.5 shrink-0" />
        <span class="truncate">检测到多层编码，已自动解包：</span>
        <div class="flex items-center space-x-1 shrink-0">
          <Badge
            v-for="(trace, i) in payloadStore.decodingTrace"
            :key="i"
            variant="outline"
            class="text-[10px] px-1.5 py-0 bg-background/80 border-amber-500/30"
          >
            {{ trace }}
          </Badge>
        </div>
      </div>

      <button
        type="button"
        @click="payloadStore.toggleOriginalDecoded"
        class="ml-2 shrink-0 cursor-pointer text-[11px] font-medium underline hover:opacity-80 flex items-center space-x-1"
      >
        <IconEye class="h-3 w-3" />
        <span>{{ payloadStore.showOriginalDecoded ? "显示解码内容" : "查看解码前原文" }}</span>
      </button>
    </div>

    <!-- Content Display -->
    <div class="flex-1 overflow-hidden p-2.5 select-text flex flex-col min-h-0">
      <!-- Image Payload View -->
      <div
        v-if="payloadStore.payloadType === 'image'"
        class="flex flex-col h-full w-full min-h-0 overflow-hidden"
      >
        <!-- Adaptive Image Stage -->
        <div
          class="flex-1 min-h-0 w-full flex items-center justify-center p-1.5 rounded-md border border-border/80 bg-background/50 shadow-2xs transition-all"
          :class="isOriginalSize ? 'overflow-auto' : 'overflow-hidden'"
        >
          <img
            v-if="imageSrc"
            :src="imageSrc"
            alt="Clipboard Image"
            :class="[
              'rounded transition-all',
              isOriginalSize
                ? 'max-w-none max-h-none object-none'
                : 'max-w-full max-h-full w-auto h-auto object-contain',
            ]"
            @load="onImageLoad"
          />
          <div v-else class="text-xs text-muted-foreground p-8 text-center">
            图片加载中或无物理缓存
          </div>
        </div>

        <!-- Image Info Badges -->
        <div
          v-if="imageMetadata"
          class="shrink-0 flex flex-wrap items-center justify-center gap-1.5 pt-2 text-xs"
        >
          <Badge variant="secondary" class="text-[11px]">
            尺寸: {{ displayWidth }} × {{ displayHeight }}
          </Badge>
          <Badge variant="secondary" class="text-[11px]">
            类型: {{ imageMetadata.mimeType }}
          </Badge>
          <Badge variant="secondary" class="text-[11px]">
            大小: {{ (imageMetadata.byteSize / 1024).toFixed(1) }} KB
          </Badge>
          <Button
            variant="ghost"
            size="sm"
            class="h-6 px-1.5 text-[11px] cursor-pointer hover:bg-muted"
            @click="isOriginalSize = !isOriginalSize"
            :title="isOriginalSize ? '自适应分栏显示' : '以 1:1 原始像素显示'"
          >
            <IconMinimize v-if="isOriginalSize" class="h-3 w-3 mr-1" />
            <IconMaximize v-else class="h-3 w-3 mr-1" />
            <span>{{ isOriginalSize ? "自适应分栏" : "原始比例" }}</span>
          </Button>
          <Button
            v-if="imageMetadata.localCachePath"
            variant="ghost"
            size="sm"
            class="h-6 px-1.5 text-[11px] cursor-pointer hover:bg-muted"
            @click="openImageDir"
          >
            <IconFolder class="h-3 w-3 mr-1" /> 定位缓存文件
          </Button>
        </div>
      </div>

      <!-- Text Payload View -->
      <div v-else class="h-full overflow-auto font-mono text-[13px] leading-relaxed antialiased">
        <pre
          v-if="payloadStore.showOriginalDecoded"
          class="whitespace-pre-wrap break-words [overflow-wrap:anywhere] text-muted-foreground bg-muted/20 p-2.5 rounded border border-border"
          >{{ payloadStore.currentPayload?.rawOriginal }}</pre>
        <pre
          v-else-if="payloadStore.hasPayload"
          class="whitespace-pre-wrap break-words [overflow-wrap:anywhere]"
          >{{ payloadStore.currentPayload?.actualContent }}</pre>
        <div
          v-else
          class="flex h-full flex-col items-center justify-center text-center text-xs text-muted-foreground select-none"
        >
          <IconFileText class="h-8 w-8 mb-2 opacity-30" />
          <p class="font-medium">暂无载荷内容</p>
          <p class="text-[11px] opacity-70 mt-1">
            在任意软件中选中文本并复制，或点击上方“读取剪贴板”
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
