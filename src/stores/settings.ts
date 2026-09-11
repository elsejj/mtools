import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { SystemSettings, ImageCacheStats } from '@/types';
import { tauriApi } from '@/lib/tauri';

const DEFAULT_SETTINGS: SystemSettings = {
  theme: 'system',
  autoCopyResult: false,
  closeWindowOnCopy: false,
  defaultProviderId: '',
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
    fetchCacheStats,
    cleanupCache,
  };
});

