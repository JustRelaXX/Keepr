<script lang="ts">
  import { tick } from 'svelte';
  import { onMount } from 'svelte';
  import { buildSteps, type TourStep } from '../lib/tour';
  import { tour, closeTour } from '../lib/tour.svelte';
  import { t } from '../lib/i18n.svelte';
  import Icon from '../lib/ui/Icon.svelte';

  let { hasItems = false }: { hasItems?: boolean } = $props();

  interface Spot {
    x: number;
    y: number;
    w: number;
    h: number;
  }

  const TIP_WIDTH = 320;
  const TIP_GAP = 14;

  let steps = $state<TourStep[]>([]);
  let spot = $state<Spot | null>(null);
  let heading = $state<HTMLHeadingElement>();

  const step = $derived(steps[tour.step]);
  const last = $derived(tour.step >= steps.length - 1);

  const tip = $derived.by(() => {
    if (!spot)
      return {
        x: Math.max(12, (window.innerWidth - TIP_WIDTH) / 2),
        y: Math.max(12, (window.innerHeight - 240) / 2),
        below: true,
      };
    const x = Math.min(
      Math.max(12, spot.x + spot.w / 2 - TIP_WIDTH / 2),
      window.innerWidth - TIP_WIDTH - 12,
    );
    const belowSpace = window.innerHeight - (spot.y + spot.h + TIP_GAP);
    // Tooltip height is unknown before render; 260px is a safe estimate.
    const below = belowSpace >= 260 || spot.y < 260;
    const y = below
      ? spot.y + spot.h + TIP_GAP
      : Math.max(12, spot.y - TIP_GAP - 260);
    return { x, y, below };
  });

  async function resolve() {
    steps = buildSteps(hasItems);
    if (tour.step >= steps.length) tour.step = steps.length - 1;
    await tick();
    const el = steps[tour.step]?.target
      ? document.querySelector(steps[tour.step].target!)
      : null;
    if (el) {
      el.scrollIntoView({ block: 'nearest', behavior: 'auto' });
      await tick();
      // Layout may still settle (fonts, images); measure on next frame.
      await new Promise((r) => requestAnimationFrame(() => r(null)));
      const r = el.getBoundingClientRect();
      spot = { x: r.x, y: r.y, w: r.width, h: r.height };
    } else {
      spot = null;
    }
    await tick();
    heading?.focus({ preventScroll: true });
  }

  function goto(delta: 1 | -1) {
    tour.step = Math.min(
      Math.max(0, tour.step + delta),
      buildSteps(hasItems).length - 1,
    );
    void resolve();
  }

  function onKey(e: KeyboardEvent) {
    if (!tour.open) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      closeTour();
    }
  }

  function onResize() {
    if (tour.open) void resolve();
  }

  // No reactive effects in this component by design: every state change
  // originates from an explicit event (open, step buttons, resize, mount),
  // so the tour can never ping-pong with app updates.
  onMount(() => {
    if (tour.open) void resolve();
  });
</script>

<svelte:window onkeydown={onKey} onresize={onResize} />

{#if tour.open && step}
  {#if spot}<div
      class="tour-spot"
      aria-hidden="true"
      style:left="{spot.x - 8}px"
      style:top="{spot.y - 8}px"
      style:width="{spot.w + 16}px"
      style:height="{spot.h + 16}px"
    ></div>{:else}<div class="tour-dim" aria-hidden="true"></div>{/if}
  <div
    class="tour-tooltip"
    role="dialog"
    aria-label={t(step.titleKey)}
    style:left="{tip.x}px"
    style:top="{tip.y}px"
  >
    <button
      type="button"
      class="icon-button small tour-skip"
      onclick={closeTour}
      aria-label={t('tour_skip')}
      data-testid="tour-skip"><Icon name="close" size={15} /></button
    >
    <span class="eyebrow tour-progress"
      >{t('tour_step', { n: tour.step + 1, total: steps.length })}</span
    >
    <h2 bind:this={heading} tabindex="-1">{t(step.titleKey)}</h2>
    <p>{t(step.bodyKey)}</p>
    <div class="tour-dots" aria-hidden="true">
      {#each steps as _, i}<span class:filled={i <= tour.step}></span>{/each}
    </div>
    <div class="tour-actions">
      <button
        type="button"
        class="button secondary"
        disabled={tour.step === 0}
        onclick={() => goto(-1)}
        data-testid="tour-back">{t('tour_back')}</button
      >
      {#if last}<button
          type="button"
          class="button primary"
          onclick={closeTour}
          data-testid="tour-done">{t('tour_done')}</button
        >{:else}<button
          type="button"
          class="button primary"
          onclick={() => goto(1)}
          data-testid="tour-next">{t('tour_next')}</button
        >{/if}
    </div>
  </div>
{/if}
