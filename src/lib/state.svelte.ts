import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Snapshot } from './generated/Snapshot';
import type { Mutation } from './generated/Mutation';
import type { Preset } from './generated/Preset';
import type { ItemView } from './generated/ItemView';
import { locale, t } from './i18n.svelte';

export type Page = 'home' | 'objects' | 'calendar' | 'history' | 'settings';
export const ui = $state({
  snapshot: null as Snapshot | null,
  presets: [] as Preset[],
  page: 'home' as Page,
  room: null as string | null,
  filter: 'active',
  query: '',
  selected: null as string | null,
  editor: null as 'new' | string | null,
  roomEditor: null as 'new' | string | null,
  busy: false,
  error: '',
  toast: null as {
    message: string;
    undo: string | null;
    error: boolean;
    id: number;
  } | null,
});

let loadId = 0;
let toastTimer: ReturnType<typeof setTimeout>;
function accept(snapshot: Snapshot) {
  ui.snapshot = snapshot;
  locale.language = snapshot.settings.language;
  locale.timezone = snapshot.settings.timezone;
  ui.error = '';
}

export async function load(): Promise<void> {
  const id = ++loadId;
  try {
    if (!isTauri()) {
      ui.error = 'desktop_required';
      return;
    }
    const snapshot = await invoke<Snapshot>('snapshot');
    if (id === loadId) accept(snapshot);
  } catch (error) {
    if (id === loadId) ui.error = String(error);
  }
}

export async function initialize(): Promise<void> {
  await load();
  if (!ui.snapshot) return;
  try {
    ui.presets = await invoke<Preset[]>('presets');
  } catch (e) {
    report(e);
  }
}

export function notify(
  message: string,
  undo: string | null = null,
  error = false,
) {
  clearTimeout(toastTimer);
  ui.toast = { message, undo, error, id: Date.now() };
  toastTimer = setTimeout(
    () => {
      ui.toast = null;
    },
    undo ? 14000 : 6500,
  );
}
export function errorText(error: unknown): string {
  const key = `error_${String(error)}`;
  const text = t(key);
  return text === key ? t('error_generic') : text;
}
export function report(error: unknown) {
  notify(errorText(error), null, true);
}

export async function mutate(
  command: string,
  args: Record<string, unknown>,
  message: string,
): Promise<Mutation | null> {
  if (ui.busy) return null;
  ui.busy = true;
  try {
    const result = await invoke<Mutation>(command, args);
    ++loadId;
    accept(result.snapshot);
    if (message) notify(t(message), result.undo_id);
    return result;
  } catch (e) {
    report(e);
    await load();
    return null;
  } finally {
    ui.busy = false;
  }
}

export async function complete(view: ItemView, date?: string) {
  return mutate(
    'complete_item',
    {
      id: view.item.id,
      revision: view.item.revision,
      completedOn: date ?? ui.snapshot!.today,
    },
    'done_toast',
  );
}
export async function defer(view: ItemView, choice: string) {
  return mutate(
    'defer_item',
    { id: view.item.id, revision: view.item.revision, choice },
    'deferred_toast',
  );
}
export function navigate(
  page: Page,
  room: string | null = null,
  filter = 'active',
) {
  ui.page = page;
  ui.room = room;
  ui.filter = filter;
  ui.query = '';
  ui.selected = null;
}
export function route(value: string) {
  if (value === 'add') {
    ui.editor = 'new';
    return;
  }
  if (value === 'today' || value === 'overdue') {
    navigate('objects', null, value);
    return;
  }
  if (['home', 'objects', 'calendar', 'history', 'settings'].includes(value))
    navigate(value as Page);
}
