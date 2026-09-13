import { marked } from "marked";
import Prism from "prismjs";
import { tauriApi } from "./tauri";

// 配置 marked 支持 GFM 表格与代码语法高亮
marked.setOptions({
  gfm: true,
  breaks: true,
});

marked.use({
  renderer: {
    code({ text, lang }: { text: string; lang?: string }) {
      const language = lang?.toLowerCase() || "plain";
      const grammar = Prism.languages[language] || Prism.languages.plain;
      const html = grammar ? Prism.highlight(text, grammar, language) : text;
      return `<pre style="background-color: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 12px; overflow-x: auto; font-family: monospace; font-size: 13px;" class="language-${language}"><code class="language-${language}">${html}</code></pre>`;
    },
  },
});

/**
 * 将 Markdown 格式文本转换为带有丰富内联样式的标准 HTML
 * 便于粘贴到 Word、Excel、微信、飞书、邮件等富文本软件时完美保留表格边框、背景及排版
 */
export function markdownToRichHtml(markdown: string): string {
  if (!markdown) return "";
  const rawHtml = marked.parse(markdown) as string;

  return rawHtml
    .replace(
      /<table/g,
      '<table style="border-collapse: collapse; width: 100%; border: 1px solid #cbd5e1; margin: 12px 0; font-size: 13.5px; line-height: 1.6; text-align: left;"',
    )
    .replace(/<thead/g, '<thead style="background-color: #f1f5f9;"')
    .replace(
      /<th/g,
      '<th style="border: 1px solid #cbd5e1; padding: 8px 12px; background-color: #f1f5f9; font-weight: 600; text-align: left;"',
    )
    .replace(
      /<td/g,
      '<td style="border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left; vertical-align: top;"',
    )
    .replace(
      /<pre(?![^>]*style=)/g,
      '<pre style="background-color: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 12px; overflow-x: auto; font-family: monospace; font-size: 13px;"',
    )
    .replace(
      /<code(?![^>]*style=)/g,
      '<code style="font-family: monospace; font-size: 90%; background-color: #f1f5f9; padding: 2px 4px; border-radius: 4px;"',
    );
}

/**
 * 复制内容至系统剪切板：
 * - asHtml 为 true 时：将内容转为富文本 HTML 写入（带文本回退），粘贴到支持富文本的应用直接呈现表格与排版
 * - asHtml 为 false 时：复制原始内容纯文本
 */
export async function copyContentToClipboard(text: string, asHtml: boolean): Promise<void> {
  if (!text) return;

  if (asHtml) {
    const html = markdownToRichHtml(text);

    // 1. 优先调用 Tauri 原生剪贴板 write_html（系统级支持，跨平台稳定）
    try {
      await tauriApi.writeClipboardHtml(html, text);
      return;
    } catch (err) {
      console.warn("tauriApi.writeClipboardHtml failed, trying Web Clipboard API:", err);
    }

    // 2. 尝试浏览器/Webview 原生 ClipboardItem API
    try {
      if (typeof ClipboardItem !== "undefined" && navigator.clipboard?.write) {
        const item = new ClipboardItem({
          "text/html": new Blob([html], { type: "text/html" }),
          "text/plain": new Blob([text], { type: "text/plain" }),
        });
        await navigator.clipboard.write([item]);
        return;
      }
    } catch (err) {
      console.warn("navigator.clipboard.write failed:", err);
    }

    // 3. 兜底方案：纯文本写入
    await navigator.clipboard.writeText(text);
  } else {
    // 纯文本/源码模式复制
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      await tauriApi.writeClipboardHtml(text, text);
    }
  }
}
