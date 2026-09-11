<script setup lang="ts">
import { computed } from 'vue';
import Prism from 'prismjs';

const props = withDefaults(
  defineProps<{
    code: string;
    language?: string;
  }>(),
  {
    language: 'json',
  }
);

const isProse = computed(() => {
  const lang = props.language?.toLowerCase();
  return lang === 'markdown' || lang === 'text';
});

const highlightedHtml = computed(() => {
  if (!props.code) return '';
  const lang = props.language.toLowerCase();
  const grammar = Prism.languages[lang] || Prism.languages.json || Prism.languages.plain;
  if (!grammar) {
    // Escape HTML as fallback
    return props.code
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  }
  return Prism.highlight(props.code, grammar, lang);
});
</script>

<template>
  <div
    class="h-full w-full overflow-y-auto overflow-x-hidden select-text p-4"
    :class="isProse ? 'font-sans text-[13.5px] leading-[1.7]' : 'font-mono text-[13px] leading-[1.6]'"
  >
    <pre class="whitespace-pre-wrap break-words overflow-x-hidden m-0 bg-transparent p-0"><code
      :class="[`language-${language}`, 'whitespace-pre-wrap break-words']"
      v-html="highlightedHtml"
    ></code></pre>
  </div>
</template>

<style scoped>
:deep(pre), :deep(code) {
  white-space: pre-wrap !important;
  word-break: break-word !important;
  overflow-wrap: anywhere !important;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

code:not(.language-markdown):not(.language-text) {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}
</style>

