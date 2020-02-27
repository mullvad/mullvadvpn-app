import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  container: Styles.createViewStyle({
    flex: 1,
  }),
  body: Styles.createViewStyle({
    flex: 1,
    paddingHorizontal: 24,
    marginTop: 20,
  }),
  footer: Styles.createViewStyle({
    flex: 0,
    paddingVertical: 24,
    paddingHorizontal: 24,
    backgroundColor: colors.darkBlue,
  }),
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 32,
    lineHeight: 40,
    fontWeight: '900',
    color: colors.white,
    marginBottom: 8,
  }),
  message: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    lineHeight: 20,
    fontWeight: '600',
    color: colors.white,
    marginBottom: 24,
  }),
  accountToken: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 24,
    fontWeight: '800',
    color: colors.white,
    marginTop: 15,
  }),
  fieldLabel: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    color: colors.white,
    marginBottom: 9,
  }),
  or: Styles.createTextStyle({
    fontSize: 20,
    lineHeight: 24,
    fontWeight: '800',
    textAlign: 'center',
    color: colors.white,
    marginVertical: 12,
  }),
};
