import { describe, it, expect } from 'vitest';
import ru from '../../resources/ru.json';
import en from '../../resources/en.json';
import presets from '../../resources/presets.json';
import { readFileSync } from 'node:fs';

describe('product resources', () => {
  it('provides matching Russian and English messages, including interpolations', () => {
    expect(Object.keys(ru).sort()).toEqual(Object.keys(en).sort());
    for (const key of Object.keys(ru) as (keyof typeof ru)[]) {
      expect(ru[key].length).toBeGreaterThan(0);
      expect(en[key].match(/\{\w+\}/g)?.sort() ?? []).toEqual(
        ru[key].match(/\{\w+\}/g)?.sort() ?? [],
      );
    }
  });
  it('ships specific, valid templates with stable identifiers', () => {
    expect(new Set(presets.map((p) => p.id)).size).toBe(presets.length);
    expect(presets.length).toBeGreaterThanOrEqual(30);
    for (const preset of presets) {
      expect(preset.every).toBeGreaterThan(0);
      expect(preset.every).toBeLessThan(1000);
      expect(preset.ru).toBeTruthy();
      expect(preset.en).toBeTruthy();
      expect(['once', 'days', 'weeks', 'months', 'years']).toContain(
        preset.repeat,
      );
    }
  });
  it('keeps the webview capabilities narrow', () => {
    const capabilities = JSON.parse(
      readFileSync('src-tauri/capabilities/main.json', 'utf8'),
    );
    expect(capabilities.permissions).toEqual([
      'core:event:allow-listen',
      'core:event:allow-unlisten',
    ]);
    const config = JSON.parse(
      readFileSync('src-tauri/tauri.conf.json', 'utf8'),
    );
    expect(config.app.windows).toEqual([]);
    expect(config.app.security.csp).not.toContain('unsafe-eval');
  });
});
