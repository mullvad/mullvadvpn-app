import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  settings: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  content: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
    justifyContent: 'space-between',
    overflow: 'visible',
  }),
  // plain CSS style
  scrollview: {
    flex: 1,
  },
  cellSpacer: Styles.createViewStyle({
    height: 24,
    flex: 0,
  }),
  cellFooter: Styles.createViewStyle({
    paddingTop: 8,
    paddingRight: 24,
    paddingBottom: 24,
    paddingLeft: 24,
  }),
  quitButtonFooter: Styles.createViewStyle({
    paddingTop: 24,
    paddingBottom: 19,
    paddingLeft: 24,
    paddingRight: 24,
  }),
  accountPaidUntilErrorLabel: Styles.createTextStyle({
    color: colors.red,
  }),
  cellFooterLabel: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 20,
    letterSpacing: -0.2,
    color: colors.white60,
  }),
  appVersionLabel: Styles.createTextStyle({
    flex: 0,
  }),
};
