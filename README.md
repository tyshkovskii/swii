# Swii

A fast overlay app for macOS that lets you switch between code editors with a keyboard shortcut. Think of it as "Command+Tab" but better and optimized for developers.

## Features

- **Quick switching** between open code editors (Cursor, VSCode, etc.)
- **Fuzzy search** to instantly find the editor you need
- **Keyboard-driven** workflow with hotkey activation
- **Native macOS integration** with overlay window

## Tech Stack

- **Frontend:** Svelte 5 + SvelteKit + TypeScript
- **Backend:** Rust + Tauri 2
- **UI:** Tailwind CSS
- **Search:** Fuse.js

## Development

```bash
# Install dependencies
bun install

# Run in development mode
bun run tauri dev

# Build for production
bun run build:mac
```