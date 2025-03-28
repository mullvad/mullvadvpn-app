import { fadeIn, fadeOut } from './fade';
import { wipeDownIn, wipeVerticalOut } from './wipe';

export const animations = {
  fade: {
    in: fadeIn,
    out: fadeOut,
  },
  wipeDown: {
    in: wipeDownIn,
    out: wipeVerticalOut,
  },
} as const;
