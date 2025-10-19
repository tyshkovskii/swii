import { invoke } from "@tauri-apps/api/core";
import type { InvokeArgs } from "@tauri-apps/api/core";
import { isTauri } from "$lib/utils/guards";

// Re-export InvokeArgs for use by other files in the tauri layer
export type { InvokeArgs } from "@tauri-apps/api/core";

// Command types moved from src/lib/commands/command.ts
export type CommandSuccess<T> = {
    readonly success: true;
    readonly data: T;
    readonly execution_time_ms: number;
    readonly timestamp: string;
    readonly error: null;
};

export type CommandFailure = {
    readonly success: false;
    readonly error: string;
    readonly execution_time_ms: number;
    readonly timestamp: string;
    readonly data: null;
};

export type CommandResult<T> = CommandSuccess<T> | CommandFailure;

export type CommandResponse<T> = 
  | { kind: 'success'; data: T }
  | { kind: 'transport_error'; error: unknown }
  | { kind: 'command_error'; error: unknown };

export type CommandFn<Args extends InvokeArgs | void, Ret> =
    [Args] extends [void]
    ? () => Promise<CommandResult<Ret>>
    : (args: Args) => Promise<CommandResult<Ret>>;

// Flattened command result type for easier error handling
export type FlatCommandResult<T> =
    | { kind: "success"; data: T }
    | { kind: "transport_error"; error: Error }
    | { kind: "command_error"; error: string };



// Main Tauri client interface
export const makeCommandFn = <Args extends InvokeArgs | void, Ret>(command_name: string): CommandFn<Args, Ret> => {
    const fn = async (args?: Args): Promise<CommandResult<Ret>> => {
        if (!isTauri()) {
            throw new Error('Tauri commands can only be executed in Tauri environment');
        }
        
        const invokePromise: Promise<CommandResult<Ret>> = args 
            ? invoke<CommandResult<Ret>>(command_name, { args }) 
            : invoke<CommandResult<Ret>>(command_name);
        return invokePromise;
    };
    return fn;
};