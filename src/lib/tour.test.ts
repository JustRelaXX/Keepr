import { describe, it, expect } from 'vitest';
import { buildSteps } from './tour';

describe('first-run guide', () => {
  it('walks through five steps ending with a centered card', () => {
    const steps = buildSteps(true);
    expect(steps).toHaveLength(5);
    expect(steps[0].target).toBe('.health-panel');
    expect(steps[4].target).toBeNull();
    for (const step of steps) {
      expect(step.titleKey).toMatch(/^tour_step\d_title$/);
      expect(step.bodyKey).toMatch(/^tour_step\d(_empty)?_body$/);
    }
  });

  it('points at a real card when items exist, at the empty state otherwise', () => {
    expect(buildSteps(true)[2].target).toContain('.item-card');
    expect(buildSteps(false)[2].target).toContain('.peaceful');
  });
});
