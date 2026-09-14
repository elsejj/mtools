import { onMounted } from "vue";
// import { tauriApi } from "@/lib/tauri";

/**
 * 窗口状态管理
 * 注：窗口尺寸（Resized）与位置（Moved）现已完全由 Rust 端的 on_window_event 原生监听与持久化，
 * 前端无需再监听 resize/move 事件或维护防抖定时器。此处仅在挂载时保留一次兜底的尺寸恢复请求。
 */
export function useWindowState() {
  onMounted(async () => {
    try {
      //await tauriApi.restoreWindowGeometry();
    } catch {
      // 纯网页开发模式无 Tauri 运行时环境，静默忽略
    }
  });
}
