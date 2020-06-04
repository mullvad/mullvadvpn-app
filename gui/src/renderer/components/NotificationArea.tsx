import { shell } from 'electron';
import React, { useCallback } from 'react';
import { useSelector } from 'react-redux';
import { Types } from 'reactxp';
import AccountExpiry from '../../shared/account-expiry';
import {
  InAppNotification,
  InAppNotificationProvider,
  AccountExpiryNotificationProvider,
  BlockWhenDisconnectedNotificationProvider,
  ConnectingNotificationProvider,
  ErrorNotificationProvider,
  InconsistentVersionNotificationProvider,
  ReconnectingNotificationProvider,
  UnsupportedVersionNotificationProvider,
  UpdateAvailableNotificationProvider,
} from '../../shared/notifications/notification';
import { useAppContext } from '../context';
import { IReduxState } from '../redux/store';
import {
  NotificationActions,
  NotificationBanner,
  NotificationContent,
  NotificationIndicator,
  NotificationOpenLinkAction,
  NotificationSubtitle,
  NotificationTitle,
} from './NotificationBanner';

interface IProps {
  style?: Types.ViewStyleRuleSet;
}

export default function NotificationArea(props: IProps) {
  const accountExpiry = useSelector((state: IReduxState) =>
    state.account.expiry
      ? new AccountExpiry(state.account.expiry, state.userInterface.locale)
      : undefined,
  );
  const tunnelState = useSelector((state: IReduxState) => state.connection.status);
  const version = useSelector((state: IReduxState) => state.version);
  const blockWhenDisconnected = useSelector(
    (state: IReduxState) => state.settings.blockWhenDisconnected,
  );

  const notifications: InAppNotificationProvider[] = [
    new ConnectingNotificationProvider({ tunnelState }),
    new ReconnectingNotificationProvider(tunnelState),
    new BlockWhenDisconnectedNotificationProvider({ tunnelState, blockWhenDisconnected }),
    new ErrorNotificationProvider(tunnelState),
    new InconsistentVersionNotificationProvider({ consistent: version.consistent }),
    new UnsupportedVersionNotificationProvider(version),
    new UpdateAvailableNotificationProvider(version),
    new AccountExpiryNotificationProvider({ accountExpiry }),
  ];

  const notification = notifications
    .find((notification) => notification.mayDisplay())
    ?.getInAppNotification();

  if (notification != undefined) {
    return (
      <NotificationBanner style={props.style} visible>
        <NotificationIndicator type={notification.indicator} />
        <NotificationContent>
          <NotificationTitle>{notification.title}</NotificationTitle>
          <NotificationSubtitle>{notification.subtitle}</NotificationSubtitle>
        </NotificationContent>
        {notification.action && <NotificationAction notification={notification} />}
      </NotificationBanner>
    );
  } else {
    return <NotificationBanner style={props.style} visible={false} />;
  }
}

interface INotificationActionProps {
  notification: InAppNotification;
}

function NotificationAction(props: INotificationActionProps) {
  const { openLinkWithAuth } = useAppContext();

  const handlePress = useCallback(() => {
    if (props.notification.action!.withAuth) {
      return openLinkWithAuth(props.notification.action!.url);
    } else {
      return shell.openExternal(props.notification.action!.url);
    }
  }, []);

  return (
    <NotificationActions>
      <NotificationOpenLinkAction onPress={handlePress} />
    </NotificationActions>
  );
}
