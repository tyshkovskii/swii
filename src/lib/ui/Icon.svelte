<script lang="ts">
interface Props {
  src?: string | null;
  alt?: string;
  fallbackIcon?: 'document' | 'folder' | 'app';
  isHighlighted?: boolean;
  size?: 'sm' | 'md' | 'lg';
}

let { 
  src = null, 
  alt = 'Icon', 
  fallbackIcon = 'document',
  isHighlighted = false,
  size = 'md'
}: Props = $props();

const hasIcon = $derived(src && src.length > 0);
let showFallback = $state(false);

function handleImageError(e: Event) {
  console.log("Image load error:", e);
  showFallback = true;
}

function handleImageLoad() {
  console.log("Image loaded successfully");
  showFallback = false;
}

// Size mappings
const sizeClasses = {
  sm: 'w-6 h-6',
  md: 'w-8 h-8',
  lg: 'w-10 h-10'
};

const iconSizeClasses = {
  sm: 'w-3 h-3',
  md: 'w-4 h-4',
  lg: 'w-5 h-5'
};

const containerSize = $derived(sizeClasses[size]);
const iconSize = $derived(iconSizeClasses[size]);

// Fallback icon SVG paths
const fallbackIcons = {
  document: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
  folder: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z',
  app: 'M4 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM14 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1V5zM4 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1v-4zM14 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z'
};
</script>

<div class={`${containerSize} rounded-lg flex-shrink-0 flex items-center justify-center transition-all duration-200 overflow-hidden ${
  isHighlighted
    ? 'bg-gradient-to-br from-blue-500/20 to-purple-500/20 border border-blue-400/30'
    : 'bg-white/10 border border-white/20'
}`}>
  {#if hasIcon && !showFallback}
    <img
      src={src}
      alt={alt}
      class={`${containerSize} object-contain`}
      onerror={handleImageError}
      onload={handleImageLoad}
    />
  {:else}
    <svg class={`${iconSize} ${isHighlighted ? 'text-blue-300' : 'text-white/60'}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={1.5} d={fallbackIcons[fallbackIcon]} />
    </svg>
  {/if}
</div>