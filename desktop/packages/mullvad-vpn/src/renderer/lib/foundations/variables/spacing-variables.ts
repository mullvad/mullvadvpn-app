import { SpacingTokens } from '../tokens';

export const spacingPrimitives = {
  '--spc-4': SpacingTokens.spc4,
  '--spc-6': SpacingTokens.spc6,
  '--spc-8': SpacingTokens.spc8,
  '--spc-12': SpacingTokens.spc12,
  '--spc-16': SpacingTokens.spc16,
  '--spc-24': SpacingTokens.spc24,
  '--spc-32': SpacingTokens.spc32,
  '--spc-40': SpacingTokens.spc40,
  '--spc-48': SpacingTokens.spc48,
  '--spc-56': SpacingTokens.spc56,
  '--spc-64': SpacingTokens.spc64,
  '--spc-72': SpacingTokens.spc72,
  '--spc-80': SpacingTokens.spc80,
};

export enum Spacings {
  tiny = 'var(--spc-4)',
  small = 'var(--spc-8)',
  medium = 'var(--spc-16)',
  large = 'var(--spc-24)',
  big = 'var(--spc-32)',
}
