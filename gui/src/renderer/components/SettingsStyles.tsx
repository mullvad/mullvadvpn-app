import { Styles } from 'reactxp';
import { colors } from '../../config.json';

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
  // plain CSS style
  settings__scrollview: {
    flex: 1,
  },
  settings__cell_spacer: Styles.createViewStyle({
    height: 24,
    flex: 0,
  }),
  settings__cell_footer: Styles.createViewStyle({
    paddingTop: 8,
    paddingRight: 24,
    paddingBottom: 24,
    paddingLeft: 24,
  }),
  settings__footer: Styles.createViewStyle({
    paddingTop: 24,
    paddingBottom: 24,
    paddingLeft: 24,
    paddingRight: 24,
  }),
  settings__version_warning: Styles.createViewStyle({
    marginLeft: 8,
  }),

  settings__account_paid_until_label__error: Styles.createTextStyle({
    color: colors.red,
  }),
  settings__cell_footer_label: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    letterSpacing: -0.2,
    color: colors.white60,
  }),
};
