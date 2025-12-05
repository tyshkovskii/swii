<script lang="ts">
  import type { HTMLAttributes, HTMLElementTagNameMap } from 'svelte/elements';

  type FooterProps<Tag extends keyof HTMLElementTagNameMap = 'footer'> =
    HTMLAttributes<HTMLElementTagNameMap[Tag]> & {
      /**
       * Optionally render the footer as a different semantic element
       * while preserving the same layout and slot structure.
       */
      as?: Tag;
      /**
       * Overrides the base styling string. Set to an empty string when composing
       * your own outer classes without the provided defaults.
       */
      baseClass?: string;
      /**
       * Overrides the layout styling string controlling grid/flex behavior.
       */
      layoutClass?: string;
      /**
       * When true, prevents default base styles from being applied. Use together
       * with `class` or `baseClass` to fully control the wrapper styling.
       */
      unstyled?: boolean;
      leftClass?: string;
      middleClass?: string;
      rightClass?: string;
    };

  const baseStyles =
    'w-full border-t border-zinc-800/70 bg-zinc-900/80 px-4 py-3 text-sm text-zinc-200 backdrop-blur';

  const layoutStyles = 'grid grid-cols-1 gap-3 md:flex md:items-center md:gap-4';

  const {
    as = 'footer',
    baseClass = baseStyles,
    layoutClass = layoutStyles,
    unstyled = false,
    class: className = '',
    leftClass = 'flex min-w-0 items-center gap-2 text-left',
    middleClass = 'flex-1 flex min-w-0 items-center justify-center gap-2 text-center',
    rightClass = 'flex min-w-0 items-center justify-end gap-2 text-right',
    ...restProps
  } = $props<FooterProps>();

  const $$restProps = restProps satisfies HTMLAttributes<HTMLElement>;
</script>

<svelte:element
  this={as}
  role={as === 'footer' ? 'contentinfo' : undefined}
  {...$$restProps}
  class={`${unstyled ? '' : baseClass} ${layoutClass} ${className}`.trim()}
>
  <div class={`${leftClass}`}>
    <slot name="left" />
  </div>

  <div class={`${middleClass}`}>
    <slot name="middle" />
  </div>

  <div class={`${rightClass}`}>
    <slot name="right" />
  </div>
</svelte:element>
