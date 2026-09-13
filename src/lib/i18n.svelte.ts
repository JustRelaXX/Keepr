import ru from '../../resources/ru.json';
import en from '../../resources/en.json';

export const locale = $state({ language: 'ru', timezone: 'UTC' });
export function t(
  key: string,
  values: Record<string, string | number> = {},
): string {
  const catalog: Record<string, string> = locale.language === 'en' ? en : ru;
  return (catalog[key] ?? (ru as Record<string, string>)[key] ?? key).replace(
    /\{(\w+)\}/g,
    (_, name: string) => String(values[name] ?? `{${name}}`),
  );
}
export function days(n: number): string {
  const rule = new Intl.PluralRules(locale.language).select(Math.abs(n));
  return t(`days_${rule === 'one' ? 'one' : rule === 'few' ? 'few' : 'many'}`, {
    n: Math.abs(n),
  });
}
/** Date-only values never go through the user's local timezone. */
export function civilDate(value: string): Date {
  return new Date(`${value}T12:00:00Z`);
}
export function formatDate(
  value: string,
  options: Intl.DateTimeFormatOptions = { day: 'numeric', month: 'long' },
): string {
  return new Intl.DateTimeFormat(locale.language, {
    ...options,
    timeZone: 'UTC',
  }).format(civilDate(value));
}
export function formatMoment(value: string): string {
  return new Intl.DateTimeFormat(locale.language, {
    day: 'numeric',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
    timeZone: locale.timezone,
  }).format(new Date(value));
}
export function dateKey(date: Date): string {
  return date.toISOString().slice(0, 10);
}
