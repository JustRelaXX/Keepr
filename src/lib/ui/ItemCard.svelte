<script lang="ts">
  import type { ItemView } from '../generated/ItemView';
  import { t, days, formatMoment } from '../i18n.svelte';
  import { ui, complete, defer } from '../state.svelte';
  import Icon from './Icon.svelte';
  import Ring from './Ring.svelte';
  let { view, compact = false }: { view: ItemView; compact?: boolean } =
    $props();
  let menu = $state(false);
  const room = $derived(
    ui.snapshot?.rooms.find((r) => r.id === view.item.input.room_id),
  );
  const label = $derived(
    view.item.completed
      ? t('completed_label')
      : view.days_left === 0
        ? t('due_today')
        : view.days_left < 0
          ? t('late', { time: days(view.days_left) })
          : t('left', { time: days(view.days_left) }),
  );
  const postponed = $derived(
    view.item.snoozed_until &&
      new Date(view.item.snoozed_until).getTime() > Date.now(),
  );
</script>

<article
  class="item-card"
  class:compact
  data-color={view.item.input.color}
  data-testid="item-card"
>
  <button
    class="item-main"
    onclick={() => (ui.selected = view.item.id)}
    aria-label="{view.item.input.name} · {label}"
  >
    <Ring
      value={view.remaining}
      status={view.status}
      icon={view.item.input.icon}
      size={compact ? 55 : 78}
    />
    <div class="item-copy">
      <span class="item-room">{room?.name ?? t('no_room')}</span>
      <h3>{view.item.input.name}</h3>
      <span class="status-label" data-status={view.status}
        ><span class="status-dot"></span>{label}</span
      >
    </div>
  </button>
  {#if postponed}<span class="snoozed"
      ><Icon name="clock" size={12} />{t('deferred_until', {
        time: formatMoment(view.item.snoozed_until!),
      })}</span
    >{/if}
  {#if !view.item.completed}
    <div class="item-actions">
      <button
        class="care-button"
        disabled={ui.busy}
        onclick={() => complete(view)}
        aria-label="{t('done')}: {view.item.input.name}"
        ><Icon name="check" size={16} /><span>{t('done')}</span></button
      >
      <div class="popover-wrap">
        <button
          class="icon-button small"
          onclick={() => (menu = !menu)}
          aria-label="{t('defer')}: {view.item.input.name}"
          aria-expanded={menu}><Icon name="clock" size={17} /></button
        >
        {#if menu}
          <div class="action-popover" role="group" aria-label={t('defer')}>
            {#each ['hour', 'evening', 'tomorrow'] as choice}<button
                disabled={ui.busy}
                onclick={() => {
                  menu = false;
                  defer(view, choice);
                }}>{t(choice)}</button
              >{/each}
            <button onclick={() => (menu = false)}>{t('cancel')}</button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</article>
