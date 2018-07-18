// @flow

import { Styles } from 'reactxp';
import { colors } from '../../config';

export default {
  settings: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  settings__container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  settings__content: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
    justifyContent: 'space-between',
    overflow: 'visible',
  }),
  settings__scrollview: Styles.createViewStyle({
    flexGrow: 1,
    flexShrink: 1,
    flexBasis: '100%',
  }),
  settings__cell_spacer: Styles.createViewStyle({
    height: 24,
    flex: 0,
  }),
  settings__footer: Styles.createViewStyle({
    paddingTop: 24,
    paddingBottom: 24,
    paddingLeft: 24,
    paddingRight: 24,
  }),

  settings__account_paid_until_label__error: Styles.createTextStyle({
    color: colors.red,
  }),
};
