import { colors } from '../../foundations';

export const levels = {
  0: { enabled: colors.brandBlue, disabled: colors.blue40 },
  1: { enabled: colors.blue60, disabled: colors.blue40 },
  2: { enabled: colors.blue40, disabled: colors.blue20 },
  3: { enabled: colors.blue20, disabled: colors.blue10 },
  4: { enabled: colors.blue10, disabled: colors.blue10 },
} as const;
