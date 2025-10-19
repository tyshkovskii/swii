import type { FlatCommandResult } from "$lib/tauri/client";

export function handleCommandError(error: unknown, context: string): void {
  console.error(`Failed to ${context}:`, error);
}

export function isCommandError<T>(result: FlatCommandResult<T>): result is { kind: 'transport_error'; error: Error } | { kind: 'command_error'; error: string } {
  return result.kind === 'transport_error' || result.kind === 'command_error';
} 