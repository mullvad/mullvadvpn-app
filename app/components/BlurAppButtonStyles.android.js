// @flow

import { Styles } from 'reactxp';
import { colors } from '../../config';

export default {
  transparent: Styles.createViewStyle({
    backgroundColor: colors.white20,
  }),
  transparentHover: Styles.createViewStyle({
    backgroundColor: colors.white40,
  }),
  redTransparent: Styles.createViewStyle({
    backgroundColor: colors.red40,
  }),
  redTransparentHover: Styles.createViewStyle({
    backgroundColor: colors.red45,
  }),
};
