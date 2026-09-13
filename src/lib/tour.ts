export interface TourStep {
  /** CSS selector of the highlighted element, or null for a centered card. */
  target: string | null;
  titleKey: string;
  bodyKey: string;
}

/** Pure step list so it stays unit-testable without a DOM. */
export function buildSteps(hasItems: boolean): TourStep[] {
  return [
    {
      target: '.health-panel',
      titleKey: 'tour_step1_title',
      bodyKey: 'tour_step1_body',
    },
    {
      target: '.topbar-actions .add-button',
      titleKey: 'tour_step2_title',
      bodyKey: 'tour_step2_body',
    },
    hasItems
      ? {
          target: '.focus-section .item-card',
          titleKey: 'tour_step3_title',
          bodyKey: 'tour_step3_body',
        }
      : {
          target: '.focus-section .peaceful',
          titleKey: 'tour_step3_title',
          bodyKey: 'tour_step3_empty_body',
        },
    {
      target: '.focus-section .room-tiles',
      titleKey: 'tour_step4_title',
      bodyKey: 'tour_step4_body',
    },
    {
      target: null,
      titleKey: 'tour_step5_title',
      bodyKey: 'tour_step5_body',
    },
  ];
}
