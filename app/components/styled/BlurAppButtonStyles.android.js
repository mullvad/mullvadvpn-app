import { colors } from '../../config';

import { createViewStyles } from '../../lib/styles';

export default {
  ...createViewStyles({
    transparent: {
      backgroundColor: colors.white20,
    },
    transparentHover: {
      backgroundColor: colors.white40,
    },
    redTransparent: {
      backgroundColor: colors.red40,
    },
    redTransparentHover: {
      backgroundColor: colors.red45,
    },
  }),
};
