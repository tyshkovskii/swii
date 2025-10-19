<script lang="ts">
import type { Snippet } from 'svelte';

interface Props {
  isSelected?: boolean;
  onClick?: () => void;
  children: Snippet;
}

let { isSelected = false, onClick, children }: Props = $props();
</script>

<div
  role="button"
  tabindex="0"
  class={`rounded-lg px-4 py-1 transition-all duration-200 cursor-pointer ${
    isSelected
      ? 'bg-white/20 border border-white/30 shadow-lg backdrop-blur-sm'
      : 'bg-white/5 border border-transparent hover:bg-white/10 hover:border-white/20'
  }`}
  onclick={onClick}
  onkeydown={(e) => e.key === 'Enter' && onClick?.()}
>
  {@render children()}
</div>
