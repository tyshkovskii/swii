// Application Configuration and Constants
// Moved from src/lib/constants/index.ts and relevant types from src/lib/types.ts

// Window management constants
export const WINDOW_VISIBILITY_CHECK_INTERVAL = 100;
export const GLOBAL_SHORTCUT_KEY = "Command+Y";

// Application types
export type WindowInfo = {
    app_name: string;
    window_name: string | null;
    pid: number;
    window_number: number;
    project: string | null;
    active_editor_tab: string | null;
    app_icon: string | null;
}