<script lang="ts">
  import { ui, navigate, mutate } from '../lib/state.svelte';
  import { t, formatDate } from '../lib/i18n.svelte';
  import Pet from '../lib/ui/Pet.svelte';
  import Icon from '../lib/ui/Icon.svelte';
  import ItemCard from '../lib/ui/ItemCard.svelte';
  import ThingArt from '../lib/ui/ThingArt.svelte';
  import Modal from '../lib/ui/Modal.svelte';
  const snapshot = $derived(ui.snapshot!);
  const urgent = $derived(
    snapshot.items.filter((v) => !v.item.completed && v.days_left <= 0),
  );
  const next = $derived(
    snapshot.items
      .filter((v) => !v.item.completed && v.days_left > 0)
      .slice(0, 4),
  );
  let showReset = $state(false);
  async function resetDemo() {
    if (await mutate('reset_home', {}, 'demo_cleared')) showReset = false;
  }
</script>

{#if snapshot.settings.demo_home}
  <div class="demo-banner" data-testid="demo-banner">
    <span class="demo-banner-copy"
      ><Icon name="sparkles" size={18} />{t('demo_banner_body')}</span
    >
    <button
      type="button"
      class="button secondary"
      data-testid="demo-reset"
      onclick={() => (showReset = true)}>{t('demo_reset')}</button
    >
  </div>
{/if}

<div class="dashboard-top">
  <section class="health-panel" aria-label={t('home_health')}>
    <div class="health-copy">
      <span class="eyebrow"
        ><span class="live-dot"></span>{t('home_health')}</span
      >
      <div class="health-number">
        {snapshot.health.score}<span>/ 100</span><Icon
          name="sparkles"
          size={25}
        />
      </div>
      <h2>{t(`health_${snapshot.health.mood}`)}</h2>
      <p>{t(`pet_says_${snapshot.health.mood}`)}</p>
      <div
        class="health-track"
        role="progressbar"
        aria-label={t('home_health')}
        aria-valuenow={snapshot.health.score}
        aria-valuemin="0"
        aria-valuemax="100"
      >
        <span style:width="{snapshot.health.score}%"></span>
      </div>
    </div>
    <div class="health-pet">
      <span class="pet-orbit orbit-one"></span><span class="pet-orbit orbit-two"
      ></span><Pet
        kind={snapshot.settings.pet}
        mood={snapshot.health.mood}
      /><span class="pet-name"
        ><span class="live-dot"></span>{t(snapshot.settings.pet)}</span
      >
    </div>
  </section>
  <section class="progress-panel">
    <span class="eyebrow">{t('your_companion')}</span>
    <div class="level-badge">
      <Icon name="star" size={27} /><span>{snapshot.health.level}</span>
    </div>
    <h3>{t('level', { n: snapshot.health.level })}</h3>
    <p>{t('care_points', { n: snapshot.health.level_progress })}</p>
    <div class="level-steps" aria-hidden="true">
      {#each Array(10) as _, i}<span
          class:filled={i < snapshot.health.level_progress}
        ></span>{/each}
    </div>
    <span class="total-cares"
      ><Icon name="heart" size={15} />{t('care_total')}
      <strong>{snapshot.health.completions}</strong></span
    >
  </section>
</div>
<div class="quick-stats">
  <button onclick={() => navigate('objects', null, 'overdue')}
    ><span class="stat-symbol peach"><Icon name="clock" size={19} /></span><span
      >{t('overdue')}</span
    ><strong>{snapshot.health.overdue}</strong></button
  ><button onclick={() => navigate('objects', null, 'today')}
    ><span class="stat-symbol yellow"><Icon name="sun" size={19} /></span><span
      >{t('today')}</span
    ><strong>{snapshot.health.today}</strong></button
  ><button onclick={() => navigate('objects', null, 'soon')}
    ><span class="stat-symbol lavender"><Icon name="calendar" size={19} /></span
    ><span>{t('soon')}</span><strong>{snapshot.health.soon}</strong></button
  >
</div>
<div class="dashboard-bottom">
  <section class="focus-section">
    <div class="section-heading">
      <div>
        <h2>{t('focus_title')}</h2>
        <p>{t('focus_subtitle')}</p>
      </div>
      <span class="count-badge">{urgent.length}</span>
    </div>
    {#if urgent.length}<div class="cards-grid">
        {#each urgent.slice(0, 6) as view (view.item.id)}<ItemCard
            {view}
          />{/each}
      </div>
      {#if urgent.length > 6}<button
          class="text-button"
          onclick={() => navigate('objects', null, 'overdue')}
          >{t('view_all')}<Icon name="arrow" size={16} /></button
        >{/if}
    {:else}<div class="peaceful">
        <span class="peaceful-art"
          ><Icon name="leaf" size={36} /><span>✦</span></span
        >
        <h3>{t('nothing_urgent')}</h3>
        <p>{t('nothing_urgent_body')}</p>
        {#if snapshot.items.length === 0}<button
            class="button primary"
            onclick={() => (ui.editor = 'new')}
            ><Icon name="plus" size={18} />{t('add')}</button
          >{/if}
      </div>{/if}
    <div class="section-heading room-section-heading">
      <h2>{t('room_overview')}</h2>
      <button
        class="icon-button small"
        onclick={() => (ui.roomEditor = 'new')}
        aria-label={t('add_room')}><Icon name="plus" size={18} /></button
      >
    </div>
    <div class="room-tiles">
      {#each snapshot.rooms as room}<button
          class="room-tile"
          data-color={room.color}
          onclick={() => navigate('objects', room.id)}
          ><span class="room-tile-icon"
            ><Icon name={room.icon} size={23} /></span
          ><strong>{room.name}</strong><small
            >{t('items_count', {
              n: snapshot.items.filter(
                (v) => v.item.input.room_id === room.id && !v.item.completed,
              ).length,
            })}</small
          ></button
        >{/each}<button
        class="room-tile new-room"
        onclick={() => (ui.roomEditor = 'new')}
        ><Icon name="plus" size={24} /><span>{t('add_room')}</span></button
      >
    </div>
  </section>
  <aside class="upcoming">
    <div class="section-heading">
      <h2>{t('up_next')}</h2>
      <Icon name="calendar" size={19} />
    </div>
    <div class="upcoming-list">
      {#each next as view}<button
          class="upcoming-item"
          onclick={() => (ui.selected = view.item.id)}
          ><span class="upcoming-art" data-color={view.item.input.color}
            ><ThingArt icon={view.item.input.icon} size={39} /></span
          ><span
            ><strong>{view.item.input.name}</strong><small
              >{formatDate(view.item.input.due_on)}</small
            ></span
          ><Icon name="right" size={15} /></button
        >{:else}<p class="muted">{t('nothing_urgent_body')}</p>{/each}
    </div>
    <button class="text-button" onclick={() => navigate('calendar')}
      >{t('calendar')}<Icon name="arrow" size={16} /></button
    >
    <div class="home-note">
      <Icon name="shield" size={22} />
      <p>{t('offline')}</p>
      <small>{t('local_only')}</small>
    </div>
  </aside>
</div>
{#if showReset}<Modal title={t('demo_reset_title')} onclose={() => (showReset = false)}
  ><p class="muted">{t('demo_reset_body')}</p>
  <div class="form-footer">
    <button
      type="button"
      class="button secondary"
      onclick={() => (showReset = false)}>{t('cancel')}</button
    ><button
      type="button"
      class="button danger"
      disabled={ui.busy}
      data-testid="demo-reset-confirm"
      onclick={resetDemo}>{t('demo_reset')}</button
    >
  </div></Modal
>{/if}
