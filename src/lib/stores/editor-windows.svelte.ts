import commands from '$lib/tauri/commands';
import { handleCommandError, isCommandError } from '$lib/utils/errorHandler';
import type { WindowInfo } from '$lib/app/config';
import { logger } from '$lib/utils/logger';
import { devErrorsStore } from './dev-errors.svelte';

class EditorWindowsStore {
  windows = $state<WindowInfo[]>([]);
  isLoading = $state(false);

  async loadWindows() {
    this.isLoading = true;
    logger.info('EDITOR_STORE', 'Loading editor windows...');
    const result = await commands.listEditorWindows();

    logger.info('EDITOR_STORE', 'Command result:', result);

    if (isCommandError(result)) {
      logger.error('EDITOR_STORE', 'Command error:', result.error);
      devErrorsStore.addError('Failed to load editor windows', result.error);
      handleCommandError(result.error, "list editor windows");
      this.isLoading = false;
      return;
    }

    logger.info('EDITOR_STORE', `Received ${result.data.length} windows:`, result.data);
    this.windows = result.data;
    this.isLoading = false;
  }

  async bringWindowToFront(pid: number, window_number: number) {
    const result = await commands.bringWindowToFront({ pid, window_number });

    if (isCommandError(result)) {
      handleCommandError(result.error, "bring window to front");
      return false;
    }

    return true;
  }
}

export const editorWindowsStore = new EditorWindowsStore();

