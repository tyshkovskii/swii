<script lang="ts">
import { onMount } from 'svelte';
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
import { overlayStore } from '$lib/stores/overlay.svelte';
import { editorWindowsStore } from '$lib/stores/editor-windows.svelte';
import { searchStore } from '$lib/stores/search.svelte';
import { GLOBAL_SHORTCUT_KEY } from '$lib/app/config';
import { logger } from '$lib/utils/logger';
import EditorWindowsList from './components/EditorWindowsList.svelte';
import EditorSearchBar from './components/EditorSearchBar.svelte';
import DevErrorBadge from '$lib/ui/DevErrorBadge.svelte';
import FooterBar from '$lib/ui/FooterBar.svelte';
import FooterLabel from '$lib/ui/FooterLabel.svelte';

logger.info('PAGE', '🚀 +page.svelte script block started');

let searchBarRef: EditorSearchBar;

let filteredResults = $derived.by(() => {
  try {
    const windows = editorWindowsStore.windows;
    const result = searchStore.search(windows);
    return result;
  } catch (error) {
    logger.error('PAGE', 'Error in filteredResults $derived', error);
    return [];
  }
});

let overlayStatus = $derived.by(() =>
  overlayStore.isVisible ? 'Overlay visible' : 'Overlay hidden'
);

async function handleBringWindowToFront(pid: number, window_number: number) {
  await editorWindowsStore.bringWindowToFront(pid, window_number);
  await overlayStore.toggle();
}

function clearSearch() {
  searchStore.clear();
}

onMount(() => {
  logger.info('PAGE', '✅ Component mounted, setting up overlay callbacks');
  
  try {
    overlayStore.setOnHide(() => {
      logger.debug('PAGE', 'onHide callback triggered');
      clearSearch();
    });

    overlayStore.setOnShow(() => {
      logger.debug('PAGE', 'onShow callback triggered');
      clearSearch();
      setTimeout(() => {
        searchBarRef?.focus();
      }, 100);
    });

    let down = new Set<number>();

    const wrappedHandler = async (e: any) => {
      logger.info('PAGE', 'Global shortcut handler triggered', e);
      if (e.state === 'Pressed') {
        if (down.has(e.id)) {
          logger.debug('PAGE', `Ignoring repeat press for id: ${e.id}`);
          return;
        }
        down.add(e.id);
        logger.info('PAGE', 'Calling overlayStore.toggle()');
        await overlayStore.toggle();
      } else {
        logger.debug('PAGE', `Key released for id: ${e.id}`);
        down.delete(e.id);
      }
    };

    logger.info('PAGE', `Attempting to register global shortcut: ${GLOBAL_SHORTCUT_KEY}`);
    register(GLOBAL_SHORTCUT_KEY, wrappedHandler)
      .then(() => logger.info('PAGE', `✅ Successfully registered ${GLOBAL_SHORTCUT_KEY}`))
      .catch((e) => {
        logger.error('PAGE', `❌ Failed to register ${GLOBAL_SHORTCUT_KEY}`, e);
      });

    return () => {
      logger.info('PAGE', 'Component unmounting, cleaning up');
      unregister(GLOBAL_SHORTCUT_KEY)
        .then(() => logger.info('PAGE', `Unregistered ${GLOBAL_SHORTCUT_KEY}`))
        .catch((e) => logger.error('PAGE', `Failed to unregister ${GLOBAL_SHORTCUT_KEY}`, e));
      overlayStore.destroy();
    };
  } catch (error) {
    logger.error('PAGE', 'Error in onMount', error);
  }
});

$effect(() => {
  try {
    logger.debug('PAGE', `Overlay visibility effect triggered. isVisible: ${overlayStore.isVisible}`);
    if (overlayStore.isVisible) {
      logger.info('PAGE', 'Overlay is visible, loading editor windows');
      editorWindowsStore.loadWindows();
      setTimeout(() => {
        logger.debug('PAGE', 'Focusing search bar');
        searchBarRef?.focus();
      }, 100);
    }
  } catch (error) {
    logger.error('PAGE', 'Error in $effect', error);
  }
});
</script>

<div class="min-h-screen bg-zinc-900/98 backdrop-blur-xl text-white rounded-2xl border border-zinc-700/50 shadow-2xl flex flex-col overflow-hidden">
  <div class="w-full max-w-2xl mx-auto flex-1 px-6 py-6">
    <EditorSearchBar
      bind:this={searchBarRef}
      onClear={clearSearch}
    />

    <EditorWindowsList
      editorWindows={filteredResults.map(result => result.item)}
      searchQuery={searchStore.query}
      onBringWindowToFront={handleBringWindowToFront}
    />
  </div>

  <FooterBar
    class="px-6 py-4"
    leftClass="text-zinc-400"
    middleClass="text-zinc-200"
    rightClass="text-zinc-400"
  >
    <FooterLabel
      slot="left"
      label="Hotkey"
      value={GLOBAL_SHORTCUT_KEY}
      class="truncate"
    />

    <div slot="middle" class="flex items-center gap-3">
      <FooterLabel
        label="Windows"
        value={filteredResults.length.toString()}
        class="shrink"
      />
      <span class="h-4 w-px bg-zinc-700" aria-hidden="true" />
      <span class="text-sm text-zinc-200">{overlayStatus}</span>
    </div>

    <FooterLabel
      slot="right"
      label="Status"
      value="Ready for shortcuts"
      class="truncate"
      valueClass="text-sm text-zinc-200"
    />
  </FooterBar>
</div>

<DevErrorBadge />

