import styled from 'styled-components';

import { colors } from '../../config.json';
import * as AppButton from './AppButton';
import { measurements, normalText, tinyText } from './common-styles';
import { RedeemVoucherButton } from './RedeemVoucher';

export const AccountContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  paddingBottom: measurements.viewMargin,
});

export const AccountRows = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
});

export const AccountRow = styled.div({
  padding: `0 ${measurements.viewMargin}`,
  marginBottom: measurements.rowVerticalMargin,
});

const AccountRowText = styled.span({
  display: 'block',
  fontFamily: 'Open Sans',
});

export const AccountRowLabel = styled(AccountRowText)(tinyText, {
  lineHeight: '20px',
  marginBottom: '5px',
  color: colors.white60,
});

export const AccountRowValue = styled(AccountRowText)(normalText, {
  fontWeight: 600,
  color: colors.white,
});

export const DeviceRowValue = styled(AccountRowValue)({
  textTransform: 'capitalize',
});

export const AccountOutOfTime = styled(AccountRowValue)({
  color: colors.red,
});

export const StyledSpinnerContainer = styled.div({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: '8px 0',
});

export const AccountFooter = styled.div({
  display: 'flex',
  flexDirection: 'column',
  padding: `0 ${measurements.viewMargin}`,
});

const buttonStyle = {
  marginBottom: '18px',
};

export const StyledRedeemVoucherButton = styled(RedeemVoucherButton)(buttonStyle);
export const StyledBuyCreditButton = styled(AppButton.GreenButton)(buttonStyle);
