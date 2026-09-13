<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import { t } from '../i18n.svelte';
  import Icon from './Icon.svelte';
  import { ui } from '../state.svelte';
  let {
    title,
    onclose,
    children,
    wide = false,
    drawer = false,
  }: {
    title: string;
    onclose: () => void;
    children: Snippet;
    wide?: boolean;
    drawer?: boolean;
  } = $props();
  let dialog: HTMLDialogElement;
  onMount(() => {
    const previous = document.activeElement as HTMLElement | null;
    dialog.showModal();
    dialog.querySelector<HTMLInputElement>('[data-initial-focus]')?.focus();
    return () => previous?.focus();
  });
</script>

<dialog
  bind:this={dialog}
  class:wide
  class:drawer
  oncancel={(e) => {
    e.preventDefault();
    onclose();
  }}
  onclick={(e) => {
    if (e.target === dialog) {
      const r = dialog.getBoundingClientRect();
      if (
        e.clientX < r.left ||
        e.clientX > r.right ||
        e.clientY < r.top ||
        e.clientY > r.bottom
      )
        onclose();
    }
  }}
  aria-label={title}
>
  <div class="modal-header">
    <h2>{title}</h2>
    <button class="icon-button" onclick={onclose} aria-label={t('close')}
      ><Icon name="close" /></button
    >
  </div>
  <div class="modal-body">
    {#if ui.toast?.error}<p class="inline-error" role="alert">
        {ui.toast.message}
      </p>{/if}{@render children()}
  </div>
</dialog>
