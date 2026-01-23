import { RadiusTokens } from '../tokens';

export const radius = {
  '--radius-4': RadiusTokens.radius4,
  '--radius-8': RadiusTokens.radius8,
  '--radius-12': RadiusTokens.radius12,
  '--radius-16': RadiusTokens.radius16,
  '--radius-20': RadiusTokens.radius20,
  '--radius-24': RadiusTokens.radius24,
  '--radius-32': RadiusTokens.radius32,
  '--radius-48': RadiusTokens.radius48,
  '--radius-full': RadiusTokens.radiusFull,
};

export enum Radius {
  radius4 = 'var(--radius-4)',
  radius8 = 'var(--radius-8)',
  radius12 = 'var(--radius-12)',
  radius16 = 'var(--radius-16)',
  radius20 = 'var(--radius-20)',
  radius24 = 'var(--radius-24)',
  radius32 = 'var(--radius-32)',
  radius48 = 'var(--radius-48)',
  radiusFull = 'var(--radius-full)',
}
