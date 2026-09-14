<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  IconX,
  IconSun,
  IconMoon,
  IconDeviceDesktop,
  IconTrash,
  IconCheck,
  IconPlus,
  IconPlugConnected,
  IconLoader2,
} from "@tabler/icons-vue";
import type { LLMProvider } from "@/types";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const settingsStore = useSettingsStore();
const activeProviderId = ref<string>("openai");
const savedNotice = ref(false);

const isTestingConnection = ref(false);
const testResult = ref<{ success: boolean; message: string } | null>(null);

onMounted(async () => {
  await settingsStore.loadSettings();
  await settingsStore.fetchCacheStats();
  if (settingsStore.settings.defaultProviderId) {
    activeProviderId.value = settingsStore.settings.defaultProviderId;
  }
});

async function handleThemeChange(theme: "light" | "dark" | "system") {
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

async function runTestConnection(provider: LLMProvider) {
  isTestingConnection.value = true;
  testResult.value = null;
  try {
    const res = await settingsStore.testProviderConnection(provider);
    testResult.value = res;
  } catch (err: any) {
    testResult.value = {
      success: false,
      message: err?.message || String(err),
    };
  } finally {
    isTestingConnection.value = false;
  }
}

function handleAddCustomProvider() {
  const newId = `custom-provider-${Date.now()}`;
  const newProvider: LLMProvider = {
    id: newId,
    name: "新建服务商",
    baseUrl: "https://api.openai.com/v1",
    apiKey: "",
    defaultModel: "gpt-4o",
  };
  settingsStore.addProvider(newProvider);
  activeProviderId.value = newId;
}

function handleDeleteProvider(id: string) {
  settingsStore.removeProvider(id);
  if (activeProviderId.value === id) {
    activeProviderId.value = settingsStore.settings.providers[0]?.id || "";
  }
}

function setDefaultProvider(id: string) {
  settingsStore.updateSettings({ defaultProviderId: id });
}
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-2xs select-none p-4 animate-in fade-in duration-150"
  >
    <div
      class="flex h-[620px] w-full max-w-2xl flex-col rounded-xl border border-border bg-background shadow-2xl overflow-hidden"
    >
      <!-- Header -->
      <div
        class="flex h-12 shrink-0 items-center justify-between border-b border-border px-5 bg-muted/20"
      >
        <h2 class="text-sm font-semibold text-foreground">系统首选项与配置中心</h2>
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
      <div class="flex-1 overflow-y-auto p-5 space-y-5 text-xs">
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
                  : 'border-border bg-card text-muted-foreground hover:bg-muted',
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
                  : 'border-border bg-card text-muted-foreground hover:bg-muted',
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
                  : 'border-border bg-card text-muted-foreground hover:bg-muted',
              ]"
            >
              <IconMoon class="h-4 w-4" />
              <span>深色模式</span>
            </button>
          </div>
        </section>

        <!-- 2. Automation Preferences -->
        <section class="space-y-2 pt-3 border-t border-border">
          <label class="font-medium text-foreground text-xs">自动化偏好</label>
          <div class="space-y-2 rounded-lg border border-border bg-muted/10 p-3">
            <label class="flex items-center space-x-2 cursor-pointer">
              <input
                type="checkbox"
                v-model="settingsStore.settings.autoCopyResult"
                class="rounded border-input text-primary"
              />
              <span class="text-foreground font-medium">处理成功后自动复制结果到剪贴板</span>
            </label>
            <label class="flex items-center space-x-2 cursor-pointer">
              <input
                type="checkbox"
                v-model="settingsStore.settings.closeWindowOnCopy"
                class="rounded border-input text-primary"
              />
              <span class="text-foreground font-medium">复制结果后自动最小化/隐藏工作台窗口</span>
            </label>
          </div>
        </section>

        <!-- 3. LLM Providers -->
        <section class="space-y-3 pt-3 border-t border-border">
          <div class="flex items-center justify-between">
            <div>
              <label class="font-medium text-foreground text-xs">多模态 AI 模型配置池</label>
              <p class="text-[11px] text-muted-foreground mt-0.5">
                支持兼容 OpenAI 协议的本地 Ollama、DeepSeek、Claude 兼容接口或自建代理。
              </p>
            </div>
            <Button
              size="sm"
              variant="outline"
              class="h-7 text-xs px-2 cursor-pointer"
              @click="handleAddCustomProvider"
            >
              <IconPlus class="h-3.5 w-3.5 mr-1" />
              添加服务商
            </Button>
          </div>

          <!-- Provider Tabs -->
          <div class="flex items-center space-x-1.5 overflow-x-auto pb-1">
            <button
              v-for="p in settingsStore.settings.providers"
              :key="p.id"
              type="button"
              @click="
                () => {
                  activeProviderId = p.id;
                  testResult = null;
                }
              "
              :class="[
                'flex items-center space-x-1.5 px-3 py-1 rounded border text-xs cursor-pointer transition-colors whitespace-nowrap',
                activeProviderId === p.id
                  ? 'border-primary bg-primary/10 text-primary font-medium'
                  : 'border-border bg-card text-muted-foreground hover:bg-muted',
              ]"
            >
              <span>{{ p.name }}</span>
              <Badge
                v-if="settingsStore.settings.defaultProviderId === p.id"
                variant="secondary"
                class="text-[9px] px-1 py-0 bg-primary/20 text-primary"
              >
                默认
              </Badge>
            </button>
          </div>

          <!-- Active Provider Settings Form -->
          <div
            v-for="provider in settingsStore.settings.providers.filter(
              (p) => p.id === activeProviderId,
            )"
            :key="provider.id"
            class="space-y-3 rounded-lg border border-border bg-card p-3.5"
          >
            <div class="flex items-center justify-between pb-1 border-b border-border/50">
              <div class="flex items-center space-x-2">
                <Input v-model="provider.name" class="h-7 text-xs font-semibold w-40" />
                <Button
                  v-if="settingsStore.settings.defaultProviderId !== provider.id"
                  variant="ghost"
                  size="sm"
                  class="h-6 text-[11px] px-2 text-primary"
                  @click="setDefaultProvider(provider.id)"
                >
                  设为默认
                </Button>
              </div>

              <div class="flex items-center space-x-1">
                <Button
                  variant="outline"
                  size="sm"
                  class="h-6 px-2 text-xs cursor-pointer"
                  :disabled="isTestingConnection"
                  @click="runTestConnection(provider)"
                >
                  <IconLoader2 v-if="isTestingConnection" class="h-3 w-3 mr-1 animate-spin" />
                  <IconPlugConnected v-else class="h-3 w-3 mr-1 text-primary" />
                  <span>测试连接</span>
                </Button>

                <Button
                  v-if="settingsStore.settings.providers.length > 1"
                  variant="ghost"
                  size="sm"
                  class="h-6 w-6 p-0 text-muted-foreground hover:text-destructive cursor-pointer"
                  title="删除服务商"
                  @click="handleDeleteProvider(provider.id)"
                >
                  <IconTrash class="h-3.5 w-3.5" />
                </Button>
              </div>
            </div>

            <!-- Test Connection Result Banner -->
            <div
              v-if="testResult"
              :class="[
                'flex items-center space-x-1.5 rounded p-2 text-xs font-medium',
                testResult.success
                  ? 'bg-green-500/10 text-green-600 border border-green-500/20'
                  : 'bg-destructive/10 text-destructive border border-destructive/20',
              ]"
            >
              <IconCheck v-if="testResult.success" class="h-3.5 w-3.5 shrink-0" />
              <IconX v-else class="h-3.5 w-3.5 shrink-0" />
              <span class="truncate">{{ testResult.message }}</span>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1">
                <label class="text-[11px] text-muted-foreground">接口地址 (Base URL)</label>
                <Input v-model="provider.baseUrl" class="h-8 text-xs font-mono" />
              </div>
              <div class="space-y-1">
                <label class="text-[11px] text-muted-foreground"
                  >默认模型名称 (Default Model)</label
                >
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

        <!-- 4. Storage & Image Cache -->
        <section class="space-y-3 pt-3 border-t border-border">
          <div class="flex items-center justify-between">
            <div>
              <label class="font-medium text-foreground text-xs">图片缓存与存储治理</label>
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
            class="flex items-center space-x-6 rounded-lg border border-border bg-muted/20 p-3"
          >
            <div>
              <span class="text-muted-foreground">已存文件：</span>
              <strong class="font-medium text-foreground"
                >{{ settingsStore.imageCacheStats.fileCount }} 张</strong
              >
            </div>
            <div>
              <span class="text-muted-foreground">磁盘占用：</span>
              <strong class="font-medium text-foreground">
                {{ (settingsStore.imageCacheStats.totalBytes / (1024 * 1024)).toFixed(2) }} MB
              </strong>
            </div>
          </div>
        </section>
      </div>

      <!-- Footer -->
      <div
        class="flex h-12 shrink-0 items-center justify-between border-t border-border px-5 bg-muted/20"
      >
        <span v-if="savedNotice" class="text-xs text-green-600 flex items-center">
          <IconCheck class="h-3.5 w-3.5 mr-1" /> 已成功保存首选项并生效
        </span>
        <span v-else class="text-xs text-muted-foreground"
          >配置修改后即刻生效并持久化到 SQLite 数据库</span
        >

        <div class="flex items-center space-x-2">
          <Button size="sm" class="h-7 text-xs px-3 cursor-pointer" @click="saveCurrentSettings">
            保存配置
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
