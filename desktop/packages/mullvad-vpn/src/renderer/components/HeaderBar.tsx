import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { closeToExpiry, formatRemainingTime, hasExpired } from '../../shared/account-expiry';
import { TunnelState } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { capitalizeEveryWord } from '../../shared/string-helpers';
import { Flex, FootnoteMini, IconButton } from '../lib/components';
import { Colors, Spacings } from '../lib/foundations';
import { transitions, useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
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

const StyledHeader = styled.header<{
  $barStyle?: HeaderBarStyle;
  $accountInfoVisible: boolean;
}>(({ $accountInfoVisible, $barStyle }) => ({
  height: $accountInfoVisible ? '80px' : '68px',
  minHeight: $accountInfoVisible ? '80px' : '68px',

  backgroundColor: headerBarStyleColorMap[$barStyle ?? HeaderBarStyle.default],
  transition: 'height 250ms ease-in-out, min-height 250ms ease-in-out',
}));

const StyledLogoRow = styled(Flex)({
  height: '38px',
});

interface IHeaderBarProps {
  barStyle?: HeaderBarStyle;
  className?: string;
  children?: React.ReactNode;
  showAccountInfo?: boolean;
}

export default function HeaderBar(props: IHeaderBarProps) {
  return (
    <StyledHeader
      $barStyle={props.barStyle}
      $accountInfoVisible={props.showAccountInfo ?? false}
      className={props.className}>
      <Flex
        $flexDirection="column"
        $justifyContent="center"
        $margin={{
          horizontal: Spacings.spacing5,
          top: Spacings.spacing5,
          bottom: Spacings.spacing3,
        }}>
        <StyledLogoRow $justifyContent="space-between" $alignItems="center">
          {props.children}
        </StyledLogoRow>
        {props.showAccountInfo && <HeaderBarDeviceInfo />}
      </Flex>
    </StyledHeader>
  );
}

export function Brand(props: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <Flex $flex={1} $alignItems="center" $gap={Spacings.spacing3} {...props}>
      <ImageView width={38} height={38} source="logo-icon" />
      <ImageView height={15.4} source="logo-text" />
    </Flex>
  );
}

const StyledDeviceInfoContainer = styled(Flex)({
  minHeight: '18px',
});

const StyledDeviceLabel = styled(FootnoteMini)({
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
});

const StyledTimeLeftLabel = styled(FootnoteMini)({
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
    <StyledDeviceInfoContainer $flex={1} $alignItems="flex-end" $gap={Spacings.spacing6}>
      <StyledDeviceLabel color={Colors.white80}>
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
        <StyledTimeLeftLabel color={Colors.white80}>
          {sprintf(messages.pgettext('device-management', 'Time left: %(timeLeft)s'), {
            timeLeft: formattedExpiry,
          })}
        </StyledTimeLeftLabel>
      )}
    </StyledDeviceInfoContainer>
  );
}

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
    <IconButton
      icon="icon-settings"
      variant="secondary"
      onClick={openSettings}
      aria-label={messages.gettext('Settings')}
    />
  );
}

export function HeaderBarAccountButton() {
  const history = useHistory();
  const openAccount = useCallback(
    () => history.push(RoutePath.account, { transition: transitions.show }),
    [history],
  );

  return (
    <IconButton
      icon="icon-account"
      variant="secondary"
      onClick={openAccount}
      data-testid="account-button"
      aria-label={messages.gettext('Account settings')}
    />
  );
}

export function DefaultHeaderBar(props: IHeaderBarProps) {
  const loggedIn = useSelector((state) => state.account.status.type === 'ok');

  return (
    <HeaderBar showAccountInfo={loggedIn} {...props}>
      <FocusFallback>
        <Brand />
      </FocusFallback>
      <Flex $gap={Spacings.spacing5} $alignItems="center">
        {loggedIn && <HeaderBarAccountButton />}
        <HeaderBarSettingsButton />
      </Flex>
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
