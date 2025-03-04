import { spacingTokens } from '../tokens';

export const spacingPrimitives = {
  '--spc-4': spacingTokens.spc4,
  '--spc-8': spacingTokens.spc8,
  '--spc-16': spacingTokens.spc16,
  '--spc-24': spacingTokens.spc24,
  '--spc-32': spacingTokens.spc32,
} as const;

export enum Spacings {
  tiny = 'var(--spc-4)',
  small = 'var(--spc-8)',
  medium = 'var(--spc-16)',
  large = 'var(--spc-24)',
  big = 'var(--spc-32)',
}
