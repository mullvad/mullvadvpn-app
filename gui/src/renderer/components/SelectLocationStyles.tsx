import styled from 'styled-components';
import { colors } from '../../config.json';
import { smallText } from './common-styles';
import { Container } from './Layout';
import { ScopeBar } from './ScopeBar';

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

export const StyledNavigationBarAttachment = styled.div({
  marginTop: '8px',
  paddingHorizontal: '4px',
});

export const StyledFilterContainer = styled.div({
  position: 'relative',
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
});

export const StyledFilterByProviderButton = styled.button({
  ...smallText,
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
