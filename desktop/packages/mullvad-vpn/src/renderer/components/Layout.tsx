import styled from 'styled-components';

import { colors } from '../../config.json';
import { spacings } from '../tokens';
import { Flex } from './common/layout/Flex';
import { measurements } from './common-styles';
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

export const Footer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 0,
  padding: `${spacings.spacing6} ${measurements.horizontalViewMargin} ${measurements.verticalViewMargin}`,
  [`${SettingsContent} &&`]: {
    paddingBottom: 0,
  },
});

export const SettingsStack = styled(Flex).attrs({
  $flexDirection: 'column',
  $gap: 'spacing5',
})({});

export const SettingsGroup = styled(Flex).attrs({
  $flex: 1,
  $flexDirection: 'column',
})({});

export const ButtonStack = styled(Flex).attrs(() => ({
  $flexDirection: 'column',
  $gap: 'spacing5',
  $margin: `0 ${spacings.spacing6}`,
}))({
  [`${Footer} &&`]: {
    margin: `0 ${spacings.spacing3}`,
  },
});

export const LabelStack = styled(Flex).attrs(() => ({
  $flexGrow: 1,
  $flexDirection: 'row',
  $alignItems: 'center',
  $gap: 'spacing3',
}))({});
