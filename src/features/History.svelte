<script lang="ts">
  import { tick } from 'svelte';
  import { ui, mutate } from '../lib/state.svelte';
  import { t, formatMoment, days, locale, civilDate } from '../lib/i18n.svelte';
  import Icon from '../lib/ui/Icon.svelte';
  import ThingArt from '../lib/ui/ThingArt.svelte';
  import type { Event } from '../lib/generated/Event';
  let limit = $state(80);
  let journal = $state<HTMLDivElement>();
  let emptyTitle = $state<HTMLHeadingElement>();
  async function removeEntry(eventId: string) {
    const buttons = [
      ...(journal?.querySelectorAll<HTMLButtonElement>('.journal-delete') ??
        []),
    ];
    const index = buttons.findIndex(
      (button) => button.dataset.eventId === eventId,
    );
    if (await mutate('delete_event', { eventId }, 'entry_deleted_toast')) {
      await tick();
      const remaining =
        journal?.querySelectorAll<HTMLButtonElement>('.journal-delete');
      // Keep keyboard navigation in the list after its focused row disappears.
      if (remaining?.length)
        remaining[Math.max(0, Math.min(index, remaining.length - 1))].focus();
      else emptyTitle?.focus();
    }
  }
  const groups = $derived.by(() => {
    const result: Record<string, Event[]> = {};
    const today = civilDate(ui.snapshot!.today).getTime();
    for (const event of ui.snapshot!.events.slice(0, limit)) {
      const parts = new Intl.DateTimeFormat('en-CA', {
        timeZone: locale.timezone,
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
      }).formatToParts(new Date(event.at));
      const part = (type: string) => parts.find((p) => p.type === type)!.value;
      const day = civilDate(
        `${part('year')}-${part('month')}-${part('day')}`,
      ).getTime();
      const ago = Math.round((today - day) / 86400000);
      const key =
        ago === 0
          ? 'today'
          : ago === 1
            ? 'yesterday'
            : ago < 7
              ? 'this_week'
              : 'earlier';
      (result[key] ??= []).push(event);
    }
    return Object.entries(result);
  });
</script>

{#if ui.snapshot!.events.length}<div class="journal" bind:this={journal}>
    {#each groups as [group, events]}<section>
        <h2 class="journal-group">{t(group)}</h2>
        {#each events as event (event.id)}<div
            class="journal-entry"
            data-event-id={event.id}
          >
            <button
              class="journal-open"
              class:undone={event.undone}
              disabled={!ui.snapshot!.items.some(
                (v) => v.item.id === event.item_id,
              )}
              onclick={() => (ui.selected = event.item_id)}
              ><span class="journal-event-icon" data-kind={event.kind}
                ><Icon
                  name={event.kind === 'done'
                    ? 'check'
                    : event.kind === 'deferred'
                      ? 'clock'
                      : event.kind === 'deleted'
                        ? 'archive'
                        : 'leaf'}
                  size={18}
                /></span
              >
              <div class="journal-copy">
                <strong>{event.name}</strong><span
                  >{t(
                    event.kind === 'done' ? 'done_event' : event.kind,
                  )}{event.undone ? ` · ${t('undone')}` : ''}{event.late_days >
                  0
                    ? ` · ${t('late', { time: days(event.late_days) })}`
                    : ''}</span
                >{#if event.kind === 'deferred' && event.detail}<small
                    >{t('deferred_until', {
                      time: formatMoment(event.detail),
                    })}</small
                  >{/if}
              </div>
              <time>{formatMoment(event.at)}</time><ThingArt
                icon={event.icon}
                size={34}
              /></button
            >
            <button
              type="button"
              class="icon-button journal-delete"
              data-event-id={event.id}
              title={t('delete_entry')}
              aria-label={t('delete_entry_label', {
                name: event.name,
                time: formatMoment(event.at),
              })}
              disabled={ui.busy}
              onclick={() => removeEntry(event.id)}
              ><Icon name="delete" size={17} /></button
            >
          </div>{/each}
      </section>{/each}{#if ui.snapshot!.events.length > limit}<button
        class="button secondary show-more"
        onclick={() => (limit += 80)}>{t('show_more')}</button
      >{/if}
  </div>
{:else}<div class="empty-state">
    <span class="empty-illustration"><Icon name="history" size={42} /></span>
    <h2 bind:this={emptyTitle} tabindex="-1">{t('empty_history')}</h2>
    <p>{t('empty_history_body')}</p>
  </div>{/if}
