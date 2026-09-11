<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import {
  IconX,
  IconSun,
  IconMoon,
  IconDeviceDesktop,
  IconTrash,
  IconSparkles,
  IconFolder,
  IconCheck,
} from '@tabler/icons-vue';

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const settingsStore = useSettingsStore();
const activeProviderId = ref<string>('openai');
const savedNotice = ref(false);

onMounted(async () => {
  await settingsStore.loadSettings();
  await settingsStore.fetchCacheStats();
  if (settingsStore.settings.defaultProviderId) {
    activeProviderId.value = settingsStore.settings.defaultProviderId;
  }
});

async function handleThemeChange(theme: 'light' | 'dark' | 'system') {
  await settingsStore.updateSettings({ theme });
}

async function handleClearCache() {
  await settingsStore.cleanupCache(0, true);
}

async function saveCurrentSettings() {
  await settingsStore.updateSettings(settingsStore.settings);
  savedNotice.value = true;
  setTimeout(() => {
    savedNotice.value = false;
  }, 1500);
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-2xs select-none p-4 animate-in fade-in duration-150">
    <div class="flex h-[560px] w-full max-w-2xl flex-col rounded-xl border border-border bg-background shadow-2xl overflow-hidden">
      <!-- Header -->
      <div class="flex h-12 shrink-0 items-center justify-between border-b border-border px-5 bg-muted/20">
        <h2 class="text-sm font-semibold text-foreground">系统首选项与设置</h2>
        <Button
          variant="ghost"
          size="sm"
          class="h-7 w-7 p-0 cursor-pointer text-muted-foreground hover:text-foreground"
          @click="emit('close')"
        >
          <IconX class="h-4 w-4" />
        </Button>
      </div>

      <!-- Main Body -->
      <div class="flex-1 overflow-y-auto p-5 space-y-6 text-xs">
        <!-- 1. Theme -->
        <section class="space-y-2">
          <label class="font-medium text-foreground text-xs">外观主题</label>
          <div class="grid grid-cols-3 gap-3">
            <button
              type="button"
              @click="handleThemeChange('system')"
              :class="[
                'flex items-center justify-center space-x-2 rounded-lg border p-2.5 transition-all cursor-pointer',
                settingsStore.settings.theme === 'system'
                  ? 'border-primary bg-primary/10 text-primary font-medium shadow-2xs'
                  : 'border-border bg-card text-muted-foreground hover:bg-muted'
              ]"
            >
              <IconDeviceDesktop class="h-4 w-4" />
              <span>跟随系统</span>
            </button>

            <button
              type="button"
              @click="handleThemeChange('light')"
              :class="[
                'flex items-center justify-center space-x-2 rounded-lg border p-2.5 transition-all cursor-pointer',
                settingsStore.settings.theme === 'light'
                  ? 'border-primary bg-primary/10 text-primary font-medium shadow-2xs'
                  : 'border-border bg-card text-muted-foreground hover:bg-muted'
              ]"
            >
              <IconSun class="h-4 w-4" />
              <span>浅色模式</span>
            </button>

            <button
              type="button"
              @click="handleThemeChange('dark')"
              :class="[
                'flex items-center justify-center space-x-2 rounded-lg border p-2.5 transition-all cursor-pointer',
                settingsStore.settings.theme === 'dark'
                  ? 'border-primary bg-primary/10 text-primary font-medium shadow-2xs'
                  : 'border-border bg-card text-muted-foreground hover:bg-muted'
              ]"
            >
              <IconMoon class="h-4 w-4" />
              <span>深色模式</span>
            </button>
          </div>
        </section>

        <!-- 2. Storage & Cache -->
        <section class="space-y-3 pt-4 border-t border-border">
          <div class="flex items-center justify-between">
            <div>
              <label class="font-medium text-foreground text-xs">图片缓存与存储</label>
              <p class="text-[11px] text-muted-foreground mt-0.5">
                剪贴板图片自动按年月归档在缓存目录中，防止磁盘长期累积溢出。
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              class="h-7 text-xs text-destructive hover:text-destructive cursor-pointer"
              @click="handleClearCache"
            >
              <IconTrash class="h-3.5 w-3.5 mr-1" />
              清空图片缓存
            </Button>
          </div>

          <div
            v-if="settingsStore.imageCacheStats"
            class="flex items-center space-x-4 rounded-lg border border-border bg-muted/20 p-3"
          >
            <div>
              <span class="text-muted-foreground">已存文件：</span>
              <strong class="font-medium text-foreground">{{ settingsStore.imageCacheStats.fileCount }} 张</strong>
            </div>
            <div>
              <span class="text-muted-foreground">占用空间：</span>
              <strong class="font-medium text-foreground">
                {{ (settingsStore.imageCacheStats.totalBytes / (1024 * 1024)).toFixed(2) }} MB
              </strong>
            </div>
          </div>
        </section>

        <!-- 3. LLM Providers -->
        <section class="space-y-3 pt-4 border-t border-border">
          <div class="flex items-center justify-between">
            <div>
              <label class="font-medium text-foreground text-xs">多模态 AI 模型配置</label>
              <p class="text-[11px] text-muted-foreground mt-0.5">
                支持兼容 OpenAI 协议的本地 Ollama 或各类云端大模型。
              </p>
            </div>
          </div>

          <div class="grid grid-cols-4 gap-2">
            <button
              v-for="p in settingsStore.settings.providers"
              :key="p.id"
              type="button"
              @click="activeProviderId = p.id"
              :class="[
                'flex items-center justify-center p-2 rounded border text-xs cursor-pointer transition-colors',
                activeProviderId === p.id
                  ? 'border-primary bg-primary/10 text-primary font-medium'
                  : 'border-border bg-card text-muted-foreground hover:bg-muted'
              ]"
            >
              {{ p.name }}
            </button>
          </div>

          <!-- Provider Fields -->
          <div
            v-for="provider in settingsStore.settings.providers.filter((p) => p.id === activeProviderId)"
            :key="provider.id"
            class="space-y-3 rounded-lg border border-border bg-card p-3.5"
          >
            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1">
                <label class="text-[11px] text-muted-foreground">接口地址 (Base URL)</label>
                <Input v-model="provider.baseUrl" class="h-8 text-xs font-mono" />
              </div>
              <div class="space-y-1">
                <label class="text-[11px] text-muted-foreground">模型名称 (Model)</label>
                <Input v-model="provider.defaultModel" class="h-8 text-xs font-mono" />
              </div>
            </div>

            <div class="space-y-1">
              <label class="text-[11px] text-muted-foreground">API 密钥 (API Key)</label>
              <Input
                v-model="provider.apiKey"
                type="password"
                placeholder="sk-..."
                class="h-8 text-xs font-mono"
              />
            </div>
          </div>
        </section>
      </div>

      <!-- Footer -->
      <div class="flex h-12 shrink-0 items-center justify-between border-t border-border px-5 bg-muted/20">
        <span v-if="savedNotice" class="text-xs text-green-600 flex items-center">
          <IconCheck class="h-3.5 w-3.5 mr-1" /> 已保存配置
        </span>
        <span v-else class="text-xs text-muted-foreground">配置修改后即刻生效并持久化到 SQLite</span>

        <div class="flex items-center space-x-2">
          <Button size="sm" class="h-7 text-xs px-3 cursor-pointer" @click="saveCurrentSettings">
            保存配置
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

