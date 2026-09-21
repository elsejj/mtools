<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
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
  IconAdjustments,
  IconSparkles,
  IconDatabase,
  IconInfoCircle,
} from "@tabler/icons-vue";
import type { LLMProvider } from "@/types";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const settingsStore = useSettingsStore();
const activeTab = ref<string>("general");
const activeProviderId = ref<string>("openai");
const savedNotice = ref(false);

const isTestingConnection = ref(false);
const testResult = ref<{ success: boolean; message: string } | null>(null);

const isTestingEval = ref(false);
const evalTestResult = ref<{ success: boolean; message: string } | null>(null);

async function runTestEvaluationModel() {
  if (!settingsStore.settings.evaluationModel) return;
  isTestingEval.value = true;
  evalTestResult.value = null;
  try {
    const res = await settingsStore.testEvaluationModelConnection(
      settingsStore.settings.evaluationModel,
    );
    evalTestResult.value = res;
  } catch (err: any) {
    evalTestResult.value = {
      success: false,
      message: err?.message || String(err),
    };
  } finally {
    isTestingEval.value = false;
  }
}

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

      <!-- Tabbed Settings Body -->
      <Tabs
        v-model="activeTab"
        default-value="general"
        class="flex flex-1 flex-col overflow-hidden gap-0"
      >
        <!-- Tab Navigation Bar -->
        <div class="flex shrink-0 items-center border-b border-border px-5 py-2 bg-muted/10">
          <TabsList class="grid grid-cols-3 w-84 h-8 p-0.5">
            <TabsTrigger value="general" class="cursor-pointer gap-1.5 text-xs">
              <IconAdjustments class="h-3.5 w-3.5" />
              <span>常规偏好</span>
            </TabsTrigger>
            <TabsTrigger value="models" class="cursor-pointer gap-1.5 text-xs">
              <IconSparkles class="h-3.5 w-3.5" />
              <span>AI 模型</span>
            </TabsTrigger>
            <TabsTrigger value="storage" class="cursor-pointer gap-1.5 text-xs">
              <IconDatabase class="h-3.5 w-3.5" />
              <span>存储与缓存</span>
            </TabsTrigger>
          </TabsList>
        </div>

        <!-- Scrollable Tab Content Panels -->
        <div class="flex-1 overflow-y-auto p-5 text-xs">
          <!-- 1. General Tab -->
          <TabsContent value="general" class="space-y-6 mt-0 outline-none">
            <!-- Appearance Theme -->
            <section class="space-y-3">
              <div>
                <label class="font-medium text-foreground text-xs">外观主题</label>
                <p class="text-[11px] text-muted-foreground mt-0.5">
                  选择应用的界面色彩模式，支持跟随操作系统明暗模式自动切换。
                </p>
              </div>
              <div class="grid grid-cols-3 gap-3">
                <button
                  type="button"
                  @click="handleThemeChange('system')"
                  :class="[
                    'flex items-center justify-center space-x-2 rounded-lg border p-3 transition-all cursor-pointer',
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
                    'flex items-center justify-center space-x-2 rounded-lg border p-3 transition-all cursor-pointer',
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
                    'flex items-center justify-center space-x-2 rounded-lg border p-3 transition-all cursor-pointer',
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

            <!-- Automation Preferences -->
            <section class="space-y-3 pt-4 border-t border-border">
              <div>
                <label class="font-medium text-foreground text-xs">自动化偏好</label>
                <p class="text-[11px] text-muted-foreground mt-0.5">
                  配置工具执行完毕及处理结果后的自动化操作行为。
                </p>
              </div>
              <div class="space-y-2.5 rounded-lg border border-border bg-card p-4">
                <label class="flex items-center space-x-2.5 cursor-pointer">
                  <input
                    type="checkbox"
                    v-model="settingsStore.settings.autoCopyResult"
                    class="rounded border-input text-primary h-4 w-4"
                  />
                  <div class="space-y-0.5">
                    <div class="text-foreground font-medium">处理成功后自动复制结果到剪贴板</div>
                    <p class="text-[11px] text-muted-foreground">工具转换成功后无需手动点击复制</p>
                  </div>
                </label>
                <div class="border-t border-border/40 my-1"></div>
                <label class="flex items-center space-x-2.5 cursor-pointer">
                  <input
                    type="checkbox"
                    v-model="settingsStore.settings.closeWindowOnCopy"
                    class="rounded border-input text-primary h-4 w-4"
                  />
                  <div class="space-y-0.5">
                    <div class="text-foreground font-medium">
                      复制结果后自动最小化/隐藏工作台窗口
                    </div>
                    <p class="text-[11px] text-muted-foreground">
                      便于立即在其他应用程序中粘贴使用结果
                    </p>
                  </div>
                </label>
              </div>
            </section>
          </TabsContent>

          <!-- 2. AI Models Tab -->
          <TabsContent value="models" class="space-y-4 mt-0 outline-none">
            <!-- Intent Evaluation Model (TypeSafe AI) -->
            <div
              v-if="settingsStore.settings.evaluationModel"
              class="space-y-3 rounded-lg border border-border bg-card p-4"
            >
              <div class="flex items-center justify-between pb-2 border-b border-border/50">
                <div class="flex items-center space-x-2">
                  <span class="font-semibold text-foreground text-xs"
                    >意图判定模型 (TypeSafe AI)</span
                  >
                  <Badge
                    v-if="
                      settingsStore.settings.evaluationModel.baseUrl &&
                      settingsStore.settings.evaluationModel.apiKey
                    "
                    variant="secondary"
                    class="text-[9px] px-1.5 py-0 bg-green-500/15 text-green-600 font-normal"
                  >
                    已启用 (Jev)
                  </Badge>
                  <Badge
                    v-else
                    variant="outline"
                    class="text-[9px] px-1.5 py-0 text-muted-foreground font-normal"
                  >
                    未启用 (使用默认保底)
                  </Badge>
                </div>

                <Button
                  variant="outline"
                  size="sm"
                  class="h-6 px-2.5 text-xs cursor-pointer"
                  :disabled="isTestingEval"
                  @click="runTestEvaluationModel"
                >
                  <IconLoader2 v-if="isTestingEval" class="h-3 w-3 mr-1 animate-spin" />
                  <IconPlugConnected v-else class="h-3 w-3 mr-1 text-primary" />
                  <span>测试连接</span>
                </Button>
              </div>

              <p class="text-[11px] text-muted-foreground">
                当固定规则（JSON/URL/JWT/时间戳/计算器等）无法判定纯文本意图时，基于 TypeSafe Choice
                API (jev-latest) 智能分析意图并自动进入相应工具。
              </p>

              <!-- Test Connection Result Banner -->
              <div
                v-if="evalTestResult"
                :class="[
                  'flex items-center space-x-1.5 rounded p-2 text-xs font-medium',
                  evalTestResult.success
                    ? 'bg-green-500/10 text-green-600 border border-green-500/20'
                    : 'bg-destructive/10 text-destructive border border-destructive/20',
                ]"
              >
                <IconCheck v-if="evalTestResult.success" class="h-3.5 w-3.5 shrink-0" />
                <IconX v-else class="h-3.5 w-3.5 shrink-0" />
                <span class="truncate">{{ evalTestResult.message }}</span>
              </div>

              <div class="grid grid-cols-2 gap-3">
                <div class="space-y-1">
                  <label class="text-[11px] text-muted-foreground"
                    >判定服务接口 (Endpoint / Base URL)</label
                  >
                  <Input
                    v-model="settingsStore.settings.evaluationModel.baseUrl"
                    placeholder="https://api.typesafe.ai/v1/systemone"
                    class="h-8 text-xs font-mono"
                  />
                </div>
                <div class="space-y-1">
                  <label class="text-[11px] text-muted-foreground">判定模型 (Model)</label>
                  <Input
                    v-model="settingsStore.settings.evaluationModel.model"
                    placeholder="jev-latest"
                    class="h-8 text-xs font-mono"
                  />
                </div>
              </div>

              <div class="space-y-1">
                <label class="text-[11px] text-muted-foreground">API 密钥 (API Key)</label>
                <Input
                  v-model="settingsStore.settings.evaluationModel.apiKey"
                  type="password"
                  placeholder="ts-..."
                  class="h-8 text-xs font-mono"
                />
              </div>
            </div>

            <div class="flex items-center justify-between pt-2">
              <div>
                <label class="font-medium text-foreground text-xs">多模态 AI 执行模型配置池</label>
                <p class="text-[11px] text-muted-foreground mt-0.5">
                  支持兼容 OpenAI 协议的本地 Ollama、DeepSeek、Claude 兼容接口或自建代理。
                </p>
              </div>
              <Button
                size="sm"
                variant="outline"
                class="h-7 text-xs px-2.5 cursor-pointer"
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
                    ? 'border-primary bg-primary/10 text-primary font-medium shadow-2xs'
                    : 'border-border bg-card text-muted-foreground hover:bg-muted',
                ]"
              >
                <span>{{ p.name }}</span>
                <Badge
                  v-if="settingsStore.settings.defaultProviderId === p.id"
                  variant="secondary"
                  class="text-[9px] px-1 py-0 bg-primary/20 text-primary font-normal"
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
              class="space-y-3.5 rounded-lg border border-border bg-card p-4"
            >
              <div class="flex items-center justify-between pb-2 border-b border-border/50">
                <div class="flex items-center space-x-2">
                  <Input v-model="provider.name" class="h-7 text-xs font-semibold w-40" />
                  <Button
                    v-if="settingsStore.settings.defaultProviderId !== provider.id"
                    variant="ghost"
                    size="sm"
                    class="h-6 text-[11px] px-2 text-primary cursor-pointer hover:bg-primary/10"
                    @click="setDefaultProvider(provider.id)"
                  >
                    设为默认
                  </Button>
                </div>

                <div class="flex items-center space-x-1.5">
                  <Button
                    variant="outline"
                    size="sm"
                    class="h-6 px-2.5 text-xs cursor-pointer"
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
          </TabsContent>

          <!-- 3. Storage & Cache Tab -->
          <TabsContent value="storage" class="space-y-5 mt-0 outline-none">
            <div class="flex items-center justify-between">
              <div>
                <label class="font-medium text-foreground text-xs">图片缓存与存储治理</label>
                <p class="text-[11px] text-muted-foreground mt-0.5">
                  剪贴板临时图片按年月归档在本地缓存目录中，防止磁盘长期累积溢出。
                </p>
              </div>
              <Button
                variant="outline"
                size="sm"
                class="h-7 text-xs text-destructive hover:text-destructive hover:bg-destructive/10 cursor-pointer"
                @click="handleClearCache"
              >
                <IconTrash class="h-3.5 w-3.5 mr-1" />
                清空图片缓存
              </Button>
            </div>

            <!-- Cache Stats -->
            <div
              v-if="settingsStore.imageCacheStats"
              class="space-y-3 rounded-lg border border-border bg-card p-4"
            >
              <div class="grid grid-cols-2 gap-4">
                <div class="rounded-md bg-muted/40 p-3 space-y-1">
                  <span class="text-[11px] text-muted-foreground">已存图片文件</span>
                  <div class="text-base font-semibold text-foreground">
                    {{ settingsStore.imageCacheStats.fileCount }}
                    <span class="text-xs font-normal text-muted-foreground">张</span>
                  </div>
                </div>
                <div class="rounded-md bg-muted/40 p-3 space-y-1">
                  <span class="text-[11px] text-muted-foreground">磁盘占用空间</span>
                  <div class="text-base font-semibold text-foreground">
                    {{ (settingsStore.imageCacheStats.totalBytes / (1024 * 1024)).toFixed(2) }}
                    <span class="text-xs font-normal text-muted-foreground">MB</span>
                  </div>
                </div>
              </div>

              <div
                v-if="settingsStore.imageCacheStats.directoryPath"
                class="pt-2 border-t border-border/50 text-[11px]"
              >
                <span class="text-muted-foreground">缓存目录：</span>
                <code
                  class="font-mono text-foreground break-all bg-muted/60 px-1.5 py-0.5 rounded ml-1"
                >
                  {{ settingsStore.imageCacheStats.directoryPath }}
                </code>
              </div>
            </div>

            <!-- Explanatory note -->
            <div
              class="flex items-start space-x-2 rounded-lg border border-border/60 bg-muted/20 p-3 text-[11px] text-muted-foreground"
            >
              <IconInfoCircle class="h-4 w-4 shrink-0 text-muted-foreground mt-0.5" />
              <div class="space-y-1 leading-relaxed">
                <div class="font-medium text-foreground">存储说明</div>
                <p>
                  清空缓存仅会安全删除已缓存的历史临时剪贴板图像文件，不会影响您保存的自定义工具、执行历史记录或系统偏好设置。
                </p>
              </div>
            </div>
          </TabsContent>
        </div>
      </Tabs>

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
