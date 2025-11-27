import styled from 'styled-components';

import { colors } from '../lib/foundations';
import { measurements } from './common-styles';
import { NavigationScrollbars } from './NavigationScrollbars';

export const Container = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  backgroundColor: colors.blue,
  overflow: 'hidden',
});

export const Layout = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  height: '100vh',
  maxWidth: '100%',
});

export const SettingsContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

export const SettingsNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const SettingsContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  overflow: 'visible',
  marginBottom: measurements.verticalViewMargin,
});
