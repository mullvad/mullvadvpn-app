import styled from 'styled-components';

import { colors } from '../../../config.json';
import * as Cell from '../cell';
import { normalText, tinyText } from '../common-styles';
import SearchBar from '../SearchBar';
import { HeaderSubTitle } from '../SettingsHeader';
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
  padding: '0 16px 14px',
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
  margin: '0 6px 14px',
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

export const StyledHeaderSubTitle = styled(HeaderSubTitle)({
  display: 'block',
  margin: '0 6px 14px',
});

export const StyledSearchBar = styled(SearchBar)({
  margin: '0 6px',
});

export const StyledNoResult = styled(Cell.CellFooter)({
  display: 'flex',
  flexDirection: 'column',
  paddingTop: 0,
  marginTop: 0,
});

export const StyledNoResultText = styled(Cell.CellFooterText)({
  textAlign: 'center',
});

export const StyledAllLocationsTitle = styled(Cell.Label)(normalText, {
  fontWeight: 'normal',
});
