<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke, isTauri } from '@tauri-apps/api/core';
  import {
    ui,
    initialize,
    load,
    route,
    navigate,
    mutate,
    type Page,
    errorText,
  } from './lib/state.svelte';
  import { t, locale, formatDate } from './lib/i18n.svelte';
  import Icon from './lib/ui/Icon.svelte';
  import Pet from './lib/ui/Pet.svelte';
  import Dashboard from './features/Dashboard.svelte';
  import Objects from './features/Objects.svelte';
  import Calendar from './features/Calendar.svelte';
  import History from './features/History.svelte';
  import Settings from './features/Settings.svelte';
  import ItemEditor from './features/ItemEditor.svelte';
  import ItemDetails from './features/ItemDetails.svelte';
  import RoomEditor from './features/RoomEditor.svelte';
  import logo from '../resources/artwork/icon.svg';

  let search = $state<HTMLInputElement>();
  let content = $state<HTMLElement>();
  $effect(() => {
    // Each destination starts at its heading, even after a long settings page.
    ui.page;
    ui.room;
    content?.scrollTo({ top: 0 });
  });
  let systemDark = $state(
    window.matchMedia('(prefers-color-scheme: dark)').matches,
  );
  const selectedRoom = $derived(
    ui.snapshot?.rooms.find((r) => r.id === ui.room),
  );
  const title = $derived(
    ui.page === 'home'
      ? t(
          new Date().getHours() < 12
            ? 'hello_morning'
            : new Date().getHours() < 18
              ? 'hello_day'
              : 'hello_evening',
        )
      : (selectedRoom?.name ?? t(ui.page)),
  );
  const subtitle = $derived(
    ui.page === 'home'
      ? t('dashboard_intro')
      : ui.page === 'calendar'
        ? t('calendar_intro')
        : ui.page === 'history'
          ? t('history_intro')
          : ui.page === 'settings'
            ? t('settings_intro')
            : selectedRoom
              ? t('items_count', {
                  n: ui.snapshot!.items.filter(
                    (v) =>
                      v.item.input.room_id === selectedRoom.id &&
                      !v.item.completed,
                  ).length,
                })
              : t('tagline'),
  );
  $effect(() => {
    const theme = ui.snapshot?.settings.theme ?? 'system';
    document.documentElement.dataset.theme =
      theme === 'system' ? (systemDark ? 'dark' : 'light') : theme;
    document.documentElement.lang = locale.language;
  });
  function keyboard(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      if (!ui.editor && !ui.roomEditor && !ui.selected) {
        navigate('objects');
        search?.focus();
      }
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n') {
      e.preventDefault();
      if (
        ui.snapshot?.settings.onboarding_done &&
        !ui.editor &&
        !ui.roomEditor
      ) {
        ui.selected = null;
        ui.editor = 'new';
      }
    }
  }
  onMount(() => {
    let disposed = false;
    const cleanups: (() => void)[] = [];
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const theme = () => (systemDark = media.matches);
    media.addEventListener('change', theme);
    const focus = () => {
      if (isTauri()) load();
    };
    window.addEventListener('focus', focus);
    (async () => {
      if (isTauri())
        for (const [name, handler] of [
          ['changed', () => load()],
          ['navigate', (event: { payload: string }) => route(event.payload)],
        ] as const) {
          const cleanup = await listen(name, handler as any);
          if (disposed) cleanup();
          else cleanups.push(cleanup);
        }
      await initialize();
      if (isTauri()) {
        const initial = await invoke<string | null>('initial_route');
        if (initial) route(initial);
      }
    })();
    return () => {
      disposed = true;
      cleanups.forEach((fn) => fn());
      media.removeEventListener('change', theme);
      window.removeEventListener('focus', focus);
    };
  });
  async function onboard(demo: boolean) {
    const result = await mutate('onboard', { demo }, '');
    if (result && !demo) ui.editor = 'new';
  }
