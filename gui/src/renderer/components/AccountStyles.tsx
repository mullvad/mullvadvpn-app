import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  account: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  account__container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
    paddingBottom: 48,
  }),
  account__scrollview: Styles.createViewStyle({
    flex: 1,
  }),
  account__content: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  account__main: Styles.createViewStyle({
    marginBottom: 24,
  }),
  account__row: Styles.createViewStyle({
    paddingTop: 0,
    paddingBottom: 0,
    paddingLeft: 24,
    paddingRight: 24,
    marginBottom: 24,
  }),
  account__footer: Styles.createViewStyle({
    paddingLeft: 24,
    paddingRight: 24,
  }),
  account__buy_button: Styles.createViewStyle({
    marginBottom: 24,
  }),
  account__row_label: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    letterSpacing: -0.2,
    color: colors.white60,
    marginBottom: 9,
  }),
  account__row_value: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    lineHeight: 19,
    fontWeight: '800',
    color: colors.white,
  }),
  account__out_of_time: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '800',
    color: colors.red,
  }),
  account__footer_label: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    letterSpacing: -0.2,
    color: colors.white80,
  }),
};
