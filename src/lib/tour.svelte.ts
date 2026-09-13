import { mutate, ui } from './state.svelte';

export const tour = $state({ open: false, step: 0 });

/** Before the first snapshot loads, pretend the tour was seen. */
export function isTourSeen(): boolean {
  return ui.snapshot?.settings.tour_seen ?? true;
}

export function startTour() {
  tour.step = 0;
  tour.open = true;
}

/** Any close (finish, skip, Esc) persists — replay stays in Settings. */
export function closeTour() {
  tour.open = false;
  const settings = ui.snapshot?.settings;
  if (settings && !settings.tour_seen) {
    // Optimistic local mark: blocks the kickoff effect from reopening the
    // tour in the gap before the save round-trip lands. The reload after
    // save confirms the same value from SQLite.
    settings.tour_seen = true;
    const next = structuredClone($state.snapshot(settings));
    void mutate('save_settings', { settings: next }, '');
  }
}
