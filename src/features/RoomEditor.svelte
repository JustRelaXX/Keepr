<script lang="ts">
  import { ui, mutate } from '../lib/state.svelte';
  import { t } from '../lib/i18n.svelte';
  import Modal from '../lib/ui/Modal.svelte';
  import Appearance from '../lib/ui/Appearance.svelte';
  import Icon from '../lib/ui/Icon.svelte';
  const original = ui.snapshot!.rooms.find((r) => r.id === ui.roomEditor);
  let room = $state(
    original
      ? structuredClone($state.snapshot(original))
      : { id: '', name: '', icon: 'home', color: 'sage' },
  );
  let confirmDelete = $state(false);
  let error = $state('');
  async function save(e: SubmitEvent) {
    e.preventDefault();
    if (
      await mutate('save_room', { room: $state.snapshot(room) }, 'room_saved')
    )
      ui.roomEditor = null;
    else error = ui.toast?.message ?? '';
  }
  async function remove() {
    if (await mutate('delete_room', { id: room.id }, 'room_deleted')) {
      ui.roomEditor = null;
      if (ui.room === room.id) ui.room = null;
    }
  }
</script>

<Modal
  title={original ? t('edit_room') : t('add_room')}
  onclose={() => (ui.roomEditor = null)}
>
  <form class="editor-form" onsubmit={save}>
    <label
      >{t('name')}<input
        bind:value={room.name}
        required
        maxlength="60"
        placeholder={t('room_name_placeholder')}
        data-testid="room-name"
      /></label
    >
    <Appearance bind:icon={room.icon} bind:color={room.color} />
    {#if error}<p class="inline-error" role="alert">{error}</p>{/if}
    {#if confirmDelete}<div class="confirm-box">
        <h3>{t('delete_room_title')}</h3>
        <p>{t('delete_room_body')}</p>
        <button
          type="button"
          class="button danger"
          onclick={remove}
          disabled={ui.busy}>{t('delete')}</button
        ><button
          type="button"
          class="text-button"
          onclick={() => (confirmDelete = false)}>{t('cancel')}</button
        >
      </div>{/if}
    <div class="form-footer">
      {#if original}<button
          type="button"
          class="icon-button danger-text"
          onclick={() => (confirmDelete = true)}
          aria-label={t('delete')}><Icon name="delete" /></button
        >{/if}<span class="grow"></span><button
        class="button primary"
        type="submit"
        disabled={ui.busy}>{t('save')}</button
      >
    </div>
  </form>
</Modal>
