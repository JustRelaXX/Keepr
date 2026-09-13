<script lang="ts">
  import { ui } from '../lib/state.svelte';
  import {
    t,
    locale,
    civilDate,
    dateKey,
    formatDate,
  } from '../lib/i18n.svelte';
  import Icon from '../lib/ui/Icon.svelte';
  import ItemCard from '../lib/ui/ItemCard.svelte';
  let month = $state(ui.snapshot!.today.slice(0, 7));
  let selected = $state(ui.snapshot!.today);
  const first = $derived(civilDate(`${month}-01`));
  const days = $derived.by(() => {
    const start = new Date(first);
    start.setUTCDate(1 - ((start.getUTCDay() + 6) % 7));
    return Array.from({ length: 42 }, (_, i) => {
      const d = new Date(start);
      d.setUTCDate(d.getUTCDate() + i);
      return dateKey(d);
    });
  });
  const weekdays = $derived(
    Array.from({ length: 7 }, (_, i) =>
      new Intl.DateTimeFormat(locale.language, {
        weekday: 'short',
        timeZone: 'UTC',
      }).format(new Date(Date.UTC(2026, 0, 5 + i))),
    ),
  );
  const active = $derived(ui.snapshot!.items.filter((v) => !v.item.completed));
  const agenda = $derived(
    active.filter((v) => v.item.input.due_on === selected),
  );
  function move(by: number) {
    const d = new Date(first);
    d.setUTCMonth(d.getUTCMonth() + by);
    month = dateKey(d).slice(0, 7);
  }
</script>

<div class="calendar-layout">
  <section class="calendar-panel">
    <div class="calendar-header">
      <h2>{formatDate(`${month}-01`, { month: 'long', year: 'numeric' })}</h2>
      <div>
        <button
          class="button subtle"
          onclick={() => {
            month = ui.snapshot!.today.slice(0, 7);
            selected = ui.snapshot!.today;
          }}>{t('this_month')}</button
        ><button
          class="icon-button"
          aria-label={t('previous_month')}
          onclick={() => move(-1)}><Icon name="left" /></button
        ><button
          class="icon-button"
          aria-label={t('next_month')}
          onclick={() => move(1)}><Icon name="right" /></button
        >
      </div>
    </div>
    <div class="calendar-weekdays">
      {#each weekdays as weekday}<span>{weekday}</span>{/each}
    </div>
    <div class="calendar-grid" aria-label={t('calendar')}>
      {#each days as day}{@const events = active.filter(
          (v) => v.item.input.due_on === day,
        )}<button
          class="calendar-day"
          class:outside={!day.startsWith(month)}
          class:today={day === ui.snapshot!.today}
          class:selected={day === selected}
          aria-pressed={day === selected}
          aria-label="{formatDate(day)} · {t('items_count', {
            n: events.length,
          })}"
          onclick={() => (selected = day)}
          ><span class="day-number">{Number(day.slice(-2))}</span><span
            class="day-events"
            >{#each events.slice(0, 2) as event}<span
                data-color={event.item.input.color}
                class:late={event.status === 'overdue'}
                >{event.item.input.name}</span
              >{/each}{#if events.length > 2}<small>+{events.length - 2}</small
              >{/if}</span
          ></button
        >{/each}
    </div>
    <p class="fine-print calendar-note">{t('calendar_note')}</p>
  </section>
  <aside class="agenda-panel">
    <span class="eyebrow">{t('day_agenda')}</span>
    <h2>{formatDate(selected)}</h2>
    {#each agenda as view}<ItemCard {view} compact />{:else}<div
        class="agenda-empty"
      >
        <Icon name="sun" size={32} />
        <p>{t('no_events_day')}</p>
      </div>{/each}
  </aside>
</div>
