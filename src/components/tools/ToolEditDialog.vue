<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useToolStore, calculateToolMatchScore } from "@/stores/tools";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { IconX, IconSparkles, IconAdjustments } from "@tabler/icons-vue";
import type { ToolDefinition, ToolType, PostActionType } from "@/types";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const toolStore = useToolStore();

const toolForm = ref<ToolDefinition>({
  id: `custom-tool-${Date.now()}`,
  name: "新自定义工具",
  icon: "IconCode",
  description: "执行自定义处理与后置逻辑",
  category: "developer",
  isCustom: true,
  enabled: true,
  sortOrder: 99,
  matcher: {
    acceptedTypes: ["text"],
    patterns: [],
    requiredFormats: [],
    basePriority: 50,
  },
  type: "code",
  postAction: {
    type: "none",
    saveConfig: {
      directory: "custom_output",
      extension: "txt",
    },
  },
  cliConfig: {
    command: "cat",
    args: [],
    stdinMode: "pipe",
    timeoutMs: 5000,
  },
  llmConfig: {
    useSystemProvider: true,
    systemPrompt: "请处理输入的文本并返回清晰的结果。",
    userPromptTemplate: "{{input}}",
    stream: true,
    temperature: 0.7,
  },
});

onMounted(() => {
  if (toolStore.editingTool) {
    toolForm.value = JSON.parse(JSON.stringify(toolStore.editingTool));
  }
});

// Pattern string for editing
const patternInput = ref(toolForm.value.matcher.patterns?.join(", ") || "");
watch(patternInput, (val) => {
  toolForm.value.matcher.patterns = val
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
});

// Real-time Matcher Simulator
const testSampleText = ref('{"status": "ok", "code": 200}');
const testSampleType = ref<"text" | "image">("text");

const matchScore = computed(() => {
  return calculateToolMatchScore(toolForm.value, testSampleText.value, testSampleType.value);
});

const scoreColorClass = computed(() => {
  if (matchScore.value >= 80) return "text-emerald-500 bg-emerald-500/10 border-emerald-500/30";
  if (matchScore.value >= 50) return "text-sky-500 bg-sky-500/10 border-sky-500/30";
  if (matchScore.value > 0) return "text-amber-500 bg-amber-500/10 border-amber-500/30";
  return "text-muted-foreground bg-muted border-border";
});

async function handleSave() {
  await toolStore.saveTool(toolForm.value);
  toolStore.setActiveTool(toolForm.value.id);
  toolStore.setEditingTool(null);
  emit("close");
}

