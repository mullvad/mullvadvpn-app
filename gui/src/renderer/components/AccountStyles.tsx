import styled from 'styled-components';
import { colors } from '../../config.json';
import * as AppButton from './AppButton';
import { Container } from './Layout';
import { RedeemVoucherButton } from './RedeemVoucher';

export const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
  flexDirection: 'column',
});

export const AccountContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  paddingBottom: '22px',
});

export const AccountRows = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
});

export const AccountRow = styled.div({
  padding: '0 22px',
  marginBottom: '20px',
});

const AccountRowText = styled.span({
  display: 'block',
  fontFamily: 'Open Sans',
});

export const AccountRowLabel = styled(AccountRowText)({
  fontSize: '13px',
  fontWeight: 600,
  lineHeight: '20px',
  marginBottom: '9px',
  color: colors.white60,
});

export const AccountRowValue = styled(AccountRowText)({
  fontSize: '16px',
  lineHeight: '19px',
  fontWeight: 800,
  color: colors.white,
});

export const AccountOutOfTime = styled(AccountRowValue)({
  color: colors.red,
});

export const AccountFooter = styled.div({
  display: 'flex',
  flexDirection: 'column',
  padding: '0 22px',
});

const buttonStyle = {
  marginBottom: '18px',
};

export const StyledRedeemVoucherButton = styled(RedeemVoucherButton)(buttonStyle);
export const StyledBuyCreditButton = styled(AppButton.GreenButton)(buttonStyle);
