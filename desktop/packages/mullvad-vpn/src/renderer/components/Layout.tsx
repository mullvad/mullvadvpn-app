import styled from 'styled-components';

import { colors } from '../../config.json';
import { measurements, spacings } from './common-styles';
import HeaderBar from './HeaderBar';
import { NavigationScrollbars } from './NavigationBar';

export const Header = styled(HeaderBar)({
  flex: 0,
});

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

export const SettingsStack = styled.div({
  display: 'flex',
  flexDirection: 'column',
  gap: spacings.spacing5,
});

export const Footer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 0,
  padding: `${spacings.spacing6} ${measurements.horizontalViewMargin} ${measurements.verticalViewMargin}`,
  [`${SettingsContent} &&`]: {
    marginBottom: 0,
  },
});

export const ButtonStack = styled.div({
  display: 'flex',
  flexDirection: 'column',
  gap: spacings.spacing5,
  margin: `0 ${spacings.spacing6}`,
  [`${Footer} &&`]: {
    margin: `0 ${spacings.spacing3}`,
  },
});
