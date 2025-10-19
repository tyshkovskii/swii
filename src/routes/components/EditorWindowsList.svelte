<script lang="ts">
import type { WindowInfo } from '$lib/app/config';
import EmptyState from '$lib/ui/EmptyState.svelte';
import NavList from '$lib/ui/NavList.svelte';
import EditorWindowItem from './EditorWindowItem.svelte';

interface Props {
  editorWindows: WindowInfo[];
  searchQuery: string;
  onBringWindowToFront: (pid: number, window_number: number) => Promise<void>;
}

let { editorWindows, searchQuery, onBringWindowToFront }: Props = $props();

let selectedIndex = $state(0);

$effect(() => {
  selectedIndex = 0;
});

function handleSelect(index: number) {
  if (editorWindows[index]) {
    const window = editorWindows[index];
    onBringWindowToFront(window.pid, window.window_number);
  }
}

// Editor-specific empty state logic
const emptyStateTitle = $derived(
  searchQuery ? "No matching editors found" : "No editor windows found"
);

const emptyStateDescription = $derived(
  searchQuery ? "Try adjusting your search terms" : "Open some editor windows to get started"
);
</script>

{#if editorWindows.length === 0}
  <EmptyState 
    title={emptyStateTitle}
    description={emptyStateDescription}
  />
{:else}
  <NavList
    initialIndex={selectedIndex}
    onActiveIndexChange={(index) => selectedIndex = index}
    onSelect={handleSelect}
    itemCount={editorWindows.length}
  >
    {#snippet children(index)}
      <EditorWindowItem
        window={editorWindows[index]}
        {index}
        {selectedIndex}
        {onBringWindowToFront}
      />
    {/snippet}
  </NavList>
{/if}
