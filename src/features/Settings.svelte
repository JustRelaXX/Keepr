<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    ui,
    mutate,
    notify,
    report,
    load,
    navigate,
  } from '../lib/state.svelte';
  import { startTour } from '../lib/tour.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { Mutation } from '../lib/generated/Mutation';
  import Icon from '../lib/ui/Icon.svelte';
  import Pet from '../lib/ui/Pet.svelte';
  import Modal from '../lib/ui/Modal.svelte';
  let settings = $state(
    structuredClone($state.snapshot(ui.snapshot!.settings)),
  );
  let autostart = $state(false);
  let systemBusy = $state(false);
  let confirmRestore = $state(false);
  onMount(async () => {
    try {
      autostart = await invoke<boolean>('autostart_status');
    } catch (e) {
      report(e);
    }
  });
  async function startup(e: Event) {
    const enabled = (e.currentTarget as HTMLInputElement).checked;
    systemBusy = true;
    try {
      await invoke('set_autostart', { enabled });
      autostart = enabled;
    } catch (error) {
      report(error);
      (e.target as HTMLInputElement).checked = autostart;
    } finally {
      systemBusy = false;
    }
  }
  async function test() {
    try {
      await invoke('test_notification');
      notify(t('notification_sent'));
    } catch (e) {
      report(e);
    }
  }
  async function backup() {
    systemBusy = true;
    try {
      if (await invoke<boolean>('export_backup')) notify(t('backup_saved'));
    } catch (e) {
      report(e);
    } finally {
      systemBusy = false;
    }
  }
  async function restore() {
    confirmRestore = false;
    systemBusy = true;
    try {
      if (await invoke<Mutation | null>('import_backup')) {
        await load();
        settings = structuredClone($state.snapshot(ui.snapshot!.settings));
        notify(t('backup_restored'));
      }
    } catch (e) {
      report(e);
    } finally {
      systemBusy = false;
    }
  }
</script>

<form
  class="settings-layout"
  onsubmit={async (e) => {
    e.preventDefault();
    await mutate(
      'save_settings',
      { settings: $state.snapshot(settings) },
      'saved',
    );
  }}
>
  <div class="settings-main">
    <section class="settings-section">
      <div class="section-heading">
        <h2>{t('personal')}</h2>
        <Icon name="home" size={20} />
      </div>
      <label
        >{t('home_name')}<input
          bind:value={settings.home_name}
          maxlength="60"
          placeholder={t('home_name_placeholder')}
        /></label
      >
      <div class="form-row">
        <label
          >{t('language')}<select bind:value={settings.language}
            ><option value="ru">Русский</option><option value="en"
              >English</option
            ></select
          ></label
        ><label
          >{t('theme')}<select bind:value={settings.theme}
            >{#each ['light', 'dark', 'system'] as theme}<option value={theme}
                >{t(theme)}</option
              >{/each}</select
          ></label
        >
      </div>
    </section>
    <section class="settings-section">
      <div class="section-heading">
        <h2>{t('notifications')}</h2>
        <Icon name="bell" size={20} />
      </div>
      <label class="toggle-row"
        ><span>{t('notifications')}</span><input
          type="checkbox"
          role="switch"
          bind:checked={settings.notifications}
        /></label
      >
      <div class="form-row">
        <label
          >{t('reminder_time')}<input
            type="time"
            bind:value={settings.reminder_time}
            required
          /></label
        ><label
          >{t('timezone')}<input
            bind:value={settings.timezone}
            required
            list="timezones"
          /></label
        >
      </div>
      <datalist id="timezones"
        >{#each ['Europe/Moscow', 'Europe/London', 'Europe/Berlin', 'Asia/Yekaterinburg', 'Asia/Novosibirsk', 'Asia/Vladivostok', 'America/New_York', 'America/Los_Angeles', 'Asia/Tokyo', 'UTC'] as zone}<option
            value={zone}
          ></option>{/each}</datalist
      >
      <h3>{t('quiet_hours')}</h3>
      <div class="form-row">
        <label
          >{t('quiet_start')}<input
            type="time"
            bind:value={settings.quiet_start}
            required
          /></label
        ><label
          >{t('quiet_end')}<input
            type="time"
            bind:value={settings.quiet_end}
            required
          /></label
        >
      </div>
      <p class="field-hint">{t('quiet_hint')}</p>
      <button type="button" class="button secondary" onclick={test}
        ><Icon name="bell" size={17} />{t('test_notification')}</button
      >
    </section>
    <section class="settings-section">
      <label class="toggle-row"
        ><span
          ><strong>{t('autostart')}</strong><small>{t('autostart_hint')}</small
          ></span
        ><input
          type="checkbox"
          role="switch"
          checked={autostart}
          onchange={startup}
          disabled={systemBusy}
        /></label
      >
      <p class="field-hint">{t('background_hint')}</p>
    </section>
    <div class="settings-save">
      <button class="button primary" type="submit" disabled={ui.busy}
        ><Icon name="check" size={18} />{t('save')}</button
      >
    </div>
  </div>
  <aside class="settings-side">
    <section class="settings-section companion-settings">
      <h2>{t('choose_pet')}</h2>
      <Pet kind={settings.pet} />
      <div class="pet-picker">
        {#each ['plant', 'cat', 'frog'] as pet}<button
            type="button"
            class:active={settings.pet === pet}
            aria-pressed={settings.pet === pet}
            onclick={() => (settings.pet = pet)}>{t(pet)}</button
          >{/each}
      </div>
      <p class="field-hint">{t('pet_description')}</p>
    </section>
    <section class="settings-section">
      <div class="section-heading">
        <h2>{t('data')}</h2>
        <Icon name="shield" size={20} />
      </div>
      <p class="field-hint">{t('backup_hint')}</p>
      <button
        type="button"
        class="button secondary full-width"
        disabled={systemBusy}
        onclick={backup}><Icon name="download" size={17} />{t('backup')}</button
      ><button
        type="button"
        class="text-button full-width"
        disabled={systemBusy}
        onclick={() => (confirmRestore = true)}
        ><Icon name="upload" size={17} />{t('restore_backup')}</button
      >
    </section>
    <section class="settings-section">
      <h2>{t('shortcuts')}</h2>
      <div class="shortcut-row">
        <span>{t('shortcut_add')}</span><kbd>Ctrl N</kbd>
      </div>
      <div class="shortcut-row">
        <span>{t('shortcut_search')}</span><kbd>Ctrl K</kbd>
      </div>
      <div class="shortcut-row">
        <span>{t('shortcut_close')}</span><kbd>Esc</kbd>
      </div>
      <button
        type="button"
        class="text-button full-width"
        onclick={() => {
          navigate('home');
          startTour();
        }}
        data-testid="replay-tour"
        ><Icon name="help" size={17} />{t('tour_replay')}</button
      >
    </section>
    <p class="version">{t('version')}</p>
  </aside>
</form>
{#if confirmRestore}<Modal
    title={t('restore_title')}
    onclose={() => (confirmRestore = false)}
    ><p class="muted">{t('restore_body')}</p>
    <div class="form-footer">
      <button class="button secondary" onclick={() => (confirmRestore = false)}
        >{t('cancel')}</button
      ><button class="button primary" onclick={restore}
        >{t('restore_backup')}</button
      >
    </div></Modal
  >{/if}
