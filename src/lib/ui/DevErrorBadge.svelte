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
    class="fixed bottom-3 left-3 z-50 bg-red-500 hover:bg-red-600 text-white text-xs font-medium px-2 py-1 rounded-md shadow-sm transition-colors cursor-pointer"
    title="Development errors detected - Click to open DevTools"
  >
    {devErrorsStore.errors.length} {devErrorsStore.errors.length === 1 ? 'error' : 'errors'}
  </button>
{/if}
