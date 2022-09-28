import styled from 'styled-components';

import { colors } from '../../config.json';
import AccountTokenLabel from './AccountTokenLabel';
import * as Cell from './cell';
import { hugeText, measurements, tinyText } from './common-styles';
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
  fontWeight: 700,
  color: colors.white,
});

export const StyledModalCellContainer = styled(Cell.Container)({
  marginTop: '18px',
  paddingLeft: '12px',
  paddingRight: '12px',
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
  padding: `0 ${measurements.viewMargin}`,
});

export const StyledTitle = styled.span(hugeText, {
  lineHeight: '38px',
  marginBottom: '8px',
});

export const StyledMessage = styled.span(tinyText, {
  marginBottom: '20px',
  color: colors.white,
});

export const StyledAccountTokenMessage = styled.span(tinyText, {
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
