<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { ui, mutate, errorText } from '../lib/state.svelte';
  import { t, locale } from '../lib/i18n.svelte';
  import type { ItemInput } from '../lib/generated/ItemInput';
  import type { Preset } from '../lib/generated/Preset';
  import Modal from '../lib/ui/Modal.svelte';
  import Icon from '../lib/ui/Icon.svelte';
  import ThingArt from '../lib/ui/ThingArt.svelte';
  import Appearance from '../lib/ui/Appearance.svelte';

  const original = ui.snapshot?.items.find(
    (v) => v.item.id === ui.editor,
  )?.item;
  let stage = $state(original ? 'form' : 'presets');
  let query = $state('');
  let error = $state('');
  let shelfLife = $state(false);
  let form = $state<ItemInput>(
    original
      ? structuredClone($state.snapshot(original.input))
      : {
          name: '',
          icon: 'sprout',
          color: 'sage',
          room_id: ui.room,
          started_on: ui.snapshot!.today,
          due_on: ui.snapshot!.today,
          repeat: 'months',
          every: 3,
          notifications: true,
          advance_days: 1,
          notes: '',
        },
  );
  let calculating = $state(false);
  let previewId = 0;
  const filtered = $derived(
    ui.presets.filter((p) =>
      `${p.ru} ${p.en}`.toLocaleLowerCase().includes(query.toLocaleLowerCase()),
    ),
  );
  async function recalculate() {
    const id = ++previewId;
    if (!form.started_on || !form.every) return;
    calculating = true;
    try {
      const next = await invoke<string>('preview_date', {
        startedOn: form.started_on,
        repeat: form.repeat === 'once' ? 'days' : form.repeat,
        every: form.every,
      });
      if (id === previewId) {
        form.due_on = next;
        error = '';
      }
    } catch (e) {
      if (id === previewId) error = errorText(e);
    } finally {
      if (id === previewId) calculating = false;
    }
  }
  async function choose(p: Preset) {
    form.name = locale.language === 'ru' ? p.ru : p.en;
    form.icon = p.icon;
    form.color = p.color;
    form.repeat = p.repeat;
    form.every = p.every;
    form.room_id =
      ui.room ?? ui.snapshot!.rooms.find((r) => r.id === p.room)?.id ?? null;
    shelfLife = p.shelf_life;
    stage = 'form';
    await recalculate();
  }
  async function submit(e: SubmitEvent) {
    e.preventDefault();
    const result = await mutate(
      'save_item',
      {
        id: original?.id ?? null,
        revision: original?.revision ?? null,
        input: $state.snapshot(form),
      },
      'saved',
    );
    if (result) {
      ui.editor = null;
      ui.selected = result.item_id;
    } else error = ui.toast?.message ?? t('error_generic');
  }
  onMount(() => {
    if (!original) recalculate();
  });
</script>

<Modal
  title={original ? t('edit_item') : t('new_item')}
  onclose={() => (ui.editor = null)}
  wide
