import commands from '$lib/tauri/commands';
import { devErrorsStore } from '$lib/stores/dev-errors.svelte';
import { isTauri } from './guards';
import { untrack } from 'svelte';

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

function safeSnapshot(value: any): any {
  return untrack(() => {
    try {
      return JSON.parse(JSON.stringify(value));
    } catch {
      return String(value);
    }
  });
}

class Logger {
  private async log(level: LogLevel, tag: string, message: string, data?: any) {
    const browserLog = console[level === 'debug' ? 'log' : level];
    const safeData = data !== undefined ? safeSnapshot(data) : '';
    browserLog(`[${tag}] ${message}`, safeData);

    // Only send logs to Rust if we're in a Tauri environment
    if (!isTauri()) {
      return;
    }

    const result = data !== undefined
      ? await commands.logFromFrontendWithData({
        level,
        tag,
        message,
        data: JSON.parse(JSON.stringify(safeData))
      })
      : await commands.logFromFrontend({ level, tag, message });

    if (result.kind !== 'success') {
      const errorMsg = result.kind === 'command_error'
        ? result.error
        : result.error.message;
      console.error('Failed to send log to Rust:', errorMsg);
      devErrorsStore.addError(`[${tag}] Failed to send log to Rust: ${errorMsg}`, { level, message, data });
    }
  }

  debug(tag: string, message: string, data?: any) {
    this.log('debug', tag, message, data);
  }

  info(tag: string, message: string, data?: any) {
    this.log('info', tag, message, data);
  }

  warn(tag: string, message: string, data?: any) {
    this.log('warn', tag, message, data);
  }

  error(tag: string, message: string, data?: any) {
    this.log('error', tag, message, data);
  }
}

export const logger = new Logger();

