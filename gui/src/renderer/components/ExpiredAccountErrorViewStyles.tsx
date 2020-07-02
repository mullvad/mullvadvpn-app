import { Styles } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import AccountTokenLabel from './AccountTokenLabel';
import * as AppButton from './AppButton';
import * as Cell from './Cell';

export const StyledAccountTokenLabel = styled(AccountTokenLabel)({
  fontFamily: 'Open Sans',
  lineHeight: '24px',
  fontSize: '24px',
  fontWeight: 800,
  color: colors.white,
});

export const ModalCellContainer = styled(Cell.Container)({
  marginTop: '18px',
  paddingLeft: '12px',
  paddingRight: '12px',
});

const buttonStyle = {
  marginBottom: '18px',
};

export const StyledBuyCreditButton = styled(AppButton.GreenButton)(buttonStyle);
export const StyledDisconnectButton = styled(AppButton.RedButton)(buttonStyle);

export default {
  // plain CSS style
  scrollview: {
    flex: 1,
  },
  container: Styles.createViewStyle({
    flex: 1,
    paddingTop: 22,
    // ReactXP don't allow setting 'minHeight' and don't allow percentages. This will work well
    // without the '@ts-ignore' when moving away from ReactXP.
    // @ts-ignore
    minHeight: '100%',
  }),
  body: Styles.createViewStyle({
    flex: 1,
    paddingHorizontal: 22,
  }),
  footer: Styles.createViewStyle({
    flex: 0,
    paddingTop: 18,
    paddingBottom: 22,
    paddingHorizontal: 22,
    backgroundColor: colors.darkBlue,
  }),
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 30,
    fontWeight: '900',
    lineHeight: 38,
    color: colors.white,
    marginBottom: 8,
  }),
  message: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    lineHeight: 20,
    fontWeight: '600',
    color: colors.white,
    marginBottom: 20,
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
};
