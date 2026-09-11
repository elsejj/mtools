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
  <div class="h-full w-full overflow-auto font-mono text-xs leading-relaxed select-text p-3">
    <pre class="whitespace-pre-wrap break-all m-0 bg-transparent p-0"><code :class="`language-${language}`" v-html="highlightedHtml"></code></pre>
  </div>
</template>

<style scoped>
pre, code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}
</style>

