import styled from 'styled-components';
import { colors } from '../../config.json';
import AccountTokenLabel from './AccountTokenLabel';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { bigText, smallText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { DefaultHeaderBar } from './HeaderBar';
import { Container } from './Layout';

export const StyledHeader = styled(DefaultHeaderBar)({
  flex: 0,
});

export const StyledAccountTokenLabel = styled(AccountTokenLabel)({
  fontFamily: 'Open Sans',
  lineHeight: '20px',
  fontSize: '20px',
  fontWeight: 800,
  color: colors.white,
});

export const StyledModalCellContainer = styled(Cell.Container)({
  marginTop: '18px',
  paddingLeft: '12px',
  paddingRight: '12px',
});

const buttonStyle = {
  marginBottom: '18px',
};

export const StyledBuyCreditButton = styled(AppButton.GreenButton)(buttonStyle);
export const StyledDisconnectButton = styled(AppButton.RedButton)(buttonStyle);

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
  padding: '0 22px',
});

export const StyledFooter = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 0,
  padding: '18px 22px 22px',
});

export const StyledTitle = styled.span(bigText, {
  lineHeight: '38px',
  marginBottom: '8px',
});

export const StyledMessage = styled.span(smallText, {
  marginBottom: '20px',
  color: colors.white,
});

export const StyledAccountTokenMessage = styled.span(smallText, {
  color: colors.white,
});

export const StyledStatusIcon = styled.div({
  alignSelf: 'center',
  width: '60px',
  height: '60px',
  marginBottom: '18px',
});

export const StyledAccountTokenContainer = styled.div({
  display: 'flex',
  height: '50px',
  alignItems: 'center',
});
