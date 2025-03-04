import styled from 'styled-components';

import { Flex } from '../lib/components';
import { Colors, spacings } from '../lib/foundations';
import { measurements } from './common-styles';
import { NavigationScrollbars } from './NavigationScrollbars';

export const Container = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  backgroundColor: Colors.blue,
  overflow: 'hidden',
});

export const Layout = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  height: '100vh',
});

export const SettingsContainer = styled(Container)({
  backgroundColor: Colors.darkBlue,
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
  padding: `${spacings.large} ${measurements.horizontalViewMargin} ${measurements.verticalViewMargin}`,
  [`${SettingsContent} &&`]: {
    paddingBottom: 0,
  },
});

export const SettingsStack = styled(Flex).attrs({
  $flexDirection: 'column',
  $gap: 'medium',
})({});

export const SettingsGroup = styled(Flex).attrs({
  $flex: 1,
  $flexDirection: 'column',
})({});

export const ButtonStack = styled(Flex).attrs({
  $flexDirection: 'column',
  $gap: 'medium',
})({
  [`${Footer} &&`]: {
    margin: `0 ${spacings.small}`,
  },
});

export const LabelStack = styled(Flex).attrs({
  $flexGrow: 1,
  $flexDirection: 'row',
  $alignItems: 'center',
  $gap: 'small',
})({});
