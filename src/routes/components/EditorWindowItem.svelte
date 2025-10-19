<script lang="ts">
import type { WindowInfo } from '$lib/app/config';
import Icon from '$lib/ui/Icon.svelte';
import ListItem from '$lib/ui/ListItem.svelte';

interface Props {
  window: WindowInfo;
  index: number;
  selectedIndex: number;
  onBringWindowToFront: (pid: number, window_number: number) => Promise<void>;
}

let { window, index, selectedIndex, onBringWindowToFront }: Props = $props();

const isSelected = $derived(index === selectedIndex);
</script>

<ListItem 
  isSelected={isSelected}
  onClick={() => onBringWindowToFront(window.pid, window.window_number)}
>
  <div class="flex items-center gap-3">
    <Icon 
      src={window.app_icon ? `data:image/png;base64,${window.app_icon}` : null}
      alt="App icon"
      fallbackIcon="document"
      isHighlighted={isSelected}
      size="md"
    />

    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2">
        {#if window.project}
          <span class={`text-sm font-medium truncate ${isSelected ? 'text-white' : 'text-white/90'}`}>
            {window.project}
          </span>
        {/if}
        {#if window.active_editor_tab}
          <span class="text-white/40">•</span>
          <span class={`text-xs truncate ${isSelected ? 'text-white/70' : 'text-white/50'}`}>
            {window.active_editor_tab}
          </span>
        {/if}
      </div>
    </div>

    <div class="flex items-center gap-2">
      <div class={`rounded-sm flex items-center justify-center gap-1 px-2 py-1 transition-all duration-200 overflow-hidden ${
        isSelected
          ? 'bg-gradient-to-br from-blue-500/20 to-purple-500/20 border border-blue-400/30'
          : 'bg-white/10 border border-white/20'
      }`}>
        <span class="text-xs">Switch to editor</span>
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
        </svg>
      </div>
    </div>
  </div>
</ListItem>
