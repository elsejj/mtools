import { onMounted, onUnmounted } from 'vue';

export interface ShortcutHandlers {
  onSearch?: () => void;
  onHistory?: () => void;
  onSettings?: () => void;
  onExecute?: () => void;
  onSelectCandidate?: (index: number) => void;
  onEscape?: () => void;
}

export function useShortcuts(handlers: ShortcutHandlers) {
  function handleKeyDown(event: KeyboardEvent) {
    const isCtrlOrCmd = event.ctrlKey || event.metaKey;

    // Ctrl+K: Search / Quick switch
    if (isCtrlOrCmd && event.key.toLowerCase() === 'k') {
      event.preventDefault();
      handlers.onSearch?.();
      return;
    }

    // Ctrl+H: History
    if (isCtrlOrCmd && event.key.toLowerCase() === 'h') {
      event.preventDefault();
      handlers.onHistory?.();
      return;
    }

    // Ctrl+,: Settings
    if (isCtrlOrCmd && event.key === ',') {
      event.preventDefault();
      handlers.onSettings?.();
      return;
    }

    // Ctrl+Enter: Execute / Action
    if (isCtrlOrCmd && event.key === 'Enter') {
      event.preventDefault();
      handlers.onExecute?.();
      return;
    }

    // Escape: Close modals / Clear
    if (event.key === 'Escape') {
      handlers.onEscape?.();
      return;
    }

    // Alt+1 to Alt+9: Quick select candidate tool
    if (event.altKey && event.code.startsWith('Digit')) {
      const digit = parseInt(event.code.replace('Digit', ''), 10);
      if (digit >= 1 && digit <= 9) {
        event.preventDefault();
        handlers.onSelectCandidate?.(digit - 1);
        return;
      }
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });
}

