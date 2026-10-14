import { type DurationToken, durationTokens } from '../tokens';

export const durationVariables = {
  '--duration-instant': `${durationTokens.instant}s`,
  '--duration-faster': `${durationTokens.faster}s`,
  '--duration-fast': `${durationTokens.fast}s`,
  '--duration-normal': `${durationTokens.normal}s`,
  '--duration-slow': `${durationTokens.slow}s`,
} as const;

export const durations: Record<DurationToken, string> = {
  instant: 'var(--duration-instant)',
  faster: 'var(--duration-faster)',
  fast: 'var(--duration-fast)',
  normal: 'var(--duration-normal)',
  slow: 'var(--duration-slow)',
} as const;

export type Durations = keyof typeof durations;
