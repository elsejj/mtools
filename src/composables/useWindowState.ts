import { onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { tauriApi } from '@/lib/tauri';

export function useWindowState() {
  let unlistenResize: UnlistenFn | null = null;
  let unlistenMove: UnlistenFn | null = null;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  async function persistGeometry() {
    try {
      const appWindow = getCurrentWindow();
      const pos = await appWindow.outerPosition();
      const size = await appWindow.outerSize();
      const isMaximized = await appWindow.isMaximized();

      await tauriApi.saveWindowGeometry({
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
        isMaximized,
      });
    } catch (err) {
      // Running outside Tauri or window closed
    }
  }

  function debouncedPersist() {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(() => {
      persistGeometry();
    }, 500);
  }

  onMounted(async () => {
    try {
      const appWindow = getCurrentWindow();
      unlistenResize = await appWindow.listen('tauri://resize', debouncedPersist);
      unlistenMove = await appWindow.listen('tauri://move', debouncedPersist);
    } catch (err) {
      // In web dev mode without Tauri backend
    }
  });

  onUnmounted(() => {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
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

