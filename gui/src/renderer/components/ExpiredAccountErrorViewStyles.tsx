import { Styles } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import AccountTokenLabel from './AccountTokenLabel';

export const StyledAccountTokenLabel = styled(AccountTokenLabel)({
  fontFamily: 'Open Sans',
  lineHeight: '24px',
  fontSize: '24px',
  fontWeight: 800,
  color: colors.white,
});

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
  button: Styles.createViewStyle({
    marginBottom: 24,
  }),
};
