// @flow

import { Styles } from 'reactxp';
import { colors } from '../../config';

export default {
  preferences: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  preferences__container: Styles.createViewStyle({
    display: 'flex',
    flexDirection: 'column',
    flex: 1,
  }),
  preferences__content: Styles.createViewStyle({
    flexDirection: 'column',
    flexGrow: 1,
    flexShrink: 1,
    flexBasis: 'auto',
  }),
};
