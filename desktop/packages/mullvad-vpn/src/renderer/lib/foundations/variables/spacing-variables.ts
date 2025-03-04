import { spacingTokens } from '../tokens';

export const spacings = {
  '--spc-4': spacingTokens.spc4,
  '--spc-8': spacingTokens.spc8,
  '--spc-16': spacingTokens.spc16,
  '--spc-24': spacingTokens.spc24,
  '--spc-32': spacingTokens.spc32,
} as const;

export enum Spacings {
  spacing1 = 'var(--spc-4)',
  spacing2 = 'var(--spc-6)',
  spacing3 = 'var(--spc-8)',
  spacing4 = 'var(--spc-12)',
  spacing5 = 'var(--spc-16)',
  spacing6 = 'var(--spc-24)',
  spacing7 = 'var(--spc-32)',
  spacing8 = 'var(--spc-40)',
  spacing9 = 'var(--spc-48)',
  spacing10 = 'var(--spc-56)',
  spacing11 = 'var(--spc-64)',
  spacing12 = 'var(--spc-72)',
  spacing13 = 'var(--spc-80)',
}
