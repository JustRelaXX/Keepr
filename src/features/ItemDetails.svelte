<script lang="ts">
  import { ui, complete, defer, mutate } from '../lib/state.svelte';
  import { t, days, formatDate, formatMoment } from '../lib/i18n.svelte';
  import Modal from '../lib/ui/Modal.svelte';
  import Icon from '../lib/ui/Icon.svelte';
  import Ring from '../lib/ui/Ring.svelte';
  const view = $derived(
    ui.snapshot!.items.find((v) => v.item.id === ui.selected),
  );
  const room = $derived(
    ui.snapshot!.rooms.find((r) => r.id === view?.item.input.room_id),
  );
  const events = $derived(
    ui.snapshot!.events.filter((e) => e.item_id === ui.selected),
  );
  let showDefer = $state(false);
  let showDate = $state(false);
  let completedOn = $state(ui.snapshot!.today);
  let nextDue = $state(ui.snapshot!.today);
  async function remove() {
    if (!view) return;
    if (
      await mutate(
        'delete_item',
        { id: view.item.id, revision: view.item.revision },
        'deleted_toast',
      )
    )
      ui.selected = null;
  }
</script>

{#if view}
  <Modal title={t('item_details')} drawer onclose={() => (ui.selected = null)}>
    <div class="detail-hero" data-color={view.item.input.color}>
      <Ring
        value={view.remaining}
        status={view.status}
        icon={view.item.input.icon}
        size={116}
      /><span class="eyebrow">{room?.name ?? t('no_room')}</span>
      <h2>{view.item.input.name}</h2>
      <span class="status-pill" data-status={view.status}
        >{view.item.completed
          ? t('completed_label')
          : view.days_left === 0
            ? t('due_today')
            : view.days_left < 0
              ? t('late', { time: days(view.days_left) })
              : t('left', { time: days(view.days_left) })}</span
      >
    </div>
    {#if !view.item.completed}<div class="detail-actions">
        <button
          class="button primary"
          disabled={ui.busy}
          onclick={() => complete(view!)}
          ><Icon name="check" />{t('done')}</button
        ><button
          class="button secondary"
          onclick={() => (showDefer = !showDefer)}
          ><Icon name="clock" />{t('defer')}</button
        >
      </div>
      {#if showDefer}<div class="defer-options">
          {#each ['hour', 'evening', 'tomorrow'] as choice}<button
              class="button subtle"
              disabled={ui.busy}
              onclick={async () => {
                if (await defer(view!, choice)) showDefer = false;
              }}>{t(choice)}</button
            >{/each}
        </div>{/if}
      <button
        class="text-button date-toggle"
        onclick={() => (showDate = !showDate)}>{t('change_date')}</button
      >
      {#if showDate}<form
          class="completion-date"
          onsubmit={async (e) => {
            e.preventDefault();
            if (await complete(view!, completedOn)) showDate = false;
          }}
        >
          <label
            >{t('completed_on')}<input
              type="date"
              bind:value={completedOn}
              min={view.item.input.started_on}
              max={ui.snapshot!.today}
              required
            /></label
          ><button class="button secondary" disabled={ui.busy}
            >{t('confirm_done')}</button
          >
        </form>{/if}
    {/if}
    {#if view.item.completed}
      <form
        class="editor-form"
        onsubmit={async (e) => {
          e.preventDefault();
          await mutate(
            'reopen_item',
            {
              id: view!.item.id,
              revision: view!.item.revision,
              dueOn: nextDue,
            },
            'saved',
          );
        }}
      >
        <label
          >{t('due_on')}<input
            type="date"
            bind:value={nextDue}
            min={ui.snapshot!.today}
            max="2200-12-31"
            required
          /></label
        >
        <button class="button primary" type="submit" disabled={ui.busy}
          ><Icon name="plus" size={17} />{t('reopen')}</button
        >
      </form>
    {/if}
    {#if view.item.snoozed_until}<p class="detail-reminder">
        <Icon name="clock" size={15} />{t('deferred_until', {
          time: formatMoment(view.item.snoozed_until),
        })}
      </p>{/if}
    <dl class="detail-facts">
      <div>
        <dt>{t('due_on')}</dt>
        <dd>
          {formatDate(view.item.input.due_on, {
            day: 'numeric',
            month: 'long',
            year: 'numeric',
          })}
        </dd>
      </div>
      <div>
        <dt>{t('repeat')}</dt>
        <dd>
          {view.item.input.repeat === 'once'
            ? t('schedule_once')
            : t('schedule_repeat', {
                n: view.item.input.every,
                unit: t(view.item.input.repeat).toLocaleLowerCase(),
              })}
        </dd>
      </div>
      <div>
        <dt>{t('started_on')}</dt>
        <dd>{formatDate(view.item.input.started_on)}</dd>
      </div>
      <div>
        <dt>{t('notifications')}</dt>
        <dd>
          <Icon
            name={view.item.input.notifications ? 'bell' : 'moon'}
            size={17}
          />
        </dd>
      </div>
    </dl>
    {#if view.item.input.notes}<p class="detail-note">
        {view.item.input.notes}
      </p>{/if}
    <div class="section-heading">
      <h3>{t('item_history')}</h3>
      <span class="count-badge">{events.length}</span>
    </div>
    <div class="mini-timeline">
      {#each events as event}<div
          class="timeline-entry"
          class:undone={event.undone}
        >
          <span class="timeline-dot"
            ><Icon
              name={event.kind === 'done'
                ? 'check'
                : event.kind === 'deferred'
                  ? 'clock'
                  : 'leaf'}
              size={14}
            /></span
          >
          <div>
            <strong
              >{t(event.kind === 'done' ? 'done_event' : event.kind)}</strong
            ><small
              >{formatMoment(event.at)}{event.undone
                ? ` · ${t('undone')}`
                : ''}</small
            >{#if event.late_days > 0}<small
                >{t('late', { time: days(event.late_days) })}</small
              >{/if}
          </div>
        </div>{/each}
    </div>
    <div class="form-footer">
      <button
        class="button secondary"
        onclick={() => {
          ui.editor = view!.item.id;
          ui.selected = null;
        }}><Icon name="edit" size={17} />{t('edit')}</button
      ><span class="grow"></span><button
        class="icon-button danger-text"
        onclick={remove}
        disabled={ui.busy}
        aria-label={t('delete')}
        data-testid="delete-item"><Icon name="delete" size={19} /></button
      >
    </div>
  </Modal>
{/if}
