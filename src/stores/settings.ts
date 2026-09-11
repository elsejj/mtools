import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { SystemSettings, ImageCacheStats, LLMProvider } from '@/types';
import { tauriApi } from '@/lib/tauri';

const DEFAULT_SETTINGS: SystemSettings = {
  theme: 'system',
  autoCopyResult: false,
  closeWindowOnCopy: false,
  defaultProviderId: 'openai',
  providers: [
    {
      id: 'openai',
      name: 'OpenAI',
      baseUrl: 'https://api.openai.com/v1',
      apiKey: '',
      defaultModel: 'gpt-4o',
    },
    {
      id: 'anthropic',
      name: 'Anthropic (Compatible)',
      baseUrl: 'https://api.anthropic.com/v1',
      apiKey: '',
      defaultModel: 'claude-3-5-sonnet-20241022',
    },
    {
      id: 'deepseek',
      name: 'DeepSeek',
      baseUrl: 'https://api.deepseek.com/v1',
      apiKey: '',
      defaultModel: 'deepseek-chat',
    },
    {
      id: 'ollama',
      name: 'Ollama (Local)',
      baseUrl: 'http://localhost:11434/v1',
      apiKey: 'ollama',
      defaultModel: 'llama3.2-vision',
    },
  ],
};

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<SystemSettings>(DEFAULT_SETTINGS);
  const isLoading = ref<boolean>(false);
  const imageCacheStats = ref<ImageCacheStats | null>(null);

  function applyTheme(theme: 'light' | 'dark' | 'system') {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark');
    } else if (theme === 'light') {
      root.classList.remove('dark');
    } else {
      const isSystemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      if (isSystemDark) {
        root.classList.add('dark');
      } else {
        root.classList.remove('dark');
      }
    }
  }

  async function loadSettings() {
    isLoading.value = true;
    try {
      const saved = await tauriApi.loadSystemSettings();
      if (saved) {
        settings.value = {
          ...DEFAULT_SETTINGS,
          ...saved,
          providers: saved.providers?.length ? saved.providers : DEFAULT_SETTINGS.providers,
        };
      }
      applyTheme(settings.value.theme);
    } catch (err) {
      console.warn('Failed to load settings, using defaults:', err);
      applyTheme(settings.value.theme);
    } finally {
      isLoading.value = false;
    }
  }

  async function updateSettings(partial: Partial<SystemSettings>) {
    settings.value = {
      ...settings.value,
      ...partial,
    };
    if (partial.theme) {
      applyTheme(partial.theme);
    }
    try {
      await tauriApi.saveSystemSettings(settings.value);
    } catch (err) {
      console.error('Failed to save settings:', err);
    }
  }

  async function addProvider(provider: LLMProvider) {
    settings.value.providers.push(provider);
    await updateSettings({ providers: settings.value.providers });
  }

  async function removeProvider(providerId: string) {
    settings.value.providers = settings.value.providers.filter((p) => p.id !== providerId);
    if (settings.value.defaultProviderId === providerId) {
      settings.value.defaultProviderId = settings.value.providers[0]?.id || '';
    }
    await updateSettings({
      providers: settings.value.providers,
      defaultProviderId: settings.value.defaultProviderId,
    });
  }

  async function testProviderConnection(
    provider: LLMProvider
  ): Promise<{ success: boolean; latencyMs?: number; message: string }> {
    let url = provider.baseUrl.trim();
    if (url.endsWith('/')) {
      url = url.slice(0, -1);
    }
    // Test endpoint: /models
    const testUrl = url.endsWith('/models') ? url : `${url}/models`;
    const startTime = Date.now();

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 6000);

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
      };
      if (provider.apiKey) {
        headers['Authorization'] = `Bearer ${provider.apiKey}`;
      }

      const res = await fetch(testUrl, {
        method: 'GET',
        headers,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);
      const latencyMs = Date.now() - startTime;

      if (res.ok) {
        return {
          success: true,
          latencyMs,
          message: `连接成功 (延迟: ${latencyMs}ms)`,
        };
      }

      // If /models returned 404 or 405, server exists but endpoint differed
      if (res.status === 404 || res.status === 405) {
        return {
          success: true,
          latencyMs,
          message: `服务可达 (${res.status})，但未开放 /models 索引，基础连接正常`,
        };
      }

      const errText = await res.text();
      return {
        success: false,
        latencyMs,
        message: `HTTP ${res.status}: ${errText.slice(0, 100)}`,
      };
    } catch (err: any) {
      const latencyMs = Date.now() - startTime;
      if (err.name === 'AbortError') {
        return {
          success: false,
          latencyMs,
          message: '连接超时 (超过 6 秒未响应)',
        };
      }
      return {
        success: false,
        latencyMs,
        message: `连接失败: ${err?.message || err}`,
      };
    }
  }

  async function fetchCacheStats() {
    try {
      const stats = await tauriApi.getImageCacheStats();
      imageCacheStats.value = stats;
      return stats;
    } catch (err) {
      console.error('Failed to fetch image cache stats:', err);
      return null;
    }
  }

  async function cleanupCache(days?: number, forceAll = false) {
    try {
      const freed = await tauriApi.cleanupImageCache(days, forceAll);
      await fetchCacheStats();
      return freed;
    } catch (err) {
      console.error('Failed to clean image cache:', err);
      return 0;
    }
  }

  return {
    // State
    settings,
    isLoading,
    imageCacheStats,

    // Actions
    loadSettings,
    updateSettings,
    applyTheme,
    addProvider,
    removeProvider,
    testProviderConnection,
    fetchCacheStats,
    cleanupCache,
  };
});
