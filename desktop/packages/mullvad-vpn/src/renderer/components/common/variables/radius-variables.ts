import { RadiusTokens } from '../../../tokens';

export const radius = {
  '--radius-4': RadiusTokens.radius4,
  '--radius-8': RadiusTokens.radius8,
  '--radius-11': RadiusTokens.radius11,
  '--radius-12': RadiusTokens.radius12,
};

export enum Radius {
  radius4 = 'var(--radius-4)',
  radius8 = 'var(--radius-8)',
  radius11 = 'var(--radius-11)',
  radius12 = 'var(--radius-12)',
}
