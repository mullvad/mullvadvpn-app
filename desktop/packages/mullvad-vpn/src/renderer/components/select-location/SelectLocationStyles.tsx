import styled from 'styled-components';

import * as Cell from '../cell';
import { ScopeBar } from './ScopeBar';

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  overflow: 'visible',
});

export const StyledScopeBar = styled(ScopeBar)({
  marginBottom: '16px',
});

export const StyledNavigationBarAttachment = styled.div({
  padding: '0 16px 16px',
});

export const StyledSelectionUnavailable = styled(Cell.CellFooter)({
  display: 'flex',
  flexDirection: 'column',
  paddingTop: 0,
  marginTop: 0,
});

export const StyledSelectionUnavailableText = styled(Cell.CellFooterText)({
  textAlign: 'center',
});
