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
  wgkeys__row: Styles.createViewStyle({
    paddingTop: 0,
    paddingBottom: 0,
    paddingLeft: 24,
    paddingRight: 24,
    marginBottom: 24,
  }),
  wgkeys__row_label: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    letterSpacing: -0.2,
    color: colors.white60,
  }),
  wgkeys__validity_row: Styles.createViewStyle({
    paddingTop: 5,
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
    fontSize: 16,
    fontWeight: '800',
    color: colors.red,
  }),
  wgkeys__valid_key: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '600',
    lineHeight: 20,
    color: colors.green,
  }),
};
