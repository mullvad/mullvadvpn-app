import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  wgkeys: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  wgkeys__container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  // plain CSS style
  wgkeys__scrollview: {
    flex: 1,
  },
  wgkeys__content: Styles.createViewStyle({
    // ReactXP don't allow setting 'minHeight' and don't allow percentages. This will work well
    // without the '@ts-ignore' when moving away from ReactXP.
    // @ts-ignore
    minHeight: '100%',
  }),
  wgkeys__messages: Styles.createViewStyle({
    flex: 1,
  }),
  wgkeys__row: Styles.createViewStyle({
    paddingVertical: 0,
    paddingHorizontal: 22,
    marginBottom: 20,
  }),
  wgkeys__button_row: Styles.createViewStyle({
    paddingHorizontal: 22,
    marginBottom: 18,
  }),
  wgkeys__last_button: Styles.createViewStyle({
    marginBottom: 22,
  }),
  wgkeys__row_label: Styles.createTextStyle({
    flex: 1,
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    color: colors.white60,
    marginBottom: 9,
  }),
  wgkeys__row_value: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    lineHeight: 19,
    fontWeight: '800',
    color: colors.white,
  }),
  wgkeys__invalid_key: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '800',
    lineHeight: 20,
    color: colors.red,
  }),
  wgkeys__valid_key: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    color: colors.green,
  }),
};