</script>

<svelte:window onkeydown={keyboard} />
{#if ui.snapshot}
  {#if !ui.snapshot.settings.onboarding_done}
    <main class="welcome">
      <div class="welcome-brand"><img src={logo} alt="" />Keepr</div>
      <div class="welcome-content">
        <span class="welcome-pill"
          ><Icon name="sparkles" size={15} />{t('tagline')}</span
        >
        <div class="welcome-pet">
          <span class="welcome-orbit"></span><Pet />
        </div>
        <h1>{t('welcome')}</h1>
        <p>{t('welcome_body')}</p>
        <button
          class="button primary"
          disabled={ui.busy}
          onclick={() => onboard(false)}
          data-testid="start-empty"
          ><Icon name="plus" size={18} />{t('welcome_start')}</button
        ><button
          class="text-button"
          disabled={ui.busy}
          onclick={() => onboard(true)}
          data-testid="start-demo"
          >{t('welcome_demo')}<Icon name="arrow" size={16} /></button
        ><small><Icon name="shield" size={15} />{t('welcome_hint')}</small>
      </div>
      <div class="welcome-footer">
        <span>© 2026 Keepr</span><button
          class="text-button"
          onclick={async () => {
            await mutate(
              'save_settings',
              {
                settings: {
                  ...ui.snapshot!.settings,
                  language: locale.language === 'ru' ? 'en' : 'ru',
                },
              },
              '',
            );
          }}>{locale.language === 'ru' ? 'English' : 'Русский'}</button
        >
      </div>
    </main>
  {:else}
    <div class="app-shell">
      <aside class="sidebar">
        <button
          class="brand"
          onclick={() => navigate('home')}
          aria-label="Keepr"
          ><img src={logo} alt="" /><span
            >Keepr<span class="brand-dot">.</span></span
          ></button
        >
        <div class="home-switch">
          <span class="home-switch-icon"><Icon name="home" size={17} /></span
          ><span>{ui.snapshot.settings.home_name || t('home')}</span><span
            class="live-dot"
          ></span>
        </div>
        <nav aria-label={t('home')} class="main-nav">
          {#each ['home', 'objects', 'calendar', 'history'] as page}<button
              class:active={ui.page === page && !ui.room}
              onclick={() => navigate(page as Page)}
              aria-current={ui.page === page && !ui.room ? 'page' : undefined}
              ><Icon name={page} size={20} /><span>{t(page)}</span
              >{#if page === 'objects'}<span class="nav-count"
                  >{ui.snapshot.items.filter((v) => !v.item.completed)
                    .length}</span
                >{/if}</button
            >{/each}
        </nav>
        <div class="sidebar-section-label">
          <span>{t('rooms')}</span><button
            class="icon-button small"
            onclick={() => (ui.roomEditor = 'new')}
            aria-label={t('add_room')}><Icon name="plus" size={16} /></button
          >
        </div>
        <nav class="room-nav" aria-label={t('rooms')}>
          {#each ui.snapshot.rooms as room}<button
              class:active={ui.room === room.id}
              onclick={() => navigate('objects', room.id)}
              aria-current={ui.room === room.id ? 'page' : undefined}
              ><span class="room-dot" data-color={room.color}></span><span
                >{room.name}</span
              ><span class="room-count"
                >{ui.snapshot.items.filter(
                  (v) => v.item.input.room_id === room.id && !v.item.completed,
                ).length || ''}</span
              ></button
            >{/each}
        </nav>
        <button class="sidebar-add" onclick={() => (ui.roomEditor = 'new')}
          ><Icon name="plus" size={15} />{t('add_room')}</button
        >
        <div class="sidebar-bottom">
          <div class="sidebar-companion">
            <span
              ><Icon name="leaf" size={16} />{t('level', {
                n: ui.snapshot.health.level,
              })}</span
            >
            <p>{t('tagline')}</p>
            <div class="tiny-progress">
              <span style:width="{ui.snapshot.health.level_progress * 10}%"
              ></span>
            </div>
          </div>
          <button
            class="settings-nav"
            class:active={ui.page === 'settings'}
            onclick={() => navigate('settings')}
            ><Icon name="settings" size={19} />{t('settings')}</button
          ><span class="local-label"
            ><Icon name="shield" size={12} />{t('local_only')}</span
          >
        </div>
      </aside>
      <div class="workspace">
        <header class="topbar">
          <span class="breadcrumb"
            ><Icon
              name={ui.page === 'home' ? 'home' : ui.page}
              size={16}
            /><span>{selectedRoom?.name ?? t(ui.page)}</span></span
          >
          <div class="topbar-actions">
            <div class="search-field">
              <Icon name="search" size={16} /><input
                bind:this={search}
                value={ui.query}
                oninput={(e) => {
                  ui.query = e.currentTarget.value;
                  ui.page = 'objects';
                  ui.room = null;
                }}
                aria-label={t('search')}
                placeholder={t('search')}
              /><kbd>Ctrl K</kbd>
            </div>
            <button
              class="button primary add-button"
              onclick={() => (ui.editor = 'new')}
              data-testid="add-item"
              ><Icon name="plus" size={18} />{t('add')}</button
            >
          </div>
        </header>
        <main class="main-content" bind:this={content}>
          <div class="page-heading">
            <div>
              {#if ui.page === 'home'}<span class="eyebrow page-date"
                  >{formatDate(ui.snapshot.today, {
                    weekday: 'long',
                    day: 'numeric',
                    month: 'long',
                  })}</span
                >{/if}
              <h1>
                {title}{#if ui.page === 'home'}<span class="greeting-spark"
                    >✦</span
                  >{/if}
              </h1>
              <p>{subtitle}</p>
            </div>
            {#if selectedRoom}<button
                class="button secondary"
                onclick={() => (ui.roomEditor = selectedRoom.id)}
                ><Icon name="sliders" size={17} />{t('edit')}</button
              >{/if}
          </div>
          {#if ui.page === 'home'}<Dashboard
            />{:else if ui.page === 'objects'}<Objects
            />{:else if ui.page === 'calendar'}<Calendar
            />{:else if ui.page === 'history'}<History />{:else}<Settings
            />{/if}
        </main>
      </div>
    </div>
  {/if}
{:else}
  <main class="loading-screen">
    <img src={logo} alt="Keepr" />
    <h1>{ui.error ? t('connection_title') : t('loading')}</h1>
    {#if ui.error}<p>
        {ui.error === 'desktop_required'
          ? t('desktop_required')
          : errorText(ui.error)}
      </p>
      <button class="button primary" onclick={initialize}>{t('retry')}</button
      >{:else}<span class="loading-dots"><i></i><i></i><i></i></span>{/if}
  </main>
{/if}
{#if ui.selected && !ui.editor}<ItemDetails />{/if}
{#if ui.editor}{#key ui.editor}<ItemEditor />{/key}{/if}
{#if ui.roomEditor}{#key ui.roomEditor}<RoomEditor />{/key}{/if}
{#if ui.toast}<div
    class="toast"
    class:error={ui.toast.error}
    role={ui.toast.error ? 'alert' : 'status'}
  >
    <span class="toast-symbol"
      ><Icon name={ui.toast.error ? 'alert' : 'check'} size={18} /></span
    ><span>{ui.toast.message}</span>{#if ui.toast.undo}<button
        disabled={ui.busy}
        onclick={() =>
          mutate('undo', { eventId: ui.toast!.undo }, 'restored_toast')}
        >{t('undo')}</button
      >{/if}<button
      class="icon-button small"
      onclick={() => (ui.toast = null)}
      aria-label={t('close')}><Icon name="close" size={15} /></button
    >
  </div>{/if}
