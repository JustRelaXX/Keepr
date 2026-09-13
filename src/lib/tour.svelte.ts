export const tour = $state({ open: false, step: 0 });

function storage(): Storage | null {
  try {
    return typeof localStorage === 'undefined' ? null : localStorage;
  } catch {
    return null;
  }
}

export function isTourSeen(): boolean {
  return storage()?.getItem('keepr.tour.v1') === 'done';
}

export function startTour() {
  tour.step = 0;
  tour.open = true;
}

/** Any close (finish, skip, Esc) counts as seen — replay stays in Settings. */
export function closeTour() {
  tour.open = false;
  try {
    storage()?.setItem('keepr.tour.v1', 'done');
  } catch {
    /* private mode etc: the tour simply shows again next launch */
  }
}
