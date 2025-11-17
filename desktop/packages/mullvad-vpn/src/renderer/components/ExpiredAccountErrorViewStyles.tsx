import styled from 'styled-components';

import { colors } from '../lib/foundations';
import AccountNumberLabel from './AccountNumberLabel';
import { hugeText, measurements, tinyText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { Container } from './Layout';

export const StyledAccountNumberLabel = styled(AccountNumberLabel)({
  fontFamily: 'Open Sans',
  lineHeight: '20px',
  fontSize: '20px',
  fontWeight: 700,
  color: colors.white,
});

export const StyledCustomScrollbars = styled(CustomScrollbars)({
  flex: 1,
});

export const StyledContainer = styled(Container)({
  paddingTop: '22px',
  minHeight: '100%',
  backgroundColor: colors.darkBlue,
});

export const StyledBody = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  padding: `0 ${measurements.horizontalViewMargin}`,
});

export const StyledTitle = styled.span(hugeText, {
  lineHeight: '38px',
  marginBottom: '8px',
});

export const StyledMessage = styled.span(tinyText, {
  marginBottom: '20px',
  color: colors.white,
});

export const StyledAccountNumberMessage = styled.span(tinyText, {
  color: colors.white,
});

export const StyledAccountNumberContainer = styled.div({
  display: 'flex',
  height: '50px',
  alignItems: 'center',
});

export const StyledDeviceLabel = styled.span(tinyText, {
  lineHeight: '20px',
  color: colors.white,
});
