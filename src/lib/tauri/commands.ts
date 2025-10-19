import type { InvokeArgs } from "@tauri-apps/api/core";
import type { Result } from "$lib/utils/tryCatch";
import { tryCatch } from "$lib/utils/tryCatch";
import { makeCommandFn, type CommandResult, type FlatCommandResult } from "./client";
import type { WindowInfo } from "$lib/app/config";

// Command utility types imported from client

function flattenCommandResult<T>(result: Result<CommandResult<T>>): FlatCommandResult<T> {
    if (result.error) return { kind: "transport_error", error: result.error };
    if (result.data.success) return { kind: "success", data: result.data.data };
    return { kind: "command_error", error: result.data.error };
}

function defineCommand<Args extends InvokeArgs | void, Ret>(command_name: string) {
    return async (args?: Args) => {
        const commandFn = makeCommandFn<Args | void, Ret>(command_name);
        const commandPromise = args ? commandFn(args) : commandFn();
        const result = await tryCatch(commandPromise);
        return flattenCommandResult<Ret>(result);
    }
}

// Command argument types
type LogLevel = 'debug' | 'info' | 'warn' | 'error';

type LogArgs = {
  level: LogLevel;
  tag: string;
  message: string;
};

type LogWithDataArgs = LogArgs & {
  data: any;
};

/**
 * Command registry for Tauri IPC communication.
 * 
 * This module exports a collection of typed commands that provide a type-safe interface
 * for communicating between the frontend and backend. Each command is defined using the
 * `defineCommand` helper which provides automatic error handling and consistent API patterns.
 * 
 * @template Args - The arguments type for the command (use `void` for no arguments)
 * @template ReturnType - The expected return type from the command execution
 * 
 * @remarks
 * - All commands automatically handle error flattening and provide consistent error responses
 * - Commands are type-safe and provide IntelliSense support
 * - The backend command names should match the string literals provided to `defineCommand`
 */
export default {
    bringWindowToFront: defineCommand<{ pid: number, window_number: number }, void>("bring_window_to_front"),
    listEditorWindows: defineCommand<void, WindowInfo[]>("list_editor_windows"),
    logFromFrontend: defineCommand<LogArgs, void>("log_from_frontend"),
    logFromFrontendWithData: defineCommand<LogWithDataArgs, void>("log_from_frontend_with_data"),
    openDevtools: defineCommand<void, void>("open_devtools"),
};