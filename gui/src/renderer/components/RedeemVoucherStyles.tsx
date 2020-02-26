import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  textInput: Styles.createTextInputStyle({
    flex: 1,
    overflow: 'hidden',
    paddingTop: 14,
    paddingLeft: 14,
    paddingRight: 14,
    paddingBottom: 14,
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 26,
    color: colors.blue,
    backgroundColor: colors.white,
    borderRadius: 4,
  }),
  redeemVoucherResponseSuccess: Styles.createTextStyle({
    marginTop: 8,
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    color: colors.green,
  }),
  redeemVoucherResponseError: Styles.createTextStyle({
    marginTop: 8,
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '800',
    lineHeight: 20,
    color: colors.red,
  }),
  redeemVoucherResponseEmpty: Styles.createViewStyle({
    height: 20,
    marginTop: 8,
  }),
};
