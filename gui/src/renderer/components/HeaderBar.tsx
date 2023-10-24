import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { closeToExpiry, formatRemainingTime, hasExpired } from '../../shared/account-expiry';
import { TunnelState } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { capitalizeEveryWord } from '../../shared/string-helpers';
import { transitions, useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import { tinyText } from './common-styles';
import { FocusFallback } from './Focus';
import ImageView from './ImageView';

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
  $barStyle?: HeaderBarStyle;
  $accountInfoVisible: boolean;
  $unpinnedWindow: boolean;
}

const HeaderBarContainer = styled.header<IHeaderBarContainerProps>((props) => ({
  padding: '15px 16px 0px',
  minHeight: props.$accountInfoVisible ? '80px' : '68px',
  height: props.$accountInfoVisible ? '80px' : '68px',
  backgroundColor: headerBarStyleColorMap[props.$barStyle ?? HeaderBarStyle.default],
  transitionProperty: 'height, min-height',
  transitionDuration: '250ms',
  transitionTimingFunction: 'ease-in-out',
}));

const HeaderBarContent = styled.div({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'flex-end',
  height: '38px',
});

interface IHeaderBarProps {
  barStyle?: HeaderBarStyle;
  className?: string;
  children?: React.ReactNode;
  showAccountInfo?: boolean;
}

export default function HeaderBar(props: IHeaderBarProps) {
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);

  return (
    <HeaderBarContainer
      $barStyle={props.barStyle}
      className={props.className}
      $accountInfoVisible={props.showAccountInfo ?? false}
      $unpinnedWindow={unpinnedWindow}>
      <HeaderBarContent>{props.children}</HeaderBarContent>
      {props.showAccountInfo && <HeaderBarDeviceInfo />}
    </HeaderBarContainer>
  );
}

const BrandContainer = styled.div({
  display: 'flex',
  flex: 1,
  alignItems: 'center',
});

const Title = styled(ImageView)({
  opacity: 0.8,
  marginLeft: '9px',
});

export function Brand(props: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <BrandContainer {...props}>
      <ImageView width={38} height={38} source="logo-icon" />
      <Title height={15.4} source="logo-text" />
    </BrandContainer>
  );
}

const StyledAccountInfo = styled.div({
  display: 'flex',
  marginTop: '2px',
  maxWidth: '100%',
});

const StyledDeviceLabel = styled.div(tinyText, {
  fontSize: '10px',
  color: colors.white80,
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
});

const StyledTimeLeftLabel = styled.div(tinyText, {
  fontSize: '10px',
  color: colors.white80,
  marginLeft: '16px',
  whiteSpace: 'nowrap',
});

function HeaderBarDeviceInfo() {
  const deviceName = useSelector((state) => state.account.deviceName);
  const accountExpiry = useSelector((state) => state.account.expiry);
  const isOutOfTime = accountExpiry ? hasExpired(accountExpiry) : false;
  const formattedExpiry = isOutOfTime
    ? sprintf(messages.ngettext('1 day', '%d days', 0), 0)
    : accountExpiry
    ? formatRemainingTime(accountExpiry)
    : '';

  return (
    <StyledAccountInfo>
      <StyledDeviceLabel>
        {sprintf(
          // TRANSLATORS: A label that will display the newly created device name to inform the user
          // TRANSLATORS: about it.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(deviceName)s - The name of the current device
          messages.pgettext('device-management', 'Device name: %(deviceName)s'),
          {
            deviceName: capitalizeEveryWord(deviceName ?? ''),
          },
        )}
      </StyledDeviceLabel>
      {accountExpiry && !closeToExpiry(accountExpiry) && !isOutOfTime && (
        <StyledTimeLeftLabel>
          {sprintf(messages.pgettext('device-management', 'Time left: %(timeLeft)s'), {
            timeLeft: formattedExpiry,
          })}
        </StyledTimeLeftLabel>
      )}
    </StyledAccountInfo>
  );
}

const HeaderBarSettingsButtonContainer = styled.button({
  cursor: 'default',
  padding: 0,
  marginLeft: 8,
  backgroundColor: 'transparent',
  border: 'none',
});

const HeaderBarAccountButtonContainer = styled(HeaderBarSettingsButtonContainer)({
  marginRight: '16px',
});

interface IHeaderBarSettingsButtonProps {
  disabled?: boolean;
}

export function HeaderBarSettingsButton(props: IHeaderBarSettingsButtonProps) {
  const history = useHistory();

  const openSettings = useCallback(() => {
    if (!props.disabled) {
      history.push(RoutePath.settings, { transition: transitions.show });
    }
  }, [history, props.disabled]);

  return (
    <HeaderBarSettingsButtonContainer
      onClick={openSettings}
      aria-label={messages.gettext('Settings')}>
      <ImageView
        height={24}
        width={24}
        source="icon-settings"
        tintColor={props.disabled ? colors.white40 : colors.white60}
        tintHoverColor={props.disabled ? colors.white40 : colors.white80}
      />
    </HeaderBarSettingsButtonContainer>
  );
}

export function HeaderBarAccountButton() {
  const history = useHistory();
  const openAccount = useCallback(
    () => history.push(RoutePath.account, { transition: transitions.show }),
    [history],
  );

  return (
    <HeaderBarAccountButtonContainer
      onClick={openAccount}
      data-testid="account-button"
      aria-label={messages.gettext('Account settings')}>
      <ImageView
        height={24}
        width={24}
        source="icon-account"
        tintColor={colors.white60}
        tintHoverColor={colors.white80}
      />
    </HeaderBarAccountButtonContainer>
  );
}

export function DefaultHeaderBar(props: IHeaderBarProps) {
  const loggedIn = useSelector((state) => state.account.status.type === 'ok');

  return (
    <HeaderBar showAccountInfo={loggedIn} {...props}>
      <FocusFallback>
        <Brand />
      </FocusFallback>
      {loggedIn && <HeaderBarAccountButton />}
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
      return !tunnelState.details.blockingError ? HeaderBarStyle.success : HeaderBarStyle.error;
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
