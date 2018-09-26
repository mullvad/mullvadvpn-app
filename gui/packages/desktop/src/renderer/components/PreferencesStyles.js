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

  preferences__cell_label: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 20,
    fontWeight: '900',
    lineHeight: 26,
    letterSpacing: -0.2,
    color: colors.white,
    paddingTop: 14,
    paddingBottom: 14,
    flexGrow: 1,
  }),
};
