<script lang="ts">
import type { Snippet } from 'svelte';

interface Props {
  icon?: Snippet;
  title: string;
  description?: string;
}

let { icon, title, description }: Props = $props();
</script>

<div class="text-center py-8">
  <div class="w-12 h-12 mx-auto mb-3 rounded-full bg-white/10 flex items-center justify-center">
    {#if icon}
      {@render icon()}
    {:else}
      <svg class="w-6 h-6 text-white/40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={1.5} d="M9.172 16.172a4 4 0 015.656 0M9 12h6m-6-4h6m2 5.291A7.962 7.962 0 0112 15c-2.34 0-4.47-.881-6.08-2.33" />
      </svg>
    {/if}
  </div>
  <p class="text-white/60 text-sm font-medium">
    {title}
  </p>
  {#if description}
    <p class="text-white/40 text-xs mt-1">
      {description}
    </p>
  {/if}
</div>
