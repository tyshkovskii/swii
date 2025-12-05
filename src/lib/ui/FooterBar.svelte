<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';

  type FooterProps = HTMLAttributes<HTMLElement> & {
    leftClass?: string;
    middleClass?: string;
    rightClass?: string;
  };

  const baseStyles =
    'flex w-full items-center gap-4 border-t border-zinc-800/70 bg-zinc-900/80 px-4 py-3 text-sm text-zinc-200 backdrop-blur';

  const {
    class: className = '',
    leftClass = '',
    middleClass = '',
    rightClass = '',
    ...restProps
  } = $props<FooterProps>();
</script>

<footer
  {...restProps}
  class={`${baseStyles} ${className}`.trim()}
>
  <div class={`flex min-w-0 items-center gap-2 text-left ${leftClass}`}>
    <slot name="left" />
  </div>

  <div class={`flex-1 flex min-w-0 items-center justify-center gap-2 text-center ${middleClass}`}>
    <slot name="middle" />
  </div>

  <div class={`flex min-w-0 items-center justify-end gap-2 text-right ${rightClass}`}>
    <slot name="right" />
  </div>
</footer>
