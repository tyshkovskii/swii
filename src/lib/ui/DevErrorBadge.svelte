<script lang="ts">
import { dev } from '$app/environment';
import { devErrorsStore } from '$lib/stores/dev-errors.svelte';
import { isTauri } from '$lib/utils/guards';
import commands from '$lib/tauri/commands';

async function openDevTools() {
  // Log all errors to console
  console.log('%c[DEV] Errors detected - see below ⬇️', 'color: red; font-size: 16px; font-weight: bold;');
  $state.snapshot(devErrorsStore.errors).forEach(error => {
    console.error('[DEV ERROR]', error.message, error.data);
  });
  
  // Only try to open DevTools if running in Tauri
  if (!isTauri()) {
    console.log('[DEV] Not in Tauri context - DevTools already available in browser');
    return;
  }
  
  // Try to open DevTools
  try {
    console.log('[DEV] Calling openDevtools command...');
    const result = await commands.openDevtools();
    console.log('[DEV] openDevtools result:', result);
    
    if (result.kind === 'success') {
      console.log('[DEV] DevTools opened successfully');
    } else {
      console.error('[DEV] Failed to open DevTools:', result);
    }
  } catch (error) {
    console.error('[DEV] Error calling openDevtools:', error);
  }
}
</script>

{#if dev && devErrorsStore.hasErrors}
  <button
    onclick={openDevTools}
    class="fixed bottom-4 right-4 z-50 relative bg-red-600 hover:bg-red-700 text-white rounded-full w-12 h-12 flex items-center justify-center shadow-lg transition-all hover:scale-110 cursor-pointer"
    title="Development errors detected - Click to open DevTools"
  >
    <span class="text-xl font-bold">!</span>
    <span class="absolute -top-1 -right-1 bg-red-800 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center">
      {devErrorsStore.errors.length}
    </span>
  </button>
{/if}
