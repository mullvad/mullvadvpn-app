import { colors, spacings } from '../../foundations';

export const levels = {
  0: { enabled: colors.blue, disabled: colors.blue40, indent: spacings.medium },
  1: { enabled: colors.blue60, disabled: colors.blue40, indent: spacings.medium },
  2: { enabled: colors.blue40, disabled: colors.blue20, indent: spacings.big },
  3: { enabled: colors.blue20, disabled: colors.blue10, indent: '48px' },
  4: { enabled: colors.blue10, disabled: colors.blue10, indent: '64px' },
} as const;
