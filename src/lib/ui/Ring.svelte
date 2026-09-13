<script lang="ts">
  import ThingArt from './ThingArt.svelte';
  import { t } from '../i18n.svelte';
  let {
    value,
    status,
    icon,
    size = 76,
  }: { value: number; status: string; icon: string; size?: number } = $props();
</script>

<div
  class="ring"
  class:urgent={status === 'overdue' || status === 'today'}
  class:soon={status === 'soon'}
  class:finished={status === 'completed'}
  style:width="{size}px"
  style:height="{size}px"
  role="img"
  aria-label={t('progress_label', { n: Math.round(value * 100) })}
>
  <svg viewBox="0 0 80 80" class="ring-track" aria-hidden="true"
    ><circle cx="40" cy="40" r="36" /><circle
      cx="40"
      cy="40"
      r="36"
      class="ring-progress"
      pathLength="100"
      stroke-dasharray="{Math.max(value * 100, 3)} 100"
    /></svg
  >
  <ThingArt {icon} size={size * 0.65} />
</div>
