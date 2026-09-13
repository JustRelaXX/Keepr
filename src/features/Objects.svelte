<script lang="ts">
  import { ui } from '../lib/state.svelte';
  import { t, locale } from '../lib/i18n.svelte';
  import Icon from '../lib/ui/Icon.svelte';
  import ItemCard from '../lib/ui/ItemCard.svelte';
  let sort = $state('due');
  let limit = $state(60);
  const filtered = $derived(
    ui
      .snapshot!.items.filter((v) => {
        if (ui.room && v.item.input.room_id !== ui.room) return false;
        if (
          ui.query &&
          !`${v.item.input.name} ${v.item.input.notes}`
            .toLocaleLowerCase()
            .includes(ui.query.toLocaleLowerCase())
        )
          return false;
        if (ui.filter === 'active') return !v.item.completed;
        if (ui.filter === 'all') return true;
        return v.status === ui.filter;
      })
      .sort((a, b) =>
        sort === 'name'
          ? a.item.input.name.localeCompare(b.item.input.name, locale.language)
          : sort === 'room'
            ? (a.item.input.room_id ?? '').localeCompare(
                b.item.input.room_id ?? '',
              )
            : a.item.input.due_on.localeCompare(b.item.input.due_on),
      ),
  );
</script>

<div class="list-toolbar">
  <div class="filter-tabs" role="group" aria-label={t('objects')}>
    {#each ['active', 'overdue', 'today', 'soon', 'completed', 'all'] as filter}<button
        class:active={ui.filter === filter}
        aria-pressed={ui.filter === filter}
        onclick={() => {
          ui.filter = filter;
          limit = 60;
        }}>{t(filter)}</button
      >{/each}
  </div>
  <select bind:value={sort} aria-label={t('sort')} class="sort-select"
    >{#each ['due', 'name', 'room'] as key}<option value={key}
        >{t(`sort_${key}`)}</option
      >{/each}</select
  >
</div>
{#if filtered.length}<div class="cards-grid objects-grid">
    {#each filtered.slice(0, limit) as view (view.item.id)}<ItemCard
        {view}
      />{/each}
  </div>
  {#if filtered.length > limit}<button
      class="button secondary show-more"
      onclick={() => (limit += 60)}>{t('show_more')}</button
    >{/if}
{:else}<div class="empty-state">
    <span class="empty-illustration"
      ><Icon name={ui.query ? 'search' : 'sprout'} size={44} /></span
    >
    <h2>
      {ui.query
        ? t('no_results')
        : ui.room
          ? t('empty_room')
          : t('empty_objects')}
    </h2>
    <p>
      {ui.query
        ? t('no_results_body')
        : ui.room
          ? t('empty_room_body')
          : t('empty_objects_body')}
    </p>
    {#if ui.query || ui.filter !== 'active'}<button
        class="button secondary"
        onclick={() => {
          ui.query = '';
          ui.filter = 'active';
        }}>{t('clear_filters')}</button
      >{:else}<button class="button primary" onclick={() => (ui.editor = 'new')}
        ><Icon name="plus" size={18} />{t('add')}</button
      >{/if}
  </div>{/if}
