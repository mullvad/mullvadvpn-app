import styled from 'styled-components';
import { colors } from '../../config.json';
import * as Cell from './cell';
import { Container } from './Layout';
import { NavigationScrollbars } from './NavigationBar';
import Selector from './cell/Selector';

export const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

export const StyledInputFrame = styled(Cell.InputFrame)({
  flex: 0,
});

export const StyledSelectorContainer = styled.div({
  flex: 0,
});

export const StyledTunnelProtocolSelector = (styled(Selector)({
  marginBottom: 0,
}) as unknown) as new <T>() => Selector<T>;

export const StyledTunnelProtocolContainer = styled(StyledSelectorContainer)({
  marginBottom: '20px',
});

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const StyledBottomCellGroup = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '22px',
});

export const StyledNoWireguardKeyErrorContainer = styled(Cell.Footer)({
  paddingBottom: 0,
});

export const StyledNoWireguardKeyError = styled(Cell.FooterText)({
  fontWeight: 800,
  color: colors.red,
});