function handleClose() {
  toolStore.setEditingTool(null);
  emit("close");
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
        <div class="flex items-center space-x-2">
          <IconAdjustments class="h-4 w-4 text-primary" />
          <h2 class="text-sm font-semibold text-foreground">
            {{
              toolStore.editingTool ? "编辑工具：" + toolStore.editingTool.name : "新建自定义工具"
            }}
          </h2>
        </div>
        <Button
          variant="ghost"
          size="sm"
          class="h-7 w-7 p-0 cursor-pointer text-muted-foreground hover:text-foreground"
          @click="handleClose"
        >
          <IconX class="h-4 w-4" />
        </Button>
      </div>

      <!-- Form Body -->
      <div class="flex-1 overflow-y-auto p-5 space-y-4 text-xs">
        <!-- Basic Info -->
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

        <!-- Matcher Rules & Real-time Score Simulator -->
        <div class="space-y-3 rounded-lg border border-border bg-muted/10 p-3.5">
          <div class="flex items-center justify-between">
            <label class="font-medium text-foreground flex items-center space-x-1.5">
              <IconSparkles class="h-3.5 w-3.5 text-primary" />
              <span>匹配规则与实时得分模拟器</span>
            </label>
            <Badge
              variant="outline"
              :class="['text-xs font-mono font-bold px-2 py-0.5', scoreColorClass]"
            >
              匹配得分: {{ matchScore }} 分
            </Badge>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div class="space-y-1">
              <label class="text-[11px] text-muted-foreground">接受载荷类型</label>
              <div class="flex space-x-3 pt-1">
                <label class="flex items-center space-x-1.5 cursor-pointer">
                  <input
                    type="checkbox"
                    :checked="toolForm.matcher.acceptedTypes.includes('text')"
                    @change="
                      (e: any) => {
                        if (e.target.checked) {
                          if (!toolForm.matcher.acceptedTypes.includes('text'))
                            toolForm.matcher.acceptedTypes.push('text');
                        } else {
                          toolForm.matcher.acceptedTypes = toolForm.matcher.acceptedTypes.filter(
                            (t) => t !== 'text',
                          );
                        }
                      }
                    "
                    class="rounded border-input text-primary"
                  />
                  <span>文本 (text)</span>
                </label>
                <label class="flex items-center space-x-1.5 cursor-pointer">
                  <input
                    type="checkbox"
                    :checked="toolForm.matcher.acceptedTypes.includes('image')"
                    @change="
                      (e: any) => {
                        if (e.target.checked) {
                          if (!toolForm.matcher.acceptedTypes.includes('image'))
                            toolForm.matcher.acceptedTypes.push('image');
                        } else {
                          toolForm.matcher.acceptedTypes = toolForm.matcher.acceptedTypes.filter(
                            (t) => t !== 'image',
                          );
                        }
                      }
                    "
                    class="rounded border-input text-primary"
                  />
                  <span>图片 (image)</span>
                </label>
              </div>
            </div>

            <div class="space-y-1">
              <label class="text-[11px] text-muted-foreground">基础优先级得分 (0~100)</label>
              <Input
                type="number"
                v-model.number="toolForm.matcher.basePriority"
                min="0"
                max="100"
                class="h-8 text-xs font-mono"
              />
            </div>
          </div>

          <div class="space-y-1">
            <label class="text-[11px] text-muted-foreground">正则匹配模式 (多个用逗号隔开)</label>
            <Input
              v-model="patternInput"
              placeholder="例如: ^https?://, ^ey[A-Za-z0-9-_]+"
              class="h-8 text-xs font-mono"
            />
          </div>

          <!-- Simulator Input Box -->
          <div class="space-y-1 pt-1 border-t border-border/50">
            <div class="flex items-center justify-between text-[11px] text-muted-foreground">
              <span>输入测试样例验证匹配效果：</span>
              <div class="flex space-x-2">
                <button
                  type="button"
                  @click="testSampleType = 'text'"
                  :class="
                    testSampleType === 'text'
                      ? 'text-primary font-medium underline'
                      : 'hover:opacity-80'
                  "
                >
                  文本样例
                </button>
                <button
                  type="button"
                  @click="testSampleType = 'image'"
                  :class="
                    testSampleType === 'image'
                      ? 'text-primary font-medium underline'
                      : 'hover:opacity-80'
                  "
                >
                  图片样例
                </button>
              </div>
            </div>
            <textarea
              v-model="testSampleText"
              rows="2"
              placeholder="在此输入样例字符串，观察上方匹配得分变化..."
              class="w-full rounded border border-border bg-background p-2 text-xs font-mono leading-relaxed"
            />
          </div>
        </div>

        <!-- Tool Type -->
        <div class="space-y-1.5 pt-2 border-t border-border">
          <label class="font-medium text-foreground">引擎执行类型</label>
          <div class="grid grid-cols-3 gap-2">
            <button
              v-for="t in ['code', 'llm', 'cli'] as ToolType[]"
              :key="t"
              type="button"
              @click="toolForm.type = t"
              :class="[
                'p-2 rounded border text-xs capitalize cursor-pointer transition-colors',
                toolForm.type === t
                  ? 'border-primary bg-primary/10 text-primary font-medium'
                  : 'border-border bg-card text-muted-foreground hover:bg-muted',
              ]"
            >
              {{ t }} 引擎
            </button>
          </div>
        </div>

        <!-- CLI Config -->
        <div
          v-if="toolForm.type === 'cli' && toolForm.cliConfig"
          class="space-y-3 rounded-lg border border-border bg-muted/10 p-3"
        >
          <div class="space-y-1">
            <label class="text-[11px] text-muted-foreground"
              >命令可执行文件 (如 jq, python3, prettier)</label
            >
            <Input v-model="toolForm.cliConfig.command" class="h-8 text-xs font-mono" />
          </div>
          <div class="space-y-1">
            <label class="text-[11px] text-muted-foreground">输入传递模式</label>
            <select
              v-model="toolForm.cliConfig.stdinMode"
              class="flex h-8 w-full rounded-md border border-input bg-background px-2 text-xs font-mono"
            >
              <option value="pipe">管道注入 (stdin)</option>
              <option value="none">无 (仅运行命令)</option>
            </select>
          </div>
        </div>

        <!-- LLM Config -->
        <div
          v-else-if="toolForm.type === 'llm' && toolForm.llmConfig"
          class="space-y-3 rounded-lg border border-border bg-muted/10 p-3"
        >
          <div class="space-y-1">
            <label class="text-[11px] text-muted-foreground"
              >System Prompt (系统角色与提示词)</label
            >
            <textarea
              v-model="toolForm.llmConfig.systemPrompt"
              rows="2"
              class="w-full rounded border border-border bg-background p-2 text-xs font-mono"
            />
          </div>
          <div class="space-y-1">
            <label class="text-[11px] text-muted-foreground" v-pre
              >用户提示词模板 (使用 {{ input }} 占位)</label
            >
            <Input v-model="toolForm.llmConfig.userPromptTemplate" class="h-8 text-xs font-mono" />
          </div>
        </div>

        <!-- Post Action -->
        <div class="space-y-2 pt-2 border-t border-border">
          <label class="font-medium text-foreground">后置执行动作 (Post-Action)</label>
          <div class="grid grid-cols-3 gap-2">
            <button
              v-for="pa in ['none', 'copy_to_clipboard', 'save_to_file'] as PostActionType[]"
              :key="pa"
              type="button"
              @click="toolForm.postAction.type = pa"
              :class="[
                'p-2 rounded border text-xs cursor-pointer transition-colors',
                toolForm.postAction.type === pa
                  ? 'border-primary bg-primary/10 text-primary font-medium'
                  : 'border-border bg-card text-muted-foreground hover:bg-muted',
              ]"
            >
              {{ pa === "none" ? "无动作" : pa === "copy_to_clipboard" ? "自动复制" : "自动存盘" }}
            </button>
          </div>

          <div
            v-if="toolForm.postAction.type === 'save_to_file' && toolForm.postAction.saveConfig"
            class="space-y-2 rounded-lg border border-border bg-muted/10 p-3 mt-2"
          >
            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1">
                <label class="text-[11px] text-muted-foreground">专属子目录名</label>
                <Input
                  v-model="toolForm.postAction.saveConfig.directory"
                  class="h-8 text-xs font-mono"
                />
              </div>
              <div class="space-y-1">
                <label class="text-[11px] text-muted-foreground">文件扩展名</label>
                <Input
                  v-model="toolForm.postAction.saveConfig.extension"
                  placeholder="txt"
                  class="h-8 text-xs font-mono"
                />
              </div>
            </div>
            <p class="text-[10px] text-muted-foreground">
              文件将自动以时间戳命名保存，例如：<code
                >$APP_DATA_DIR/saved/{{
                  toolForm.postAction.saveConfig.directory
                }}/YYYY-MM-DD_HH-mm-ss.{{ toolForm.postAction.saveConfig.extension || "txt" }}</code
              >
            </p>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div
        class="flex h-12 shrink-0 items-center justify-end space-x-2 border-t border-border px-5 bg-muted/20"
      >
        <Button variant="ghost" size="sm" class="h-7 text-xs cursor-pointer" @click="handleClose">
          取消
        </Button>
        <Button size="sm" class="h-7 text-xs cursor-pointer" @click="handleSave">
          保存工具配置
        </Button>
      </div>
    </div>
  </div>
</template>
