import { onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { tauriApi } from "@/lib/tauri";

export function useWindowState() {
  let unlistenResize: UnlistenFn | null = null;
  let unlistenMove: UnlistenFn | null = null;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  async function persistGeometry() {
    try {
      const appWindow = getCurrentWindow();
      const isMaximized = await appWindow.isMaximized();
      const pos = await appWindow.outerPosition();
      const size = await appWindow.innerSize();

      if (size.width >= 400 && size.height >= 300) {
        await tauriApi.saveWindowGeometry({
          x: pos.x,
          y: pos.y,
          width: size.width,
          height: size.height,
          isMaximized,
        });
      }
    } catch (err) {
      console.warn("Failed to persist window geometry:", err);
    }
  }

  function debouncedPersist() {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(() => {
      persistGeometry();
    }, 300);
  }

  onMounted(async () => {
    try {
      // 1. 尝试从存储中恢复上次记忆的尺寸和位置
      await tauriApi.restoreWindowGeometry();

      // 2. 监听窗口调整与移动事件
      const appWindow = getCurrentWindow();
      unlistenResize = await appWindow.onResized(debouncedPersist);
      unlistenMove = await appWindow.onMoved(debouncedPersist);
    } catch (err) {
      console.warn("Failed to initialize window geometry listeners:", err);
    }

    window.addEventListener("beforeunload", persistGeometry);
  });

  onUnmounted(() => {
    window.removeEventListener("beforeunload", persistGeometry);
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    persistGeometry();
    if (unlistenResize) {
      unlistenResize();
      unlistenResize = null;
    }
    if (unlistenMove) {
      unlistenMove();
      unlistenMove = null;
    }
  });

  return {
    persistGeometry,
  };
}
