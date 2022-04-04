import styled from 'styled-components';

import { colors } from '../../config.json';
import HeaderBar from './HeaderBar';

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

export const SettingsContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

export const Layout = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  height: '100vh',
});
