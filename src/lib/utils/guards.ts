// Tauri environment detection guard
// In Tauri v2, check for __TAURI_INTERNALS__ instead of __TAURI__
export const isTauri = (): boolean => {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

// Type guards for common operations
export const isString = (value: unknown): value is string => {
    return typeof value === 'string';
};

export const isNumber = (value: unknown): value is number => {
    return typeof value === 'number' && !isNaN(value);
};

export const isObject = (value: unknown): value is Record<string, unknown> => {
    return typeof value === 'object' && value !== null && !Array.isArray(value);
};

export const isArray = (value: unknown): value is unknown[] => {
    return Array.isArray(value);
};

export const isDefined = <T>(value: T | undefined | null): value is T => {
    return value !== undefined && value !== null;
};