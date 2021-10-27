import React, { useCallback } from 'react';
import { useSelector } from 'react-redux';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { TunnelState } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useHistory } from '../lib/history';
import { IReduxState } from '../redux/store';
import { FocusFallback } from './Focus';
import { sourceSansPro } from './common-styles';
import ImageView from './ImageView';
import { RoutePath } from '../lib/routes';

export enum HeaderBarStyle {
  default = 'default',
  defaultDark = 'defaultDark',
  error = 'error',
  success = 'success',
}

const headerBarStyleColorMap = {
  [HeaderBarStyle.default]: colors.blue,
  [HeaderBarStyle.defaultDark]: colors.darkBlue,
  [HeaderBarStyle.error]: colors.red,
  [HeaderBarStyle.success]: colors.green,
};

interface IHeaderBarContainerProps {
  barStyle?: HeaderBarStyle;
  unpinnedWindow: boolean;
}

const HeaderBarContainer = styled.header({}, (props: IHeaderBarContainerProps) => ({
  padding: '12px 16px',
  paddingTop: window.env.platform === 'darwin' && !props.unpinnedWindow ? '24px' : '12px',
  backgroundColor: headerBarStyleColorMap[props.barStyle ?? HeaderBarStyle.default],
}));

const HeaderBarContent = styled.div({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'flex-end',
  // In views without the brand components we still want the Header to have the same height.
  minHeight: '51px',
});

interface IHeaderBarProps {
  barStyle?: HeaderBarStyle;
  className?: string;
  children?: React.ReactNode;
}

export default function HeaderBar(props: IHeaderBarProps) {
  const unpinnedWindow = useSelector(
    (state: IReduxState) => state.settings.guiSettings.unpinnedWindow,
  );

  return (
    <HeaderBarContainer
      barStyle={props.barStyle}
      className={props.className}
      unpinnedWindow={unpinnedWindow}>
      <HeaderBarContent>{props.children}</HeaderBarContent>
    </HeaderBarContainer>
  );
}

const BrandContainer = styled.div({
  display: 'flex',
  flex: 1,
  alignItems: 'center',
});

const Title = styled.span({
  ...sourceSansPro,
  fontSize: '27px',
  lineHeight: '30px',
  color: colors.white80,
  marginLeft: '7px',
  letterSpacing: '0.015em',
  position: 'relative',
  top: '-1px',
});

const Logo = styled(ImageView)({
  margin: '4px 0 3px',
});

export function Brand(props: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <BrandContainer {...props}>
      <Logo width={44} height={44} source="logo-icon" />
      <Title>MULLVAD VPN</Title>
    </BrandContainer>
  );
}

const HeaderBarSettingsButtonContainer = styled.button({
  cursor: 'default',
  padding: 0,
  marginLeft: 8,
  backgroundColor: 'transparent',
  border: 'none',
});

export function HeaderBarSettingsButton() {
  const history = useHistory();

  const openSettings = useCallback(() => {
    history.show(RoutePath.settings);
  }, [history]);

  return (
    <HeaderBarSettingsButtonContainer
      onClick={openSettings}
      aria-label={messages.gettext('Settings')}>
      <ImageView
        height={24}
        width={24}
        source="icon-settings"
        tintColor={colors.white60}
        tintHoverColor={colors.white80}
      />
    </HeaderBarSettingsButtonContainer>
  );
}

export function DefaultHeaderBar(props: IHeaderBarProps) {
  return (
    <HeaderBar {...props}>
      <FocusFallback>
        <Brand />
      </FocusFallback>
      <HeaderBarSettingsButton />
    </HeaderBar>
  );
}

export function calculateHeaderBarStyle(tunnelState: TunnelState): HeaderBarStyle {
  switch (tunnelState.state) {
    case 'disconnected':
      return HeaderBarStyle.error;
    case 'connecting':
    case 'connected':
      return HeaderBarStyle.success;
    case 'error':
      return !tunnelState.details.blockFailure ? HeaderBarStyle.success : HeaderBarStyle.error;
    case 'disconnecting':
      switch (tunnelState.details) {
        case 'block':
        case 'reconnect':
          return HeaderBarStyle.success;
        case 'nothing':
          return HeaderBarStyle.error;
      }
  }
}
