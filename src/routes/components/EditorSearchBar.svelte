<script lang="ts">
import { searchStore } from '$lib/stores/search.svelte';

interface Props {
  onClear?: () => void;
}

let { onClear }: Props = $props();
let inputElement: HTMLInputElement;

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    onClear?.();
  }
}

export function focus() {
  inputElement?.focus();
}
</script>

<div class="mb-4">
  <div class="relative">
    <input
      bind:this={inputElement}
      type="text"
      placeholder="Search editor windows..."
      bind:value={searchStore.query}
      onkeydown={handleKeyDown}
      class="w-full px-4 py-3 pl-12 bg-transparent border-b-[0.5px] border-white/30 text-white placeholder-white/50 focus:outline-none focus:border-white/70 text-base transition-all duration-200"
      spellcheck={false}
    />
    <svg class="absolute left-4 top-1/2 transform -translate-y-1/2 w-4 h-4 text-white/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
  </div>
</div>
