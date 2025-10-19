<script lang="ts">
import type { Snippet } from 'svelte';

interface Props {
  children: Snippet<[number]>;
  onActiveIndexChange?: (index: number) => void;
  onSelect?: (index: number) => void;
  initialIndex?: number;
  itemCount: number;
}

let { 
  children, 
  onActiveIndexChange, 
  onSelect, 
  initialIndex = 0,
  itemCount 
}: Props = $props();

let activeIndex = $state(Math.min(Math.max(initialIndex, 0), itemCount - 1));

function clampIndex(idx: number): number {
  return Math.max(0, Math.min(idx, itemCount - 1));
}

function handleGlobalKeyDown(e: KeyboardEvent) {
  if (itemCount === 0) return;

  let delta = 0;
  if (e.key === 'ArrowDown') delta = 1;
  else if (e.key === 'ArrowUp') delta = -1;
  
  if (delta !== 0) {
    e.preventDefault();
    const next = clampIndex(activeIndex + delta);
    if (next !== activeIndex) {
      activeIndex = next;
      onActiveIndexChange?.(next);
    }
  } else if (e.key === 'Enter') {
    e.preventDefault();
    onSelect?.(activeIndex);
  }
}

$effect(() => {
  activeIndex = Math.min(Math.max(initialIndex, 0), itemCount - 1);
});
</script>

<svelte:window onkeydown={handleGlobalKeyDown} />

<div class="space-y-2">
  {#each Array(itemCount) as _, index}
    <div>
      {@render children(activeIndex === index ? activeIndex : index)}
    </div>
  {/each}
</div>