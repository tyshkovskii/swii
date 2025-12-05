<script lang="ts">
  import { clsx } from 'clsx';
  import type { HTMLAttributes } from 'svelte/elements';

  type FooterLabelProps = HTMLAttributes<HTMLDivElement> & {
    label: string;
    value: string;
    labelClass?: string;
    valueClass?: string;
  };

  const {
    label,
    value,
    class: className = '',
    labelClass = 'text-xs uppercase tracking-wide text-zinc-500',
    valueClass = 'text-sm font-medium text-zinc-100',
    ...restProps
  } = $props<FooterLabelProps>();

  const $$restProps = restProps satisfies HTMLAttributes<HTMLDivElement>;
</script>

<div class={clsx('flex items-center gap-2', className)} {...$$restProps}>
  <span class={labelClass}>{label}</span>
  <span class={valueClass}>{value}</span>
</div>
