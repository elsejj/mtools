<script setup lang="ts">
import { computed } from 'vue';
import { marked } from 'marked';
import Prism from 'prismjs';
import { tauriApi } from '@/lib/tauri';

const props = withDefaults(
  defineProps<{
    content: string;
  }>(),
  {
    content: '',
  }
);

// Configure marked with GFM and Prism code block highlighting
marked.setOptions({
  gfm: true,
  breaks: true,
});

marked.use({
  renderer: {
    code({ text, lang }: { text: string; lang?: string }) {
      const language = lang?.toLowerCase() || 'plain';
      const grammar = Prism.languages[language] || Prism.languages.plain;
      const html = grammar ? Prism.highlight(text, grammar, language) : text;
      return `<pre class="language-${language}"><code class="language-${language}">${html}</code></pre>`;
    },
  },
});

const renderedHtml = computed(() => {
  if (!props.content) return '';
  return marked.parse(props.content) as string;
});

function handleContentClick(event: MouseEvent) {
  const target = event.target as HTMLElement;
  const link = target.closest('a');
  if (link && link.href) {
    event.preventDefault();
    tauriApi.openUrl(link.href);
  }
}
</script>

<template>
  <div
    class="markdown-content h-full w-full overflow-y-auto overflow-x-auto select-text p-4 font-sans text-[13.5px] leading-[1.7] text-foreground"
    @click="handleContentClick"
    v-html="renderedHtml"
  />
</template>

<style scoped>
.markdown-content {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Headings */
:deep(h1) {
  font-size: 1.4rem;
  font-weight: 700;
  margin-top: 1.25rem;
  margin-bottom: 0.75rem;
  padding-bottom: 0.35rem;
  border-bottom: 1px solid var(--border);
  color: var(--foreground);
}

:deep(h2) {
  font-size: 1.2rem;
  font-weight: 600;
  margin-top: 1rem;
  margin-bottom: 0.5rem;
  padding-bottom: 0.25rem;
  border-bottom: 1px solid var(--border);
  color: var(--foreground);
}

:deep(h3) {
  font-size: 1.05rem;
  font-weight: 600;
  margin-top: 0.85rem;
  margin-bottom: 0.4rem;
  color: var(--foreground);
}

:deep(h4), :deep(h5), :deep(h6) {
  font-size: 0.95rem;
  font-weight: 600;
  margin-top: 0.75rem;
  margin-bottom: 0.35rem;
  color: var(--foreground);
}

/* Paragraphs */
:deep(p) {
  margin-bottom: 0.75rem;
  line-height: 1.65;
}

/* Table Styles - GFM Table */
:deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 1rem 0;
  font-size: 13px;
  line-height: 1.55;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}

:deep(thead) {
  background-color: var(--muted);
}

:deep(th) {
  border: 1px solid var(--border);
  padding: 8px 12px;
  font-weight: 600;
  text-align: left;
  color: var(--foreground);
  background-color: var(--muted);
}

:deep(td) {
  border: 1px solid var(--border);
  padding: 8px 12px;
  text-align: left;
  vertical-align: top;
  color: var(--foreground);
}

:deep(tr:nth-child(even) td) {
  background-color: color-mix(in srgb, var(--muted) 35%, transparent);
}

:deep(tr:hover td) {
  background-color: color-mix(in srgb, var(--accent) 50%, transparent);
}

/* Lists */
:deep(ul) {
  list-style-type: disc;
  padding-left: 1.5rem;
  margin-bottom: 0.75rem;
}

:deep(ol) {
  list-style-type: decimal;
  padding-left: 1.5rem;
  margin-bottom: 0.75rem;
}

:deep(li) {
  margin-bottom: 0.25rem;
}

/* Blockquotes */
:deep(blockquote) {
  border-left: 3px solid var(--primary);
  padding: 0.25rem 0.75rem;
  margin: 0.75rem 0;
  color: var(--muted-foreground);
  background: color-mix(in srgb, var(--muted) 25%, transparent);
  border-radius: 0 4px 4px 0;
}

/* Inline Code */
:deep(code:not(pre code)) {
  background-color: var(--muted);
  padding: 0.15rem 0.35rem;
  border-radius: 0.25rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.88em;
  color: var(--foreground);
}

/* Code Blocks */
:deep(pre) {
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  padding: 0.75rem;
  margin: 0.75rem 0;
  overflow-x: auto;
  background: color-mix(in srgb, var(--muted) 35%, transparent);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 12.5px;
  line-height: 1.55;
}

:deep(pre code) {
  font-family: inherit;
  font-size: inherit;
  background: transparent !important;
  padding: 0;
  border: none;
}

/* Links */
:deep(a) {
  color: var(--primary);
  text-decoration: underline;
  text-underline-offset: 2px;
  cursor: pointer;
}

:deep(a:hover) {
  opacity: 0.85;
}

/* Horizontal rule */
:deep(hr) {
  border: none;
  border-top: 1px solid var(--border);
  margin: 1.25rem 0;
}
</style>
