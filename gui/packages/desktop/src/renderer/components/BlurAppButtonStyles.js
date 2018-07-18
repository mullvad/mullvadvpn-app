// @flow

import { Styles } from 'reactxp';
import { colors } from '../../config';

export default {
  transparent: Styles.createViewStyle({
    backgroundColor: colors.white20,
    backdropFilter: 'blur(4px)',
  }),
  transparentHover: Styles.createViewStyle({
    backgroundColor: colors.white40,
    backdropFilter: 'blur(4px)',
  }),
  redTransparent: Styles.createViewStyle({
    backgroundColor: colors.red40,
    backdropFilter: 'blur(4px)',
  }),
  redTransparentHover: Styles.createViewStyle({
    backgroundColor: colors.red45,
    backdropFilter: 'blur(4px)',
  }),
};
