import styled from 'styled-components';
import { colors } from '../../config.json';
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
