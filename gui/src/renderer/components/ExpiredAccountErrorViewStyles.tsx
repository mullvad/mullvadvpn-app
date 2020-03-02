import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  container: Styles.createViewStyle({
    flex: 1,
    paddingTop: 24,
  }),
  body: Styles.createViewStyle({
    flex: 1,
    paddingHorizontal: 24,
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
    fontWeight: '900',
    lineHeight: 40,
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
  statusIcon: Styles.createViewStyle({
    alignSelf: 'center',
    width: 60,
    height: 60,
    marginBottom: 18,
  }),
  fieldLabel: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    color: colors.white,
    marginBottom: 9,
  }),
  accountTokenMessage: Styles.createViewStyle({
    marginBottom: 0,
  }),
  accountTokenFieldLabel: Styles.createTextStyle({
    marginBottom: 0,
  }),
  accountTokenContainer: Styles.createViewStyle({
    height: 68,
    justifyContent: 'center',
  }),
  accountToken: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    lineHeight: 24,
    fontSize: 24,
    fontWeight: '800',
    color: colors.white,
  }),
  button: Styles.createViewStyle({
    marginBottom: 24,
  }),
};