>
  {#if stage === 'presets'}
    <p class="muted modal-intro">{t('new_item_subtitle')}</p>
    <div class="search-field large">
      <Icon name="search" /><input
        bind:value={query}
        placeholder={t('preset_search')}
        aria-label={t('preset_search')}
        data-initial-focus
      />
    </div>
    <button
      class="create-custom"
      onclick={() => {
        form.name = query;
        stage = 'form';
      }}
      ><span class="custom-icon"><Icon name="plus" /></span><span
        >{t('from_scratch')}</span
      ><Icon name="arrow" size={18} /></button
    >
    <h3 class="eyebrow preset-heading">{t('popular_presets')}</h3>
    <div class="preset-grid">
      {#each filtered as p}<button class="preset" onclick={() => choose(p)}
          ><span data-color={p.color} class="preset-art"
            ><ThingArt icon={p.icon} size={43} /></span
          ><span
            ><strong>{locale.language === 'ru' ? p.ru : p.en}</strong><small
              >{p.repeat === 'once'
                ? `${p.every} · ${t('days')}`
                : t('schedule_repeat', {
                    n: p.every,
                    unit: t(p.repeat).toLocaleLowerCase(),
                  })}</small
            ></span
          ></button
        >{:else}<p class="muted">{t('no_presets')}</p>{/each}
    </div>
    <p class="fine-print">{t('preset_note')}</p>
  {:else}
    <form onsubmit={submit} class="editor-form">
      {#if !original}<button
          class="text-button back-button"
          type="button"
          onclick={() => (stage = 'presets')}
          ><Icon name="back" size={16} />{t('back')}</button
        >{/if}
      <div class="form-identity">
        <span class="identity-art" data-color={form.color}
          ><ThingArt icon={form.icon} size={60} /></span
        ><label class="grow"
          >{t('name')}<input
            bind:value={form.name}
            required
            maxlength="120"
            placeholder={t('name_placeholder')}
            data-testid="item-name"
          /></label
        >
      </div>
      <label
        >{t('room')}<select bind:value={form.room_id}
          ><option value={null}>{t('no_room')}</option
          >{#each ui.snapshot!.rooms as room}<option value={room.id}
              >{room.name}</option
            >{/each}</select
        ></label
      >
      <div class="form-row">
        <label
          >{t('started_on')}<input
            type="date"
            min="1900-01-01"
            max="2200-12-31"
            bind:value={form.started_on}
            onchange={recalculate}
            required
          /></label
        ><label
          >{t('due_on')}<input
            type="date"
            min={form.started_on}
            max="2200-12-31"
            bind:value={form.due_on}
            required
            data-testid="due-on"
          /></label
        >
      </div>
      <div class="form-row">
        <label
          >{t('repeat')}<select
            bind:value={form.repeat}
            onchange={recalculate}
            data-testid="repeat"
            >{#each ['once', 'days', 'weeks', 'months', 'years'] as unit}<option
                value={unit}>{t(unit)}</option
              >{/each}</select
          ></label
        >{#if form.repeat !== 'once'}<label
            >{t('every')}<input
              type="number"
              min="1"
              max="999"
              bind:value={form.every}
              onchange={recalculate}
              required
              data-testid="interval"
            /></label
          >{/if}
      </div>
      <p class="field-hint">
        {shelfLife ? t('shelf_life_note') : t('interval_description')}
      </p>
      <details>
        <summary>{t('appearance')}<Icon name="down" size={16} /></summary
        ><Appearance bind:icon={form.icon} bind:color={form.color} />
      </details>
      <details>
        <summary>{t('extra_settings')}<Icon name="down" size={16} /></summary>
        <div class="details-content">
          <label class="toggle-row"
            ><span>{t('notifications')}</span><input
              type="checkbox"
              role="switch"
              bind:checked={form.notifications}
            /></label
          >{#if form.notifications}<label
              >{t('advance_days')}<input
                type="number"
                min="0"
                max="90"
                bind:value={form.advance_days}
                required
              /></label
            >{/if}<label
            >{t('notes')}<textarea
              rows="3"
              bind:value={form.notes}
              maxlength="4000"
              placeholder={t('notes_placeholder')}
            ></textarea></label
          >
        </div>
      </details>
      {#if error}<p class="inline-error" role="alert">{error}</p>{/if}
      <div class="form-footer">
        <button
          type="button"
          class="button secondary"
          onclick={() => (ui.editor = null)}>{t('cancel')}</button
        ><button
          class="button primary"
          type="submit"
          disabled={ui.busy || calculating}
          data-testid="save-item"
          ><Icon name={original ? 'check' : 'plus'} size={18} />{ui.busy
            ? t('saving')
            : original
              ? t('save')
              : t('create_item')}</button
        >
      </div>
    </form>
  {/if}
</Modal>
