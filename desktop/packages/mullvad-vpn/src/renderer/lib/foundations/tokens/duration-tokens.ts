export const durationTokens = {
  instant: 0,
  faster: 0.1,
  fast: 0.15,
  normal: 0.25,
  slow: 0.4,
} as const;

export type DurationToken = keyof typeof durationTokens;
