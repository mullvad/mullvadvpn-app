import styled from 'styled-components';
import { colors } from '../../config.json';
import { tinyText } from './common-styles';
import { Container } from './Layout';
import { ScopeBar } from './ScopeBar';
import SettingsHeader from './SettingsHeader';

export const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

export const StyledScopeBar = styled(ScopeBar)({
  marginTop: '8px',
});

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  overflow: 'visible',
});

export const StyledNavigationBarAttachment = styled.div({}, (props: { top: number }) => ({
  position: 'sticky',
  top: `${props.top}px`,
  padding: '8px 18px 8px 16px',
  backgroundColor: colors.darkBlue,
  zIndex: 1,
}));

export const StyledFilterContainer = styled.div({
  display: 'flex',
  position: 'relative',
  justifySelf: 'end',
});

export const StyledFilterIconButton = styled.button({
  borderWidth: 0,
  padding: 0,
  margin: 0,
  cursor: 'default',
  backgroundColor: 'transparent',
});

export const StyledFilterMenu = styled.div({
  position: 'absolute',
  top: 'calc(100% + 4px)',
  right: '0',
  borderRadius: '4px',
  backgroundColor: colors.darkBlue,
  overflow: 'hidden',
  zIndex: 2,
});

export const StyledFilterByProviderButton = styled.button(tinyText, {
  borderWidth: 0,
  margin: 0,
  cursor: 'default',
  color: colors.white,
  padding: '7px 15px',
  whiteSpace: 'nowrap',
  borderRadius: 0,
  backgroundColor: colors.blue,
  ':hover': {
    backgroundColor: colors.blue80,
  },
});

export const StyledSettingsHeader = styled(SettingsHeader)({
  paddingLeft: '6px',
  paddingBottom: '11px',
});

export const StyledProviderCountRow = styled.div({
  ...tinyText,
  color: colors.white,
  marginLeft: '6px',
  marginBottom: '8px',
});

export const StyledProvidersCount = styled.div({
  ...tinyText,
  display: 'inline-flex',
  alignItems: 'center',
  backgroundColor: colors.blue,
  borderRadius: '4px',
  padding: '3px 8px',
  marginLeft: '6px',
  color: colors.white,
});

export const StyledClearProvidersButton = styled.div({
  display: 'inline-block',
  borderWidth: 0,
  padding: 0,
  margin: '0 0 0 6px',
  cursor: 'default',
  backgroundColor: 'transparent',
});
