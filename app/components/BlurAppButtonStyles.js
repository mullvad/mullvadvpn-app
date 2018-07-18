import { createViewStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    transparent: {
      backgroundColor: colors.white20,
      backdropFilter: 'blur(4px)',
    },
    transparentHover: {
      backgroundColor: colors.white40,
      backdropFilter: 'blur(4px)',
    },
    redTransparent: {
      backgroundColor: colors.red40,
      backdropFilter: 'blur(4px)',
    },
    redTransparentHover: {
      backgroundColor: colors.red45,
      backdropFilter: 'blur(4px)',
    },
  }),
};
