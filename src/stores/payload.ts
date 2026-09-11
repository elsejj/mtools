import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { EnrichedPayload, PayloadType, ToolScoreItem } from '@/types';
import { tauriApi } from '@/lib/tauri';

export const usePayloadStore = defineStore('payload', () => {
  const currentPayload = ref<EnrichedPayload | null>(null);
  const isLoading = ref<boolean>(false);
  const error = ref<string | null>(null);
  const showOriginalDecoded = ref<boolean>(false);
  const manualInput = ref<string>('');

  let unlistenPayloadReady: UnlistenFn | null = null;

  // Getters
  const hasPayload = computed(() => !!currentPayload.value);
  const payloadType = computed<PayloadType | null>(() => currentPayload.value?.payloadType ?? null);
  
  const decodingTrace = computed<string[]>(() => {
    if (!currentPayload.value?.metadata) return [];
    return (currentPayload.value.metadata as { decodingTrace?: string[] }).decodingTrace || [];
  });

  const hasDecodingTrace = computed(() => decodingTrace.value.length > 0);

  const preprocessedText = computed<string>(() => {
    if (!currentPayload.value) return '';
    return currentPayload.value.preprocessedResult?.formattedText || currentPayload.value.actualContent || '';
  });

  const recommendedToolId = computed<string>(() => currentPayload.value?.recommendedToolId || '');

  const candidateScores = computed<ToolScoreItem[]>(() => currentPayload.value?.candidateToolScores || []);

  // Actions
  function setPayload(payload: EnrichedPayload) {
    currentPayload.value = payload;
    showOriginalDecoded.value = false;
    error.value = null;
  }

  async function fetchFromClipboard() {
    isLoading.value = true;
    error.value = null;
    try {
      const payload = await tauriApi.fetchAndProcessClipboard();
      setPayload(payload);
      return payload;
    } catch (err: any) {
      error.value = err?.message || String(err);
      console.error('Failed to fetch from clipboard:', err);
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  async function processText(text: string) {
    if (!text.trim()) return null;
    isLoading.value = true;
    error.value = null;
    try {
      const payload = await tauriApi.processCustomContent(text);
      setPayload(payload);
      return payload;
    } catch (err: any) {
      error.value = err?.message || String(err);
      console.error('Failed to process custom content:', err);
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  async function startListening() {
    if (unlistenPayloadReady) {
      return;
    }
    try {
      unlistenPayloadReady = await listen<EnrichedPayload>('payload-ready', (event) => {
        console.log('Received payload-ready event:', event.payload);
        setPayload(event.payload);
      });
    } catch (err) {
      console.warn('Failed to listen to payload-ready event (possibly in browser mode):', err);
    }
  }

  function stopListening() {
    if (unlistenPayloadReady) {
      unlistenPayloadReady();
      unlistenPayloadReady = null;
    }
  }

  function toggleOriginalDecoded() {
    showOriginalDecoded.value = !showOriginalDecoded.value;
  }

  function reset() {
    currentPayload.value = null;
    isLoading.value = false;
    error.value = null;
    showOriginalDecoded.value = false;
    manualInput.value = '';
  }

  return {
    // State
    currentPayload,
    isLoading,
    error,
    showOriginalDecoded,
    manualInput,

    // Getters
    hasPayload,
    payloadType,
    decodingTrace,
    hasDecodingTrace,
    preprocessedText,
    recommendedToolId,
    candidateScores,

    // Actions
    setPayload,
    fetchFromClipboard,
    processText,
    startListening,
    stopListening,
    toggleOriginalDecoded,
    reset,
  };
});

