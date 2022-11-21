import styled from 'styled-components';

import { colors } from '../../../config.json';
import { tinyText } from '../common-styles';
import { ScopeBar } from './ScopeBar';

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  overflow: 'visible',
});

export const StyledScopeBar = styled(ScopeBar)({
  marginBottom: '14px',
});

export const StyledNavigationBarAttachment = styled.div({
  padding: '0px 16px 14px',
});

export const StyledFilterIconButton = styled.button({
  justifySelf: 'end',
  borderWidth: 0,
  padding: 0,
  margin: 0,
  cursor: 'default',
  backgroundColor: 'transparent',
});

export const StyledFilterRow = styled.div({
  ...tinyText,
  color: colors.white,
  margin: '0px 6px 14px',
});

export const StyledFilter = styled.div({
  ...tinyText,
  display: 'inline-flex',
  alignItems: 'center',
  backgroundColor: colors.blue,
  borderRadius: '4px',
  padding: '3px 8px',
  marginLeft: '6px',
  color: colors.white,
});

export const StyledClearFilterButton = styled.div({
  display: 'inline-block',
  borderWidth: 0,
  padding: 0,
  margin: '0 0 0 6px',
  cursor: 'default',
  backgroundColor: 'transparent',
});
